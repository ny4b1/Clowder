#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod credentials;
mod e621;
mod settings;

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use e621::{Client, Comment, Credentials, Post, Tag};
use serde::Serialize;
use settings::Settings;
use tauri::http::{
    Response, StatusCode,
    header::{
        ACCEPT_RANGES, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_EXPOSE_HEADERS, CONTENT_LENGTH,
        CONTENT_RANGE, CONTENT_TYPE, RANGE,
    },
};
use tauri::{Manager, State};
use tokio::sync::Mutex;

const ALLOWED_MEDIA_HOSTS: &[&str] = &[
    "e621.net",
    "static1.e621.net",
    "static2.e621.net",
    "e926.net",
    "static1.e926.net",
    "static2.e926.net",
];

fn validate_remote_url(raw: &str) -> Result<reqwest::Url, String> {
    let parsed = reqwest::Url::parse(raw).map_err(|err| format!("invalid URL: {err}"))?;
    if parsed.scheme() != "https" {
        return Err(format!(
            "only https URLs are allowed (got {})",
            parsed.scheme()
        ));
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| "URL has no host".to_string())?;
    if !ALLOWED_MEDIA_HOSTS.contains(&host) {
        return Err(format!("disallowed media host: {host}"));
    }
    Ok(parsed)
}

const USER_ACTIONABLE_PATTERNS: &[&str] = &[
    "login required",
    "invalid username or API key",
    "username and api key are required",
    "tag changes are required",
    "comment body is required",
    "disallowed media host",
    "only https URLs are allowed",
    "URL has no host",
    "invalid URL",
];

fn is_user_actionable(msg: &str) -> bool {
    USER_ACTIONABLE_PATTERNS.iter().any(|p| msg.contains(p))
}

fn report(operation: &'static str, fallback: &'static str, err: anyhow::Error) -> String {
    let chain = format!("{err:#}");
    tracing::error!(operation, error = %chain, "command failed");
    let top = err.to_string();
    if is_user_actionable(&top) {
        top
    } else {
        fallback.to_string()
    }
}

struct AppState {
    client: Mutex<Option<Client>>,
    settings: RwLock<Settings>,
}

impl AppState {
    fn new(settings: Settings) -> Self {
        Self {
            client: Mutex::new(None),
            settings: RwLock::new(settings),
        }
    }
}

#[derive(Serialize)]
struct SearchResponse {
    posts: Vec<Post>,
    ech_enabled: bool,
}

#[derive(Serialize)]
struct AccountResponse {
    username: Option<String>,
}

