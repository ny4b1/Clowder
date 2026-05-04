use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use reqwest::header::{
    ACCEPT, ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, HeaderMap, HeaderValue,
    RANGE, USER_AGENT,
};
use tokio::sync::Mutex;

use super::ech::configure_ech_client;
use super::types::{
    Comment, CommentCreateResponse, CommentUpdateResponse, CommentsResponse, Credentials, Post,
    PostUpdateResponse, PostsResponse, Tag, TagsResponse,
};
use crate::settings::DohProvider;

const HOST: &str = "e621.net";
const MAX_LIMIT: u32 = 320;
const MIN_LIMIT: u32 = 8;
const CREDENTIAL_DOMAINS: &[&str] = &["e621.net", "e926.net"];

fn host_accepts_credentials(host: &str) -> bool {
    CREDENTIAL_DOMAINS
        .iter()
        .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
}
const USER_AGENT_VALUE: &str = concat!(
    "clowder/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/nyabi021/Clowder)"
);

#[derive(Debug, Clone)]
pub struct SearchOutcome {
    pub posts: Vec<Post>,
    pub ech_enabled: bool,
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
    api_http: reqwest::Client,
    media_clients: Arc<Mutex<HashMap<String, reqwest::Client>>>,
    limiter: Arc<Mutex<Instant>>,
    credentials: Arc<RwLock<Option<Credentials>>>,
    ech_enabled: bool,
    doh_provider: DohProvider,
    fail_closed_ech: bool,
}

impl Client {
    pub async fn new(doh_provider: DohProvider, fail_closed_ech: bool) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let builder = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(45))
            .connect_timeout(Duration::from_secs(15));

        let configured = configure_ech_client(builder, HOST, fail_closed_ech, doh_provider).await?;
        let api_http = configured
            .builder
            .build()
            .context("build e621 API client")?;

        Ok(Self {
            api_http,
            media_clients: Arc::new(Mutex::new(HashMap::new())),
            limiter: Arc::new(Mutex::new(Instant::now() - Duration::from_secs(1))),
            credentials: Arc::new(RwLock::new(None)),
            ech_enabled: configured.ech_enabled,
            doh_provider,
            fail_closed_ech,
        })
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

    pub async fn search(&self, tags: &str, page: u32, limit: u32) -> Result<SearchOutcome> {
        self.wait_for_api_slot().await;

        let page = page.max(1);
        let limit = limit.clamp(MIN_LIMIT, MAX_LIMIT);
        let mut req = self
            .api_http
            .get(format!("https://{HOST}/posts.json"))
            .query(&[
                ("tags", tags.trim()),
                ("limit", &limit.to_string()),
                ("page", &page.to_string()),
            ]);
        if let Some(creds) = self.auth() {
            req = req.basic_auth(&creds.username, Some(&creds.api_key));
        }

        let resp = req.send().await.context("send e621 search request")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("search failed: HTTP {status} {}", trim_body(&body)));
        }

        let parsed: PostsResponse = resp.json().await.context("decode posts response")?;
        Ok(SearchOutcome {
            posts: parsed.posts,
            ech_enabled: self.ech_enabled,
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

        let mut req = self
            .api_http
            .get(format!("https://{HOST}/tags.json"))
            .query(&query);
        if let Some(creds) = self.auth() {
            req = req.basic_auth(&creds.username, Some(&creds.api_key));
        }

        let resp = req.send().await.context("send tag autocomplete request")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "tag autocomplete failed: HTTP {status} {}",
                trim_body(&body)
            ));
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
            .api_http
            .get(format!("https://{HOST}/favorites.json"))
            .query(&[("limit", "1")])
            .basic_auth(&creds.username, Some(&creds.api_key))
            .send()
            .await
            .context("send credential validation request")?;

        let status = resp.status();
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(anyhow!("invalid username or API key"));
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "credential check failed: HTTP {status} {}",
                trim_body(&body)
            ));
        }
        Ok(())
    }

    pub async fn favorite(&self, post_id: u64) -> Result<()> {
        let creds = self
            .auth()
            .ok_or_else(|| anyhow!("login required to favorite"))?;
        self.wait_for_api_slot().await;

        let resp = self
            .api_http
            .post(format!("https://{HOST}/favorites.json"))
            .query(&[("post_id", post_id.to_string())])
            .basic_auth(&creds.username, Some(&creds.api_key))
            .send()
            .await
            .context("send favorite request")?;

        let status = resp.status();
        if status.is_success() || status.as_u16() == 422 {
            return Ok(());
        }
        let body = resp.text().await.unwrap_or_default();
        Err(anyhow!(
            "favorite failed: HTTP {status} {}",
            trim_body(&body)
        ))
    }

    pub async fn unfavorite(&self, post_id: u64) -> Result<()> {
        let creds = self
            .auth()
            .ok_or_else(|| anyhow!("login required to unfavorite"))?;
        self.wait_for_api_slot().await;

        let resp = self
            .api_http
            .delete(format!("https://{HOST}/favorites/{post_id}.json"))
            .basic_auth(&creds.username, Some(&creds.api_key))
            .send()
            .await
            .context("send unfavorite request")?;

        let status = resp.status();
        if status.is_success() || status.as_u16() == 404 {
            return Ok(());
        }
        let body = resp.text().await.unwrap_or_default();
        Err(anyhow!(
            "unfavorite failed: HTTP {status} {}",
            trim_body(&body)
        ))
    }

    pub async fn download_bytes(&self, url: &str) -> Result<Vec<u8>> {
        Ok(self.download_media(url, None).await?.bytes)
    }

    pub async fn download_media(&self, url: &str, range: Option<&str>) -> Result<MediaResponse> {
        let parsed = reqwest::Url::parse(url).context("parse media URL")?;
        let host = parsed
            .host_str()
            .ok_or_else(|| anyhow!("media URL has no host"))?
            .to_string();

        let http = self.media_client_for(&host).await?;
        let mut req = http.get(parsed);
        if let Some(range) = range {
            req = req.header(RANGE, range);
        }
        if host_accepts_credentials(&host)
            && let Some(creds) = self.auth()
        {
            req = req.basic_auth(&creds.username, Some(&creds.api_key));
        }

        let resp = req.send().await.context("send media request")?;
        let status = resp.status();
        if !status.is_success() && status.as_u16() != 206 {
            return Err(anyhow!("media download failed: HTTP {status}"));
        }
        let headers = resp.headers().clone();
        let bytes = resp.bytes().await.context("read media response")?.to_vec();
        Ok(MediaResponse {
            status: status.as_u16(),
            content_type: header_string(&headers, CONTENT_TYPE),
            content_length: header_string(&headers, CONTENT_LENGTH),
            content_range: header_string(&headers, CONTENT_RANGE),
            accept_ranges: header_string(&headers, ACCEPT_RANGES),
            bytes,
        })
    }

    async fn media_client_for(&self, host: &str) -> Result<reqwest::Client> {
        let mut clients = self.media_clients.lock().await;
        if let Some(client) = clients.get(host) {
            return Ok(client.clone());
        }

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));

        let builder = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(45))
            .connect_timeout(Duration::from_secs(15));
        let configured =
            configure_ech_client(builder, host, self.fail_closed_ech, self.doh_provider).await?;
        let client = configured
            .builder
            .build()
            .with_context(|| format!("build media client for {host}"))?;
        clients.insert(host.to_string(), client.clone());
        Ok(client)
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

