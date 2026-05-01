mod credentials;
mod e621;

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use base64::Engine;
use e621::{Client, Credentials, Post, Tag};
use serde::Serialize;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Default)]
struct AppState {
    client: Mutex<Option<Client>>,
}

#[derive(Serialize)]
struct SearchResponse {
    posts: Vec<Post>,
    ech_enabled: bool,
}

#[derive(Serialize)]
struct PreviewResponse {
    data_url: String,
}

#[derive(Serialize)]
struct AccountResponse {
    username: Option<String>,
}

#[tauri::command]
async fn autocomplete_tags(
    term: String,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<Tag>, String> {
    let client = get_client(&state).await?;
    client
        .autocomplete_tags(&term)
        .await
        .map_err(|err| format!("{err:#}"))
}

#[tauri::command]
async fn search_posts(
    tags: String,
    state: State<'_, Arc<AppState>>,
) -> Result<SearchResponse, String> {
    let client = get_client(&state).await?;
    let outcome = client
        .search(&tags)
        .await
        .map_err(|err| format!("{err:#}"))?;
    Ok(SearchResponse {
        posts: outcome.posts,
        ech_enabled: outcome.ech_enabled,
    })
}

#[tauri::command]
async fn fetch_preview(
    url: String,
    state: State<'_, Arc<AppState>>,
) -> Result<PreviewResponse, String> {
    let client = get_client(&state).await?;
    let bytes = client
        .download_bytes(&url)
        .await
        .map_err(|err| format!("{err:#}"))?;
    let mime = mime_for_url(&url);
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(PreviewResponse {
        data_url: format!("data:{mime};base64,{encoded}"),
    })
}

#[tauri::command]
async fn download_file(
    url: String,
    filename: String,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let client = get_client(&state).await?;
    let bytes = client
        .download_bytes(&url)
        .await
        .map_err(|err| format!("{err:#}"))?;
    let path = unique_download_path(&filename).map_err(|err| format!("{err:#}"))?;
    fs::write(&path, bytes).map_err(|err| format!("{err:#}"))?;
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
        .map_err(|err| format!("{err:#}"))?;

    credentials::save(&creds).map_err(|err| format!("{err:#}"))?;
    client.set_credentials(Some(creds.clone()));

    Ok(AccountResponse {
        username: Some(creds.username),
    })
}

#[tauri::command]
async fn sign_out(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    credentials::clear().map_err(|err| format!("{err:#}"))?;
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
        .map_err(|err| format!("{err:#}"))?;
    Ok(true)
}

#[tauri::command]
async fn unfavorite_post(post_id: u64, state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    let client = get_client(&state).await?;
    client
        .unfavorite(post_id)
        .await
        .map_err(|err| format!("{err:#}"))?;
    Ok(false)
}

async fn get_client(state: &State<'_, Arc<AppState>>) -> Result<Client, String> {
    let mut guard = state.client.lock().await;
    if let Some(client) = guard.as_ref() {
        return Ok(client.clone());
    }

    let client = Client::new(false).await.map_err(|err| format!("{err:#}"))?;

    match credentials::load() {
        Ok(Some(creds)) => client.set_credentials(Some(creds)),
        Ok(None) => {}
        Err(err) => tracing::warn!("could not load saved credentials: {err:#}"),
    }

    *guard = Some(client.clone());
    Ok(client)
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

fn unique_download_path(filename: &str) -> anyhow::Result<PathBuf> {
    let downloads = downloads_dir()?;
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

fn sanitize_filename(filename: &str) -> String {
    let mut safe = filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .trim_matches('.')
        .to_string();
    if safe.is_empty() {
        safe = "download".to_string();
    }
    safe
}

fn main() {
    tauri::Builder::default()
        .manage(Arc::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            autocomplete_tags,
            search_posts,
            fetch_preview,
            download_file,
            get_account,
            sign_in,
            sign_out,
            favorite_post,
            unfavorite_post
        ])
        .run(tauri::generate_context!())
        .expect("run tauri app");
}