#[tauri::command]
async fn autocomplete_tags(
    term: String,
    category: Option<u8>,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<Tag>, String> {
    let client = get_client(&state).await?;
    client
        .autocomplete_tags(&term, category)
        .await
        .map_err(|err| report("autocomplete_tags", "Tag autocomplete failed.", err))
}

#[tauri::command]
async fn search_posts(
    tags: String,
    page: u32,
    limit: u32,
    state: State<'_, Arc<AppState>>,
) -> Result<SearchResponse, String> {
    let client = get_client(&state).await?;
    let outcome = client
        .search(&tags, page, limit)
        .await
        .map_err(|err| report("search_posts", "Search failed. Please try again.", err))?;
    Ok(SearchResponse {
        posts: outcome.posts,
        ech_enabled: outcome.ech_enabled,
    })
}

#[tauri::command]
fn media_url(url: String) -> Result<String, String> {
    let parsed = validate_remote_url(&url)?;
    let token = URL_SAFE_NO_PAD.encode(parsed.as_str().as_bytes());
    #[cfg(target_os = "windows")]
    let out = format!("http://clowder-media.localhost/{token}");
    #[cfg(not(target_os = "windows"))]
    let out = format!("clowder-media://localhost/{token}");
    Ok(out)
}

#[tauri::command]
async fn download_file(
    url: String,
    filename: String,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let parsed = validate_remote_url(&url)?;
    let client = get_client(&state).await?;
    let bytes = client
        .download_bytes(parsed.as_str())
        .await
        .map_err(|err| report("download_file", "Download failed.", err))?;
    let custom_dir = state
        .settings
        .read()
        .expect("settings lock")
        .downloads
        .directory
        .clone();
    let path = unique_download_path(&filename, custom_dir.as_deref())
        .map_err(|err| report("download_file_path", "Could not allocate a file name.", err))?;
    fs::write(&path, bytes).map_err(|err| {
        let chain = format!("{err:#}");
        tracing::error!(operation = "download_file_write", error = %chain);
        "Could not save the file.".to_string()
    })?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
async fn get_account(state: State<'_, Arc<AppState>>) -> Result<AccountResponse, String> {
    let client = get_client(&state).await?;
    Ok(AccountResponse {
        username: client.current_username(),
    })
}

#[tauri::command]
async fn sign_in(
    username: String,
    api_key: String,
    state: State<'_, Arc<AppState>>,
) -> Result<AccountResponse, String> {
    let username = username.trim().to_string();
    let api_key = api_key.trim().to_string();
    if username.is_empty() || api_key.is_empty() {
        return Err("username and api key are required".to_string());
    }

    let creds = Credentials { username, api_key };
    let client = get_client(&state).await?;
    client
        .validate(&creds)
        .await
        .map_err(|err| report("sign_in", "Sign in failed.", err))?;

    credentials::save(&creds)
        .map_err(|err| report("credentials_save", "Could not save credentials.", err))?;
    client.set_credentials(Some(creds.clone()));

    Ok(AccountResponse {
        username: Some(creds.username),
    })
}

#[tauri::command]
async fn sign_out(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    credentials::clear()
        .map_err(|err| report("credentials_clear", "Could not clear credentials.", err))?;
    let client = get_client(&state).await?;
    client.set_credentials(None);
    Ok(())
}

#[tauri::command]
async fn favorite_post(post_id: u64, state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    let client = get_client(&state).await?;
    client
        .favorite(post_id)
        .await
        .map_err(|err| report("favorite_post", "Favorite failed.", err))?;
    Ok(true)
}

#[tauri::command]
async fn unfavorite_post(post_id: u64, state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    let client = get_client(&state).await?;
    client
        .unfavorite(post_id)
        .await
        .map_err(|err| report("unfavorite_post", "Unfavorite failed.", err))?;
    Ok(false)
}

#[tauri::command]
async fn fetch_comments(
    post_id: u64,
    limit: u32,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<Comment>, String> {
    let client = get_client(&state).await?;
    client
        .comments(post_id, limit)
        .await
        .map_err(|err| report("fetch_comments", "Could not load comments.", err))
}

#[tauri::command]
async fn create_comment(
    post_id: u64,
    body: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Comment, String> {
    let client = get_client(&state).await?;
    client
        .create_comment(post_id, &body)
        .await
        .map_err(|err| report("create_comment", "Failed to post comment.", err))
}

#[tauri::command]
async fn update_post_tags(
    post_id: u64,
    tag_string_diff: String,
    edit_reason: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Post, String> {
    let client = get_client(&state).await?;
    client
        .update_post_tags(post_id, &tag_string_diff, &edit_reason)
        .await
        .map_err(|err| report("update_post_tags", "Failed to update tags.", err))
}

#[tauri::command]
async fn hide_comment(comment_id: u64, state: State<'_, Arc<AppState>>) -> Result<Comment, String> {
    let client = get_client(&state).await?;
    client
        .hide_comment(comment_id)
        .await
        .map_err(|err| report("hide_comment", "Failed to hide comment.", err))
}

#[tauri::command]
fn get_settings(state: State<'_, Arc<AppState>>) -> Result<Settings, String> {
    Ok(state.settings.read().expect("settings lock").clone())
}

#[tauri::command]
async fn update_settings(
    new_settings: Settings,
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<Settings, String> {
    settings::save(&app, &new_settings)
        .map_err(|err| report("settings_save", "Failed to save settings.", err))?;

    let invalidate_client = {
        let mut guard = state.settings.write().expect("settings lock");
        let changed = guard.doh_provider != new_settings.doh_provider
            || guard.fail_closed_ech != new_settings.fail_closed_ech;
        *guard = new_settings.clone();
        changed
    };

    if invalidate_client {
        *state.client.lock().await = None;
    }

    Ok(new_settings)
}

#[tauri::command]
fn set_window_fullscreen(window: tauri::Window, fullscreen: bool) -> Result<(), String> {
    window.set_fullscreen(fullscreen).map_err(|err| {
        let chain = format!("{err:#}");
        tracing::error!(operation = "set_window_fullscreen", error = %chain);
        "Could not toggle fullscreen.".to_string()
    })
}

async fn get_client(state: &State<'_, Arc<AppState>>) -> Result<Client, String> {
    get_client_inner(state.inner()).await
}

async fn get_client_inner(state: &Arc<AppState>) -> Result<Client, String> {
    let mut guard = state.client.lock().await;
    if let Some(client) = guard.as_ref() {
        return Ok(client.clone());
    }

    let (doh_provider, fail_closed_ech) = {
        let s = state.settings.read().expect("settings lock");
        (s.doh_provider, s.fail_closed_ech)
    };

    let client = Client::new(doh_provider, fail_closed_ech)
        .await
        .map_err(|err| report("client_init", "Could not initialize HTTP client.", err))?;

    match credentials::load() {
        Ok(Some(creds)) => client.set_credentials(Some(creds)),
        Ok(None) => {}
        Err(err) => tracing::warn!(error = %format!("{err:#}"), "could not load saved credentials"),
    }

    *guard = Some(client.clone());
    Ok(client)
}

async fn serve_media_request(
    request: tauri::http::Request<Vec<u8>>,
    state: Arc<AppState>,
) -> Response<Vec<u8>> {
    match fetch_media_response(request, state).await {
        Ok(response) => response,
        Err(error) => text_response(StatusCode::BAD_GATEWAY, format!("{error:#}")),
    }
}

async fn fetch_media_response(
    request: tauri::http::Request<Vec<u8>>,
    state: Arc<AppState>,
) -> anyhow::Result<Response<Vec<u8>>> {
    let token = request.uri().path().trim_start_matches('/');
    let decoded = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|err| anyhow::anyhow!("decode media token: {err}"))?;
    let url =
        String::from_utf8(decoded).map_err(|err| anyhow::anyhow!("decode media URL: {err}"))?;
    let parsed = validate_remote_url(&url).map_err(|err| anyhow::anyhow!(err))?;
    let url = parsed.as_str().to_string();
    let max_chunk = state
        .settings
        .read()
        .expect("settings lock")
        .playback
        .video_chunk_bytes();
    let range = request
        .headers()
        .get(RANGE)
        .and_then(|value| value.to_str().ok())
        .and_then(|range| media_range(&url, range, max_chunk));

    let client = get_client_inner(&state)
        .await
        .map_err(|err| anyhow::anyhow!(err))?;
    let media = client
        .download_media(&url, range.as_deref())
        .await
        .map_err(|err| anyhow::anyhow!("fetch media: {err:#}"))?;

    let mut builder = Response::builder()
        .status(StatusCode::from_u16(media.status).unwrap_or(StatusCode::OK))
        .header(ACCEPT_RANGES, "bytes")
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(
            ACCESS_CONTROL_EXPOSE_HEADERS,
            "content-range, content-length, accept-ranges",
        );
    if let Some(content_type) = media.content_type {
        builder = builder.header(CONTENT_TYPE, content_type);
    } else {
        builder = builder.header(CONTENT_TYPE, mime_for_url(&url));
    }
    if let Some(content_length) = media.content_length {
        builder = builder.header(CONTENT_LENGTH, content_length);
    }
    if let Some(content_range) = media.content_range {
        builder = builder.header(CONTENT_RANGE, content_range);
    }
    if let Some(accept_ranges) = media.accept_ranges {
        builder = builder.header(ACCEPT_RANGES, accept_ranges);
    }

    builder
        .body(media.bytes)
        .map_err(|err| anyhow::anyhow!("build media response: {err}"))
}

fn media_range(url: &str, range: &str, max_chunk: u64) -> Option<String> {
    if is_video_url(url) {
        return capped_video_range(range, max_chunk);
    }
    Some(range.trim().to_string())
}

fn capped_video_range(range: &str, max_chunk: u64) -> Option<String> {
    let range = range.trim();
    let value = range.strip_prefix("bytes=")?;
    let first = value.split(',').next()?.trim();
    let (start, end) = first.split_once('-')?;
    if start.is_empty() {
        return Some(format!("bytes=-{max_chunk}"));
    }
    let start = start.parse::<u64>().ok()?;
    let requested_end = end.parse::<u64>().ok();
    let capped_end = start.saturating_add(max_chunk.saturating_sub(1));
    let end = requested_end.map_or(capped_end, |end| end.min(capped_end));
    Some(format!("bytes={start}-{end}"))
}

fn is_video_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.ends_with(".webm") || lower.ends_with(".mp4")
}

fn text_response(status: StatusCode, body: String) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(body.into_bytes())
        .expect("build text response")
}

fn mime_for_url(url: &str) -> &'static str {
    let lower = url.to_ascii_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".webm") {
        "video/webm"
    } else if lower.ends_with(".mp4") {
        "video/mp4"
    } else {
        "image/jpeg"
    }
}

fn unique_download_path(filename: &str, custom_dir: Option<&str>) -> anyhow::Result<PathBuf> {
    let downloads = match custom_dir {
        Some(d) if !d.trim().is_empty() => PathBuf::from(d),
        _ => downloads_dir()?,
    };
    fs::create_dir_all(&downloads)?;

    let safe = sanitize_filename(filename);
    let mut path = downloads.join(&safe);
    if !path.exists() {
        return Ok(path);
    }

    let stem = PathBuf::from(&safe)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("download")
        .to_string();
    let ext = PathBuf::from(&safe)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    for index in 1..1000 {
        let candidate = match &ext {
            Some(ext) => format!("{stem}-{index}.{ext}"),
            None => format!("{stem}-{index}"),
        };
        path = downloads.join(candidate);
        if !path.exists() {
            return Ok(path);
        }
    }

    Err(anyhow::anyhow!(
        "could not allocate a unique download filename"
    ))
}

fn downloads_dir() -> anyhow::Result<PathBuf> {
    if let Some(home) = std::env::var_os("HOME") {
        return Ok(PathBuf::from(home).join("Downloads").join("Clowder"));
    }
    if let Some(profile) = std::env::var_os("USERPROFILE") {
        return Ok(PathBuf::from(profile).join("Downloads").join("Clowder"));
    }
    Ok(std::env::current_dir()?.join("downloads").join("Clowder"))
}

const RESERVED_FILENAME_STEMS: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

const MAX_FILENAME_BYTES: usize = 200;

fn sanitize_filename(filename: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    let mut safe: String = filename
        .nfc()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            '\u{2044}' | '\u{2215}' | '\u{29F8}' | '\u{29F5}' | '\u{FF0F}' | '\u{FF3C}' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    safe = safe.trim().trim_matches(['.', ' ']).to_string();
    if safe.is_empty() {
        safe = "download".to_string();
    }
    let stem_upper = safe.split('.').next().unwrap_or("").to_ascii_uppercase();
    if RESERVED_FILENAME_STEMS.contains(&stem_upper.as_str()) {
        safe.insert(0, '_');
    }
    if safe.len() > MAX_FILENAME_BYTES {
        let mut cut = MAX_FILENAME_BYTES;
        while cut > 0 && !safe.is_char_boundary(cut) {
            cut -= 1;
        }
        safe.truncate(cut);
        safe = safe.trim_end_matches(['.', ' ']).to_string();
        if safe.is_empty() {
            safe = "download".to_string();
        }
    }
    safe
}

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};
    let filter = EnvFilter::try_from_env("CLOWDER_LOG")
        .unwrap_or_else(|_| EnvFilter::new("clowder=info,warn"));
    let _ = fmt().with_env_filter(filter).with_target(true).try_init();
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_filename_strips_separators() {
        assert_eq!(
            sanitize_filename("a/b\\c:d*e?f\"g<h>i|j"),
            "a_b_c_d_e_f_g_h_i_j"
        );
    }

    #[test]
    fn sanitize_filename_handles_empty_and_dots() {
        assert_eq!(sanitize_filename(""), "download");
        assert_eq!(sanitize_filename("   "), "download");
        assert_eq!(sanitize_filename("..."), "download");
        assert_eq!(sanitize_filename("../etc/passwd"), "_etc_passwd");
    }

    #[test]
    fn sanitize_filename_replaces_control_chars() {
        assert_eq!(sanitize_filename("foo\nbar\tbaz"), "foo_bar_baz");
    }

    #[test]
    fn sanitize_filename_replaces_unicode_slashes() {
        assert_eq!(sanitize_filename("a\u{2044}b\u{2215}c"), "a_b_c");
        assert_eq!(sanitize_filename("a\u{FF0F}b\u{FF3C}c"), "a_b_c");
        assert_eq!(sanitize_filename("a\u{29F8}b\u{29F5}c"), "a_b_c");
    }

    #[test]
    fn sanitize_filename_blocks_windows_reserved_names() {
        assert_eq!(sanitize_filename("CON"), "_CON");
        assert_eq!(sanitize_filename("con.txt"), "_con.txt");
        assert_eq!(sanitize_filename("PRN.png"), "_PRN.png");
        assert_eq!(sanitize_filename("LPT1"), "_LPT1");
        assert_eq!(sanitize_filename("NUL.jpg"), "_NUL.jpg");
        assert_eq!(sanitize_filename("notreserved.txt"), "notreserved.txt");
    }

    #[test]
    fn sanitize_filename_caps_length_at_char_boundary() {
        let long_ascii = "a".repeat(300);
        let safe = sanitize_filename(&long_ascii);
        assert!(safe.len() <= MAX_FILENAME_BYTES);
        assert_eq!(safe.len(), MAX_FILENAME_BYTES);

        let long_multibyte = "한".repeat(100);
        let safe = sanitize_filename(&long_multibyte);
        assert!(safe.len() <= MAX_FILENAME_BYTES);
        assert!(safe.is_char_boundary(safe.len()));
    }

    #[test]
    fn sanitize_filename_trims_trailing_space_and_dot() {
        assert_eq!(sanitize_filename("file.  "), "file");
        assert_eq!(sanitize_filename("file.   . . "), "file");
    }

    #[test]
    fn sanitize_filename_normalises_to_nfc() {
        let decomposed = "\u{1100}\u{1161}";
        let composed = "\u{AC00}";
        assert_eq!(sanitize_filename(decomposed), composed);
    }

    #[test]
    fn capped_video_range_caps_to_configured_chunk() {
        const MB2: u64 = 2 * 1024 * 1024;
        assert_eq!(
            capped_video_range("bytes=0-", MB2).as_deref(),
            Some("bytes=0-2097151")
        );
        assert_eq!(
            capped_video_range("bytes=0-9999999", MB2).as_deref(),
            Some("bytes=0-2097151")
        );
        assert_eq!(
            capped_video_range("bytes=100-200", MB2).as_deref(),
            Some("bytes=100-200")
        );
        assert_eq!(
            capped_video_range("bytes=-500", MB2).as_deref(),
            Some("bytes=-2097152")
        );

        const MB8: u64 = 8 * 1024 * 1024;
        assert_eq!(
            capped_video_range("bytes=0-", MB8).as_deref(),
            Some("bytes=0-8388607")
        );
    }

    #[test]
    fn capped_video_range_handles_overflow() {
        const MB2: u64 = 2 * 1024 * 1024;
        let near_max = u64::MAX - 100;
        let result = capped_video_range(&format!("bytes={near_max}-"), MB2).unwrap();
        assert_eq!(result, format!("bytes={near_max}-{}", u64::MAX));
    }

    #[test]
    fn capped_video_range_rejects_invalid_input() {
        const MB2: u64 = 2 * 1024 * 1024;
        assert!(capped_video_range("bogus", MB2).is_none());
        assert!(capped_video_range("bytes=", MB2).is_none());
        assert!(capped_video_range("bytes=abc-def", MB2).is_none());
    }

    #[test]
    fn is_video_url_matches_extensions() {
        assert!(is_video_url("https://x.example/v.webm"));
        assert!(is_video_url("https://x.example/v.MP4"));
        assert!(!is_video_url("https://x.example/image.png"));
        assert!(!is_video_url("https://x.example/no_ext"));
    }

    #[test]
    fn mime_for_url_returns_known_types() {
        assert_eq!(mime_for_url("a.png"), "image/png");
        assert_eq!(mime_for_url("a.gif"), "image/gif");
        assert_eq!(mime_for_url("a.webp"), "image/webp");
        assert_eq!(mime_for_url("a.webm"), "video/webm");
        assert_eq!(mime_for_url("a.mp4"), "video/mp4");
        assert_eq!(mime_for_url("a.unknown"), "image/jpeg");
    }

    #[test]
    fn validate_remote_url_accepts_allowed_hosts() {
        assert!(validate_remote_url("https://static1.e621.net/posts/1.jpg").is_ok());
        assert!(validate_remote_url("https://e621.net/posts.json").is_ok());
        assert!(validate_remote_url("https://static2.e926.net/x.png").is_ok());
        assert!(validate_remote_url("https://e926.net/").is_ok());
    }

    #[test]
    fn validate_remote_url_rejects_other_hosts() {
        assert!(validate_remote_url("https://example.com/x").is_err());
        assert!(validate_remote_url("https://10.0.0.1/x").is_err());
        assert!(validate_remote_url("https://e621.net.evil.com/x").is_err());
        assert!(validate_remote_url("https://notreally.com/e621.net/x").is_err());
    }

    #[test]
    fn validate_remote_url_rejects_non_https_schemes() {
        assert!(validate_remote_url("http://e621.net/x").is_err());
        assert!(validate_remote_url("file:///etc/passwd").is_err());
        assert!(validate_remote_url("ftp://e621.net/x").is_err());
    }

    #[test]
    fn validate_remote_url_rejects_invalid_input() {
        assert!(validate_remote_url("").is_err());
        assert!(validate_remote_url("not a url").is_err());
        assert!(validate_remote_url("https://").is_err());
    }

    #[test]
    fn is_user_actionable_recognises_known_messages() {
        assert!(is_user_actionable("login required to favorite"));
        assert!(is_user_actionable("invalid username or API key"));
        assert!(is_user_actionable("comment body is required"));
        assert!(is_user_actionable("disallowed media host: 10.0.0.1"));
        assert!(is_user_actionable("only https URLs are allowed (got http)"));
    }

    #[test]
    fn is_user_actionable_rejects_internal_messages() {
        assert!(!is_user_actionable(
            "error sending request: connection refused"
        ));
        assert!(!is_user_actionable("decode posts response"));
        assert!(!is_user_actionable("HTTP 500 internal server error"));
    }
}

fn main() {
    init_tracing();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let loaded = settings::load(app.handle());
            app.manage(Arc::new(AppState::new(loaded)));
            Ok(())
        })
        .register_asynchronous_uri_scheme_protocol("clowder-media", |ctx, request, responder| {
            let state = ctx.app_handle().state::<Arc<AppState>>().inner().clone();
            tauri::async_runtime::spawn(async move {
                responder.respond(serve_media_request(request, state).await);
            });
        })
        .invoke_handler(tauri::generate_handler![
            autocomplete_tags,
            search_posts,
            media_url,
            download_file,
            get_account,
            sign_in,
            sign_out,
            favorite_post,
            unfavorite_post,
            fetch_comments,
            create_comment,
            update_post_tags,
            hide_comment,
            get_settings,
            update_settings,
            set_window_fullscreen
        ])
        .run(tauri::generate_context!())
        .expect("run tauri app");
}