impl Client {
    pub async fn update_post_tags(
        &self,
        post_id: u64,
        tag_string_diff: &str,
        edit_reason: &str,
    ) -> Result<Post> {
        let creds = self
            .auth()
            .ok_or_else(|| anyhow!("login required to edit tags"))?;
        let tag_string_diff = tag_string_diff.trim();
        if tag_string_diff.is_empty() {
            return Err(anyhow!("tag changes are required"));
        }

        self.wait_for_api_slot().await;

        let mut query = vec![("post[tag_string_diff]", tag_string_diff.to_string())];
        let edit_reason = edit_reason.trim();
        if !edit_reason.is_empty() {
            query.push(("post[edit_reason]", edit_reason.to_string()));
        }

        let resp = self
            .api_http
            .patch(format!("https://{HOST}/posts/{post_id}.json"))
            .query(&query)
            .basic_auth(&creds.username, Some(&creds.api_key))
            .send()
            .await
            .context("send post tag update request")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "tag update failed: HTTP {status} {}",
                trim_body(&body)
            ));
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
        let mut req = self
            .api_http
            .get(format!("https://{HOST}/comments.json"))
            .query(&[
                ("search[post_id]", post_id.to_string()),
                ("limit", limit.to_string()),
                ("group_by", "comment".to_string()),
            ]);
        if let Some(creds) = self.auth() {
            req = req.basic_auth(&creds.username, Some(&creds.api_key));
        }

        let resp = req.send().await.context("send comments request")?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "comments failed: HTTP {status} {}",
                trim_body(&body)
            ));
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
        let creds = self
            .auth()
            .ok_or_else(|| anyhow!("login required to comment"))?;
        let body = body.trim();
        if body.is_empty() {
            return Err(anyhow!("comment body is required"));
        }

        self.wait_for_api_slot().await;

        let resp = self
            .api_http
            .post(format!("https://{HOST}/comments.json"))
            .query(&[
                ("comment[post_id]", post_id.to_string()),
                ("comment[body]", body.to_string()),
            ])
            .basic_auth(&creds.username, Some(&creds.api_key))
            .send()
            .await
            .context("send create comment request")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "comment failed: HTTP {status} {}",
                trim_body(&body)
            ));
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
        let creds = self
            .auth()
            .ok_or_else(|| anyhow!("login required to hide comments"))?;

        self.wait_for_api_slot().await;

        let resp = self
            .api_http
            .post(format!("https://{HOST}/comments/{comment_id}/hide.json"))
            .basic_auth(&creds.username, Some(&creds.api_key))
            .send()
            .await
            .context("send hide comment request")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "hide comment failed: HTTP {status} {}",
                trim_body(&body)
            ));
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
}

fn header_string(headers: &HeaderMap, name: reqwest::header::HeaderName) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
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
        assert!(host_accepts_credentials("e621.net"));
        assert!(host_accepts_credentials("e926.net"));
    }

    #[test]
    fn host_accepts_credentials_for_subdomains() {
        assert!(host_accepts_credentials("static1.e621.net"));
        assert!(host_accepts_credentials("static2.e621.net"));
        assert!(host_accepts_credentials("static1.e926.net"));
        assert!(host_accepts_credentials("api.e621.net"));
    }

    #[test]
    fn host_accepts_credentials_rejects_other_hosts() {
        assert!(!host_accepts_credentials(""));
        assert!(!host_accepts_credentials("example.com"));
        assert!(!host_accepts_credentials("e621.net.evil.com"));
        assert!(!host_accepts_credentials("notrealle621.net"));
        assert!(!host_accepts_credentials("e621-net"));
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
