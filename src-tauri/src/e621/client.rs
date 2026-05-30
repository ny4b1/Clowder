use std::io::Write;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use tokio::sync::Mutex;
use wreq::header::{
    ACCEPT, ACCEPT_LANGUAGE, ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, HeaderMap,
    HeaderName, HeaderValue, RANGE, USER_AGENT,
};
use wreq_util::Emulation;

use super::types::{
    Comment, CommentCreateResponse, CommentUpdateResponse, CommentsResponse, Credentials, Post,
    PostUpdateResponse, PostsResponse, Tag, TagsResponse,
};
use crate::site::Site;

const MAX_LIMIT: u32 = 320;
const MIN_LIMIT: u32 = 8;
const EMULATION_PROFILE: Emulation = Emulation::Chrome136;

pub const MAX_DOWNLOAD_BYTES: u64 = 1024 * 1024 * 1024;
pub const MAX_MEDIA_RESPONSE_BYTES: u64 = 256 * 1024 * 1024;

pub const SESSION_EXPIRED: &str = "Your session expired. Sign in again.";

fn host_accepts_credentials(host: &str, domains: &[&str]) -> bool {
    domains
        .iter()
        .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
}

const USER_AGENT_VALUE: &str = concat!(
    "clowder/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/nyattic/Clowder)"
);

#[derive(Debug, Clone)]
pub struct SearchOutcome {
    pub posts: Vec<Post>,
}

pub struct MediaResponse {
    pub status: u16,
    pub content_type: Option<String>,
    pub content_length: Option<String>,
    pub content_range: Option<String>,
    pub accept_ranges: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct Client {
    site: Site,
    http: wreq::Client,
    limiter: Arc<Mutex<Instant>>,
    credentials: Arc<RwLock<Option<Credentials>>>,
}

impl Client {
    pub async fn new(site: Site, proxy_url: Option<&str>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));

        let mut builder = wreq::Client::builder()
            .emulation(EMULATION_PROFILE)
            .default_headers(headers)
            .timeout(Duration::from_secs(45))
            .connect_timeout(Duration::from_secs(15));

        if let Some(url) = proxy_url {
            let proxy =
                wreq::Proxy::all(url).with_context(|| format!("configure SOCKS5 proxy `{url}`"))?;
            builder = builder.proxy(proxy);
        }

        let http = builder.build().context("build wreq client")?;

        let initial_limiter = Instant::now()
            .checked_sub(Duration::from_secs(1))
            .unwrap_or_else(Instant::now);

