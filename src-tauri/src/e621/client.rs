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
use super::types::{Credentials, Post, PostsResponse, Tag, TagsResponse};

const HOST: &str = "e621.net";
const MAX_LIMIT: u32 = 48;
const USER_AGENT_VALUE: &str = concat!(
    "clowder/",
    env!("CARGO_PKG_VERSION"),
    " (desktop viewer; contact: local user)"
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
}

impl Client {
    pub async fn new(fail_closed_ech: bool) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let builder = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(45))
            .connect_timeout(Duration::from_secs(15));

        let configured = configure_ech_client(builder, HOST, fail_closed_ech).await?;
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

    pub async fn search(&self, tags: &str) -> Result<SearchOutcome> {
        self.wait_for_api_slot().await;

        let mut req = self
            .api_http
            .get(format!("https://{HOST}/posts.json"))
            .query(&[("tags", tags.trim()), ("limit", &MAX_LIMIT.to_string())]);
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

    pub async fn autocomplete_tags(&self, term: &str) -> Result<Vec<Tag>> {
        let term = term.trim().trim_start_matches('-').replace(' ', "_");
        if term.len() < 2 {
            return Ok(Vec::new());
        }

        self.wait_for_api_slot().await;

        let name_matches = format!("{term}*");
        let mut req = self
            .api_http
            .get(format!("https://{HOST}/tags.json"))
            .query(&[
                ("search[name_matches]", name_matches.as_str()),
                ("search[order]", "count"),
                ("search[hide_empty]", "true"),
                ("limit", "12"),
            ]);
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
        if let Some(creds) = self.auth() {
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
        let configured = configure_ech_client(builder, host, false).await?;
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
        let min_gap = Duration::from_millis(550);
        if elapsed < min_gap {
            tokio::time::sleep(min_gap - elapsed).await;
        }
        *last = Instant::now();
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