        Ok(Self {
            site,
            http,
            limiter: Arc::new(Mutex::new(initial_limiter)),
            credentials: Arc::new(RwLock::new(None)),
        })
    }

    fn host(&self) -> &'static str {
        self.site.host()
    }

    fn url(&self, path: &str) -> String {
        format!("https://{}/{path}", self.host())
    }

    pub fn set_credentials(&self, creds: Option<Credentials>) {
        *self.credentials.write().expect("credentials lock") = creds;
    }

    pub fn current_username(&self) -> Option<String> {
        self.credentials
            .read()
            .expect("credentials lock")
            .as_ref()
            .map(|c| c.username.clone())
    }

    fn auth(&self) -> Option<Credentials> {
        self.credentials.read().expect("credentials lock").clone()
    }

    fn apply_auth(&self, req: wreq::RequestBuilder) -> wreq::RequestBuilder {
        match self.auth() {
            Some(creds) => req.basic_auth(&creds.username, Some(&creds.api_key)),
            None => req,
        }
    }

    fn require_auth(&self, action: &'static str) -> Result<Credentials> {
        self.auth()
            .ok_or_else(|| anyhow!("login required to {action}"))
    }

    fn auth_expired(&self, status: u16) -> Option<anyhow::Error> {
        if status == 401 && self.auth().is_some() {
            self.set_credentials(None);
            Some(anyhow!(SESSION_EXPIRED))
        } else {
            None
        }
    }

    async fn fail(&self, resp: wreq::Response, action: &'static str) -> anyhow::Error {
        if let Some(err) = self.auth_expired(resp.status().as_u16()) {
            return err;
        }
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow!("{action} failed: HTTP {status} {}", trim_body(&body))
    }

    pub async fn search(&self, tags: &str, page: u32, limit: u32) -> Result<SearchOutcome> {
        self.wait_for_api_slot().await;

        let page = page.max(1);
        let limit = limit.clamp(MIN_LIMIT, MAX_LIMIT);
        let req = self.http.get(self.url("posts.json")).query(&[
            ("tags", tags.trim()),
            ("limit", &limit.to_string()),
            ("page", &page.to_string()),
        ]);

        let resp = self
            .apply_auth(req)
            .send()
            .await
            .context("send e621 search request")?;

        let status = resp.status();
        if !status.is_success() {
            if let Some(err) = self.auth_expired(status.as_u16()) {
                return Err(err);
            }
            let body = resp.text().await.unwrap_or_default();
            if status.as_u16() == 403 && is_cloudflare_challenge(&body) {
                return Err(anyhow!(
                    "Cloudflare blocked the request. Try a different VPN exit node."
                ));
            }
            return Err(anyhow!("search failed: HTTP {status} {}", trim_body(&body)));
        }

        let parsed: PostsResponse = resp.json().await.context("decode posts response")?;
        Ok(SearchOutcome {
            posts: parsed.posts,
        })
    }

    pub async fn autocomplete_tags(&self, term: &str, category: Option<u8>) -> Result<Vec<Tag>> {
        let term = term.trim().trim_start_matches('-').replace(' ', "_");
        if term.len() < 2 && category.is_none() {
            return Ok(Vec::new());
        }

        self.wait_for_api_slot().await;

        let name_matches = if term.is_empty() {
            "*".to_string()
        } else {
            format!("{term}*")
        };
        let mut query = vec![
            ("search[name_matches]", name_matches),
            ("search[order]", "count".to_string()),
            ("search[hide_empty]", "true".to_string()),
            ("limit", "12".to_string()),
        ];
        if let Some(category) = category {
            query.push(("search[category]", category.to_string()));
        }

        let resp = self
            .apply_auth(
                self.http.get(self.url("tags.json")).query(&query),
            )
            .send()
            .await
            .context("send tag autocomplete request")?;

        if !resp.status().is_success() {
            return Err(self.fail(resp, "tag autocomplete").await);
        }

        match resp
            .json::<TagsResponse>()
            .await
            .context("decode tag autocomplete response")?
        {
            TagsResponse::List(tags) | TagsResponse::Empty { tags } => Ok(tags),
        }
    }

    pub async fn validate(&self, creds: &Credentials) -> Result<()> {
        self.wait_for_api_slot().await;

        let resp = self
            .http
            .get(self.url("favorites.json"))
            .query(&[("limit", "1")])
            .basic_auth(&creds.username, Some(&creds.api_key))
            .send()
            .await
            .context("send credential validation request")?;

        let status = resp.status();
        if status.as_u16() == 401 {
            return Err(anyhow!("invalid username or API key"));
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            if status.as_u16() == 403 && is_cloudflare_challenge(&body) {
                return Err(anyhow!(
                    "Cloudflare blocked the request (the network or VPN you're using is flagged). Try a different exit node."
                ));
            }
            if status.as_u16() == 403 {
                return Err(anyhow!("invalid username or API key"));
            }
            return Err(anyhow!(
                "credential check failed: HTTP {status} {}",
                trim_body(&body)
            ));
        }
        Ok(())
    }

    pub async fn favorite(&self, post_id: u64) -> Result<()> {
        let creds = self.require_auth("favorite")?;
        self.wait_for_api_slot().await;

        let resp = self
            .http
            .post(self.url("favorites.json"))
            .query(&[("post_id", post_id.to_string())])
            .basic_auth(&creds.username, Some(&creds.api_key))
            .send()
            .await
            .context("send favorite request")?;

        let status = resp.status();
        if status.is_success() || status.as_u16() == 422 {
            return Ok(());
        }
        Err(self.fail(resp, "favorite").await)
    }

    pub async fn unfavorite(&self, post_id: u64) -> Result<()> {
        let creds = self.require_auth("unfavorite")?;
        self.wait_for_api_slot().await;

        let resp = self
            .http
            .delete(self.url(&format!("favorites/{post_id}.json")))
            .basic_auth(&creds.username, Some(&creds.api_key))
            .send()
            .await
            .context("send unfavorite request")?;

        let status = resp.status();
        if status.is_success() || status.as_u16() == 404 {
            return Ok(());
        }
        Err(self.fail(resp, "unfavorite").await)
    }

    pub async fn update_post_tags(
        &self,
        post_id: u64,
        tag_string_diff: &str,
        edit_reason: &str,
    ) -> Result<Post> {
        let creds = self.require_auth("edit tags")?;
        let tag_string_diff = tag_string_diff.trim();
        if tag_string_diff.is_empty() {
            return Err(anyhow!("tag changes are required"));
        }

        self.wait_for_api_slot().await;

        let mut form = vec![("post[tag_string_diff]", tag_string_diff.to_string())];
        let edit_reason = edit_reason.trim();
        if !edit_reason.is_empty() {
            form.push(("post[edit_reason]", edit_reason.to_string()));
        }

        let resp = self
            .http
            .patch(self.url(&format!("posts/{post_id}.json")))
            .form(&form)
            .basic_auth(&creds.username, Some(&creds.api_key))
            .send()
            .await
            .context("send post tag update request")?;

        if !resp.status().is_success() {
            return Err(self.fail(resp, "tag update").await);
        }

        match resp
            .json::<PostUpdateResponse>()
            .await
            .context("decode updated post response")?
        {
            PostUpdateResponse::Post(post) | PostUpdateResponse::Wrapped { post } => Ok(post),
        }
    }

    pub async fn comments(&self, post_id: u64, limit: u32) -> Result<Vec<Comment>> {
        self.wait_for_api_slot().await;

        let limit = limit.clamp(1, MAX_LIMIT);
        let resp = self
            .apply_auth(
                self.http.get(self.url("comments.json")).query(&[
                        ("search[post_id]", post_id.to_string()),
                        ("limit", limit.to_string()),
                        ("group_by", "comment".to_string()),
                    ]),
            )
            .send()
            .await
            .context("send comments request")?;

        if !resp.status().is_success() {
            return Err(self.fail(resp, "comments").await);
        }

        match resp
            .json::<CommentsResponse>()
            .await
            .context("decode comments response")?
        {
            CommentsResponse::List(comments) | CommentsResponse::Empty { comments } => Ok(comments),
        }
    }

    pub async fn create_comment(&self, post_id: u64, body: &str) -> Result<Comment> {
        let creds = self.require_auth("comment")?;
        let body = body.trim();
        if body.is_empty() {
            return Err(anyhow!("comment body is required"));
        }

        self.wait_for_api_slot().await;

        let resp = self
            .http
            .post(self.url("comments.json"))
            .form(&[
                ("comment[post_id]", post_id.to_string()),
                ("comment[body]", body.to_string()),
            ])
            .basic_auth(&creds.username, Some(&creds.api_key))
            .send()
            .await
            .context("send create comment request")?;

        if !resp.status().is_success() {
            return Err(self.fail(resp, "comment").await);
        }

        match resp
            .json::<CommentCreateResponse>()
            .await
            .context("decode created comment response")?
        {
            CommentCreateResponse::Comment(comment)
            | CommentCreateResponse::Wrapped { comment } => Ok(comment),
        }
    }

    pub async fn hide_comment(&self, comment_id: u64) -> Result<Comment> {
        let creds = self.require_auth("hide comments")?;

        self.wait_for_api_slot().await;

        let resp = self
            .http
            .post(self.url(&format!("comments/{comment_id}/hide.json")))
            .basic_auth(&creds.username, Some(&creds.api_key))
            .send()
            .await
            .context("send hide comment request")?;

        if !resp.status().is_success() {
            return Err(self.fail(resp, "hide comment").await);
        }

        let body = resp.text().await.context("read hidden comment response")?;
        if body.trim().is_empty() {
            return Ok(Comment {
                id: comment_id,
                is_hidden: true,
                ..Comment::default()
            });
        }

        match serde_json::from_str::<CommentUpdateResponse>(&body)
            .context("decode hidden comment response")?
        {
            CommentUpdateResponse::Comment(comment)
            | CommentUpdateResponse::Wrapped { comment } => Ok(comment),
        }
    }

    pub async fn download_to_file(
        &self,
        url: &str,
        dest: &Path,
        mut file: std::fs::File,
        max_bytes: u64,
    ) -> Result<u64> {
        let parsed = url::Url::parse(url).context("parse media URL")?;
        let host = parsed
            .host_str()
            .ok_or_else(|| anyhow!("media URL has no host"))?
            .to_string();

        let mut req = self.http.get(parsed.as_str());
        if host_accepts_credentials(&host, self.site.credential_domains()) {
            req = self.apply_auth(req);
        }

        let mut resp = req.send().await.context("send download request")?;
        let status = resp.status();
        if !status.is_success() {
            return Err(anyhow!("download failed: HTTP {status}"));
        }

        if let Some(length) = parse_content_length(resp.headers())
            && length > max_bytes
        {
            return Err(anyhow!(
                "file is too large to download ({length} bytes > {max_bytes})"
            ));
        }

        let mut total: u64 = 0;
        while let Some(chunk) = resp.chunk().await.context("read download chunk")? {
            total = total.saturating_add(chunk.len() as u64);
            if total > max_bytes {
                drop(file);
                let _ = std::fs::remove_file(dest);
                return Err(anyhow!(
                    "file exceeds download cap of {max_bytes} bytes; aborted"
                ));
            }
            file.write_all(&chunk).context("write download chunk")?;
        }
        file.flush().context("flush download file")?;
        Ok(total)
    }

    pub async fn download_media(&self, url: &str, range: Option<&str>) -> Result<MediaResponse> {
        let parsed = url::Url::parse(url).context("parse media URL")?;
        let host = parsed
            .host_str()
            .ok_or_else(|| anyhow!("media URL has no host"))?
            .to_string();

        let mut req = self.http.get(parsed.as_str());
        if let Some(range) = range {
            req = req.header(RANGE, range);
        }
        if host_accepts_credentials(&host, self.site.credential_domains()) {
            req = self.apply_auth(req);
        }

        let mut resp = req.send().await.context("send media request")?;
        let status = resp.status();
        if !status.is_success() && status.as_u16() != 206 {
            return Err(anyhow!("media download failed: HTTP {status}"));
        }
        let headers = resp.headers().clone();
        if let Some(length) = parse_content_length(&headers)
            && length > MAX_MEDIA_RESPONSE_BYTES
        {
            return Err(anyhow!(
                "media response is too large ({length} bytes > {MAX_MEDIA_RESPONSE_BYTES})"
            ));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = resp.chunk().await.context("read media chunk")? {
            if bytes.len() as u64 + chunk.len() as u64 > MAX_MEDIA_RESPONSE_BYTES {
                return Err(anyhow!(
                    "media response exceeds cap of {MAX_MEDIA_RESPONSE_BYTES} bytes"
                ));
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(MediaResponse {
            status: status.as_u16(),
            content_type: header_string(&headers, CONTENT_TYPE),
            content_length: header_string(&headers, CONTENT_LENGTH),
            content_range: header_string(&headers, CONTENT_RANGE),
            accept_ranges: header_string(&headers, ACCEPT_RANGES),
            bytes,
        })
    }

    async fn wait_for_api_slot(&self) {
        let mut last = self.limiter.lock().await;
        let elapsed = last.elapsed();
        let min_gap = Duration::from_secs(1);
        if elapsed < min_gap {
            tokio::time::sleep(min_gap - elapsed).await;
        }
        *last = Instant::now();
    }
}

fn header_string(headers: &HeaderMap, name: HeaderName) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
}

fn parse_content_length(headers: &HeaderMap) -> Option<u64> {
    headers
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
}

fn is_cloudflare_challenge(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    lower.contains("cdn-cgi/") || lower.contains("cloudflare") || lower.contains("just a moment")
}

fn trim_body(body: &str) -> String {
    let mut out = body.trim().replace('\n', " ");
    if out.len() > 240 {
        out.truncate(240);
        out.push_str("...");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_accepts_credentials_for_e621_apex() {
        let domains = Site::E621.credential_domains();
        assert!(host_accepts_credentials("e621.net", domains));
        assert!(host_accepts_credentials("e926.net", domains));
    }

    #[test]
    fn host_accepts_credentials_for_subdomains() {
        let domains = Site::E621.credential_domains();
        assert!(host_accepts_credentials("static1.e621.net", domains));
        assert!(host_accepts_credentials("static2.e621.net", domains));
        assert!(host_accepts_credentials("static1.e926.net", domains));
        assert!(host_accepts_credentials("api.e621.net", domains));
    }

    #[test]
    fn host_accepts_credentials_for_e6ai() {
        let domains = Site::E6ai.credential_domains();
        assert!(host_accepts_credentials("e6ai.net", domains));
        assert!(host_accepts_credentials("static1.e6ai.net", domains));
        assert!(!host_accepts_credentials("e621.net", domains));
    }

    #[test]
    fn host_accepts_credentials_rejects_other_hosts() {
        let domains = Site::E621.credential_domains();
        assert!(!host_accepts_credentials("", domains));
        assert!(!host_accepts_credentials("example.com", domains));
        assert!(!host_accepts_credentials("e621.net.evil.com", domains));
        assert!(!host_accepts_credentials("notrealle621.net", domains));
        assert!(!host_accepts_credentials("e621-net", domains));
    }

    #[test]
    fn detects_cloudflare_challenge_pages() {
        assert!(is_cloudflare_challenge(
            "<html><a href=\"https://e621.net/cdn-cgi/content?id=abc\"></a>"
        ));
        assert!(is_cloudflare_challenge("Just a moment..."));
        assert!(is_cloudflare_challenge(
            "<title>Cloudflare Error 1020: Access Denied</title>"
        ));
        assert!(!is_cloudflare_challenge(
            "regular error body without markers"
        ));
        assert!(!is_cloudflare_challenge(
            "{\"success\":false,\"reason\":\"banned\"}"
        ));
    }

    #[test]
    fn trim_body_truncates_long_content() {
        let long = "x".repeat(500);
        let trimmed = trim_body(&long);
        assert!(trimmed.ends_with("..."));
        assert_eq!(trimmed.len(), 243);
    }

    #[test]
    fn trim_body_keeps_short_content() {
        assert_eq!(trim_body("hello"), "hello");
        assert_eq!(trim_body("  hello\nworld  "), "hello world");
    }
}
