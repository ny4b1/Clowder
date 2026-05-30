#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod credentials;
mod e621;
mod settings;
mod vpn;

use std::fs;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use e621::{Client, Comment, Credentials, MAX_DOWNLOAD_BYTES, Post, SESSION_EXPIRED, Tag};
use serde::Serialize;
use settings::Settings;
use tauri::http::{
    Response, StatusCode,
    header::{
        ACCEPT_RANGES, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_EXPOSE_HEADERS, CONTENT_LENGTH,
        CONTENT_RANGE, CONTENT_TYPE, RANGE,
    },
};
use tauri::{Emitter, Manager, State};
use tokio::sync::Mutex;

const ALLOWED_MEDIA_HOSTS: &[&str] = &[
    "e621.net",
    "static1.e621.net",
    "static2.e621.net",
    "e926.net",
    "static1.e926.net",
    "static2.e926.net",
];

fn validate_remote_url(raw: &str) -> Result<url::Url, String> {
    let parsed = url::Url::parse(raw).map_err(|err| format!("invalid URL: {err}"))?;
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
    "Cloudflare blocked the request",
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

const VPN_TUNNEL_HINT: &str = "Couldn't reach e621 through the VPN tunnel. If a system-wide VPN (like the Mullvad app) is also connected, turn one off — running two VPNs at once breaks the connection.";

fn is_transport_error(chain: &str) -> bool {
    chain.contains("send") && !chain.contains("HTTP")
}

fn finish<T>(
    app: &tauri::AppHandle,
    vpn_live: bool,
    operation: &'static str,
    fallback: &'static str,
    result: anyhow::Result<T>,
) -> Result<T, String> {
    let err = match result {
        Ok(value) => return Ok(value),
        Err(err) => err,
    };
    let chain = format!("{err:#}");
    if chain.contains(SESSION_EXPIRED) {
        if let Err(clear_err) = credentials::clear() {
            tracing::warn!(error = %format!("{clear_err:#}"), "clear credentials on session expiry failed");
        }
        let _ = app.emit("auth-expired", ());
        tracing::info!(operation, "e621 session expired; signed out");
        return Err(SESSION_EXPIRED.to_string());
    }
    if vpn_live && is_transport_error(&chain) {
        tracing::error!(operation, error = %chain, "command failed through vpn tunnel");
        return Err(VPN_TUNNEL_HINT.to_string());
    }
    Err(report(operation, fallback, err))
}

async fn vpn_live(state: &State<'_, Arc<AppState>>) -> bool {
    state.vpn.lock().await.is_some()
}

struct AppState {
    client: Mutex<Option<Client>>,
    settings: RwLock<Settings>,
    vpn: Mutex<Option<vpn::VpnHandle>>,
    mullvad_relays: Mutex<Option<vpn::mullvad::RelayList>>,
}

impl AppState {
    fn new(settings: Settings) -> Self {
        Self {
            client: Mutex::new(None),
            settings: RwLock::new(settings),
            vpn: Mutex::new(None),
            mullvad_relays: Mutex::new(None),
        }
    }
}

#[derive(Serialize)]
struct VpnStatus {
    configured: bool,
    enabled: bool,
    endpoint: Option<String>,
    proxy_url: Option<String>,
    provider: Option<String>,
    account: Option<String>,
    device: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    city: Option<String>,
    city_code: Option<String>,
}

#[derive(Default)]
struct MullvadDisplay {
    provider: Option<String>,
    account: Option<String>,
    device: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    city: Option<String>,
    city_code: Option<String>,
}

#[derive(Serialize)]
struct SearchResponse {
    posts: Vec<Post>,
}

#[derive(Serialize)]
struct AccountResponse {
    username: Option<String>,
}

#[tauri::command]
async fn autocomplete_tags(
    term: String,
    category: Option<u8>,
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<Tag>, String> {
    let client = get_client(&state).await?;
    let live = vpn_live(&state).await;
    finish(
        &app,
        live,
        "autocomplete_tags",
        "Tag autocomplete failed.",
        client.autocomplete_tags(&term, category).await,
    )
}

#[tauri::command]
async fn search_posts(
    tags: String,
    page: u32,
    limit: u32,
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<SearchResponse, String> {
    let client = get_client(&state).await?;
    let live = vpn_live(&state).await;
    let outcome = finish(
        &app,
        live,
        "search_posts",
        "Search failed. Please try again.",
        client.search(&tags, page, limit).await,
    )?;
    Ok(SearchResponse {
        posts: outcome.posts,
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
    let custom_dir = state
        .settings
        .read()
        .expect("settings lock")
        .downloads
        .directory
        .clone();
    let (path, file) = create_unique_download_file(&filename, custom_dir.as_deref())
        .map_err(|err| report("download_file_path", "Could not allocate a file name.", err))?;
    if let Err(err) = client
        .download_to_file(parsed.as_str(), &path, file, MAX_DOWNLOAD_BYTES)
        .await
    {
        let _ = fs::remove_file(&path);
        return Err(report("download_file", "Download failed.", err));
    }
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
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<AccountResponse, String> {
    let username = username.trim().to_string();
    let api_key = api_key.trim().to_string();
    if username.is_empty() || api_key.is_empty() {
        return Err("username and api key are required".to_string());
    }

    let creds = Credentials { username, api_key };
    let client = get_client(&state).await?;
    let live = vpn_live(&state).await;
    finish(
        &app,
        live,
        "sign_in",
        "Sign in failed.",
        client.validate(&creds).await,
    )?;

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
async fn favorite_post(
    post_id: u64,
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    let client = get_client(&state).await?;
    let live = vpn_live(&state).await;
    finish(
        &app,
        live,
        "favorite_post",
        "Favorite failed.",
        client.favorite(post_id).await,
    )?;
    Ok(true)
}

#[tauri::command]
async fn unfavorite_post(
    post_id: u64,
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    let client = get_client(&state).await?;
    let live = vpn_live(&state).await;
    finish(
        &app,
        live,
        "unfavorite_post",
        "Unfavorite failed.",
        client.unfavorite(post_id).await,
    )?;
    Ok(false)
}

#[tauri::command]
async fn fetch_comments(
    post_id: u64,
    limit: u32,
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<Comment>, String> {
    let client = get_client(&state).await?;
    let live = vpn_live(&state).await;
    finish(
        &app,
        live,
        "fetch_comments",
        "Could not load comments.",
        client.comments(post_id, limit).await,
    )
}

#[tauri::command]
async fn create_comment(
    post_id: u64,
    body: String,
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<Comment, String> {
    let client = get_client(&state).await?;
    let live = vpn_live(&state).await;
    finish(
        &app,
        live,
        "create_comment",
        "Failed to post comment.",
        client.create_comment(post_id, &body).await,
    )
}

#[tauri::command]
async fn update_post_tags(
    post_id: u64,
    tag_string_diff: String,
    edit_reason: String,
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<Post, String> {
    let client = get_client(&state).await?;
    let live = vpn_live(&state).await;
    finish(
        &app,
        live,
        "update_post_tags",
        "Failed to update tags.",
        client
            .update_post_tags(post_id, &tag_string_diff, &edit_reason)
            .await,
    )
}

#[tauri::command]
async fn hide_comment(
    comment_id: u64,
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<Comment, String> {
    let client = get_client(&state).await?;
    let live = vpn_live(&state).await;
    finish(
        &app,
        live,
        "hide_comment",
        "Failed to hide comment.",
        client.hide_comment(comment_id).await,
    )
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
    let mut new_settings = new_settings;
    new_settings.normalize();
    settings::save(&app, &new_settings)
        .map_err(|err| report("settings_save", "Failed to save settings.", err))?;
    *state.settings.write().expect("settings lock") = new_settings.clone();
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

#[tauri::command]
async fn import_vpn_config(
    path: String,
    state: State<'_, Arc<AppState>>,
) -> Result<VpnStatus, String> {
    let content = fs::read_to_string(&path)
        .map_err(|err| format!("could not read VPN config file: {err}"))?;
    let cfg = vpn::parse(&content).map_err(|err| report_chain("vpn_parse", err))?;
    vpn::storage::save(&cfg).map_err(|err| report_chain("vpn_save", err))?;
    read_status(&state).await
}

#[tauri::command]
async fn enable_vpn(
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<VpnStatus, String> {
    let cfg = vpn::storage::load()
        .map_err(|err| report_chain("vpn_load", err))?
        .ok_or_else(|| "import a VPN config first".to_string())?;

    swap_tunnel(&state, cfg).await?;
    persist_vpn_enabled(&app, &state, true)?;
    read_status(&state).await
}

async fn swap_tunnel(state: &State<'_, Arc<AppState>>, cfg: vpn::WgConfig) -> Result<(), String> {
    let prev = state.vpn.lock().await.take();
    if let Some(prev) = prev {
        prev.shutdown().await;
    }

    let handle = vpn::VpnHandle::start(cfg)
        .await
        .map_err(|err| report_chain("vpn_start", err))?;
    *state.vpn.lock().await = Some(handle);
    *state.client.lock().await = None;
    Ok(())
}

async fn stop_tunnel(state: &State<'_, Arc<AppState>>) {
    let prev = state.vpn.lock().await.take();
    if let Some(prev) = prev {
        prev.shutdown().await;
    }
    *state.client.lock().await = None;
}

#[tauri::command]
async fn disable_vpn(
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<VpnStatus, String> {
    stop_tunnel(&state).await;
    persist_vpn_enabled(&app, &state, false)?;
    read_status(&state).await
}

#[tauri::command]
async fn clear_vpn_config(
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<VpnStatus, String> {
    stop_tunnel(&state).await;
    deregister_mullvad_device().await;
    vpn::storage::clear_mullvad().map_err(|err| report_chain("vpn_clear", err))?;
    vpn::storage::clear().map_err(|err| report_chain("vpn_clear", err))?;
    *state.mullvad_relays.lock().await = None;
    persist_vpn_enabled(&app, &state, false)?;
    read_status(&state).await
}

#[tauri::command]
async fn get_vpn_status(state: State<'_, Arc<AppState>>) -> Result<VpnStatus, String> {
    read_status(&state).await
}

#[tauri::command]
async fn mullvad_sign_in(
    account: String,
    state: State<'_, Arc<AppState>>,
) -> Result<VpnStatus, String> {
    let account = vpn::mullvad::normalize_account(&account)
        .map_err(|err| report_chain("mullvad_account", err))?;

    let token = vpn::mullvad::fetch_token(&account)
        .await
        .map_err(|err| report_chain("mullvad_token", err))?;

    let existing = vpn::storage::load_mullvad().map_err(|err| report_chain("mullvad_load", err))?;
    let reusable = match existing {
        Some(profile) if profile.account_number == account => {
            match derive_public(&profile.private_key) {
                Some(public_key) => {
                    vpn::mullvad::device_exists(&token, &profile.device_id, &public_key)
                        .await
                        .unwrap_or(false)
                        .then_some(profile)
                }
                None => None,
            }
        }
        _ => None,
    };

    let (private_key, device_id, device_name, addresses) = match reusable {
        Some(profile) => (
            profile.private_key,
            profile.device_id,
            profile.device_name,
            profile.addresses,
        ),
        None => {
            let (private_key, public_key) = vpn::mullvad::generate_keypair();
            let device = vpn::mullvad::register_device(&token, &public_key)
                .await
                .map_err(|err| report_chain("mullvad_register", err))?;
            (private_key, device.id, device.name, device.addresses)
        }
    };

    let relays = vpn::mullvad::fetch_relays()
        .await
        .map_err(|err| report_chain("mullvad_relays", err))?;
    let chosen = relays
        .default_choice()
        .ok_or_else(|| "Mullvad returned no usable servers".to_string())?;

    let profile = vpn::mullvad::MullvadProfile {
        account_number: account,
        private_key,
        device_id,
        device_name,
        addresses,
        country_code: chosen.country_code.clone(),
        country_name: chosen.country_name.clone(),
        city_code: chosen.city_code.clone(),
        city_name: chosen.city_name.clone(),
    };

    let cfg = vpn::mullvad::build_config(&profile, &chosen)
        .map_err(|err| report_chain("mullvad_config", err))?;
    vpn::storage::save(&cfg).map_err(|err| report_chain("mullvad_save", err))?;
    vpn::storage::save_mullvad(&profile).map_err(|err| report_chain("mullvad_save", err))?;
    *state.mullvad_relays.lock().await = Some(relays);

    if state.vpn.lock().await.is_some() {
        swap_tunnel(&state, cfg).await?;
    }

    read_status(&state).await
}

#[tauri::command]
async fn mullvad_locations(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<vpn::mullvad::MullvadCountry>, String> {
    let cached = state.mullvad_relays.lock().await.clone();
    let relays = match cached {
        Some(relays) => relays,
        None => {
            let relays = vpn::mullvad::fetch_relays()
                .await
                .map_err(|err| report_chain("mullvad_relays", err))?;
            *state.mullvad_relays.lock().await = Some(relays.clone());
            relays
        }
    };
    Ok(relays.locations_tree())
}

#[tauri::command]
async fn mullvad_select_relay(
    city_code: String,
    state: State<'_, Arc<AppState>>,
) -> Result<VpnStatus, String> {
    let mut profile = vpn::storage::load_mullvad()
        .map_err(|err| report_chain("mullvad_load", err))?
        .ok_or_else(|| "sign in to Mullvad first".to_string())?;

    let cached = state.mullvad_relays.lock().await.clone();
    let relays = match cached {
        Some(relays) => relays,
        None => {
            let relays = vpn::mullvad::fetch_relays()
                .await
                .map_err(|err| report_chain("mullvad_relays", err))?;
            *state.mullvad_relays.lock().await = Some(relays.clone());
            relays
        }
    };

    let chosen = relays
        .choose(&city_code)
        .ok_or_else(|| "that Mullvad location is no longer available".to_string())?;

    profile.country_code = chosen.country_code.clone();
    profile.country_name = chosen.country_name.clone();
    profile.city_code = chosen.city_code.clone();
    profile.city_name = chosen.city_name.clone();

    let cfg = vpn::mullvad::build_config(&profile, &chosen)
        .map_err(|err| report_chain("mullvad_config", err))?;
    vpn::storage::save(&cfg).map_err(|err| report_chain("mullvad_save", err))?;
    vpn::storage::save_mullvad(&profile).map_err(|err| report_chain("mullvad_save", err))?;

    if state.vpn.lock().await.is_some() {
        swap_tunnel(&state, cfg).await?;
    }

    read_status(&state).await
}

#[tauri::command]
async fn mullvad_sign_out(
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<VpnStatus, String> {
    stop_tunnel(&state).await;
    deregister_mullvad_device().await;
    vpn::storage::clear_mullvad().map_err(|err| report_chain("mullvad_clear", err))?;
    vpn::storage::clear().map_err(|err| report_chain("mullvad_clear", err))?;
    *state.mullvad_relays.lock().await = None;
    persist_vpn_enabled(&app, &state, false)?;
    read_status(&state).await
}

async fn deregister_mullvad_device() {
    let profile = match vpn::storage::load_mullvad() {
        Ok(Some(profile)) => profile,
        _ => return,
    };
    match vpn::mullvad::fetch_token(&profile.account_number).await {
        Ok(token) => {
            if let Err(err) = vpn::mullvad::delete_device(&token, &profile.device_id).await {
                tracing::warn!(error = %format!("{err:#}"), "mullvad device removal failed");
            }
        }
        Err(err) => {
            tracing::warn!(error = %format!("{err:#}"), "mullvad sign-out token failed");
        }
    }
}

fn derive_public(private_key: &str) -> Option<String> {
    use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
    use boringtun::x25519::{PublicKey, StaticSecret};

    let decoded = BASE64.decode(private_key.trim()).ok()?;
    let bytes: [u8; 32] = decoded.try_into().ok()?;
    let secret = StaticSecret::from(bytes);
    let public = PublicKey::from(&secret);
    Some(BASE64.encode(public.to_bytes()))
}

async fn read_status(state: &State<'_, Arc<AppState>>) -> Result<VpnStatus, String> {
    let stored = vpn::storage::load().map_err(|err| report_chain("vpn_status", err))?;
    let mullvad = vpn::storage::load_mullvad().map_err(|err| report_chain("vpn_status", err))?;
    let vpn_guard = state.vpn.lock().await;

    let status = match &mullvad {
        Some(profile) => MullvadDisplay {
            provider: Some("mullvad".to_string()),
            account: Some(vpn::mullvad::mask_account(&profile.account_number)),
            device: Some(profile.device_name.clone()),
            country: Some(profile.country_name.clone()),
            country_code: Some(profile.country_code.clone()),
            city: Some(profile.city_name.clone()),
            city_code: Some(profile.city_code.clone()),
        },
        None if stored.is_some() => MullvadDisplay {
            provider: Some("manual".to_string()),
            ..MullvadDisplay::default()
        },
        None => MullvadDisplay::default(),
    };

    Ok(VpnStatus {
        configured: stored.is_some(),
        enabled: vpn_guard.is_some(),
        endpoint: stored.as_ref().map(|c| c.peer.endpoint.clone()),
        proxy_url: vpn_guard.as_ref().and_then(|h| h.proxy_display_url()),
        provider: status.provider,
        account: status.account,
        device: status.device,
        country: status.country,
        country_code: status.country_code,
        city: status.city,
        city_code: status.city_code,
    })
}

fn persist_vpn_enabled(
    app: &tauri::AppHandle,
    state: &State<'_, Arc<AppState>>,
    enabled: bool,
) -> Result<(), String> {
    let mut guard = state.settings.write().expect("settings lock");
    guard.vpn.enabled = enabled;
    settings::save(app, &guard).map_err(|err| report_chain("vpn_persist", err))?;
    Ok(())
}

fn report_chain(operation: &'static str, err: anyhow::Error) -> String {
    let chain = format!("{err:#}");
    tracing::error!(operation, error = %chain, "vpn command failed");
    chain
}

async fn get_client(state: &State<'_, Arc<AppState>>) -> Result<Client, String> {
    get_client_inner(state.inner()).await
}

async fn get_client_inner(state: &Arc<AppState>) -> Result<Client, String> {
    let proxy_url = {
        let vpn_guard = state.vpn.lock().await;
        vpn_guard.as_ref().and_then(|h| h.proxy_url())
    };
    let mut guard = state.client.lock().await;
    if let Some(client) = guard.as_ref() {
        return Ok(client.clone());
    }

    let client = Client::new(proxy_url.as_deref())
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
    let requested_range = request
        .headers()
        .get(RANGE)
        .and_then(|value| value.to_str().ok())
        .and_then(|range| media_range(&url, range, max_chunk));
    let range = requested_range.or_else(|| initial_media_range(&url, max_chunk));

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

fn initial_media_range(url: &str, max_chunk: u64) -> Option<String> {
    if is_video_url(url) {
        return Some(format!("bytes=0-{}", max_chunk.saturating_sub(1)));
    }
    None
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

fn create_unique_download_file(
    filename: &str,
    custom_dir: Option<&str>,
) -> anyhow::Result<(PathBuf, File)> {
    let downloads = match custom_dir {
        Some(d) if !d.trim().is_empty() => PathBuf::from(d),
        _ => downloads_dir()?,
    };
    fs::create_dir_all(&downloads)?;

    let safe = sanitize_filename(filename);
    let mut path = downloads.join(&safe);
    if let Some(file) = create_new_file(&path)? {
        return Ok((path, file));
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
        if let Some(file) = create_new_file(&path)? {
            return Ok((path, file));
        }
    }

    Err(anyhow::anyhow!(
        "could not allocate a unique download filename"
    ))
}

fn create_new_file(path: &std::path::Path) -> anyhow::Result<Option<File>> {
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(file) => Ok(Some(file)),
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => Ok(None),
        Err(err) => Err(err.into()),
    }
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
    fn create_unique_download_file_uses_create_new() {
        let root = std::env::temp_dir().join(format!(
            "clowder-download-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("sample.jpg"), b"existing").unwrap();

        let (path, file) =
            create_unique_download_file("sample.jpg", Some(root.to_str().unwrap())).unwrap();
        drop(file);

        assert_eq!(
            path.file_name().and_then(|s| s.to_str()),
            Some("sample-1.jpg")
        );
        assert!(path.exists());
        fs::remove_dir_all(root).unwrap();
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
    fn initial_media_range_caps_videos_only() {
        const MB2: u64 = 2 * 1024 * 1024;
        assert_eq!(
            initial_media_range("https://x.example/v.webm", MB2).as_deref(),
            Some("bytes=0-2097151")
        );
        assert_eq!(initial_media_range("https://x.example/i.jpg", MB2), None);
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
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let loaded = settings::load(app.handle());
            let auto_start = loaded.vpn.enabled;
            let app_state = Arc::new(AppState::new(loaded));
            app.manage(app_state.clone());
            if auto_start {
                tauri::async_runtime::spawn(async move {
                    match vpn::storage::load() {
                        Ok(Some(cfg)) => match vpn::VpnHandle::start(cfg).await {
                            Ok(handle) => {
                                *app_state.vpn.lock().await = Some(handle);
                                *app_state.client.lock().await = None;
                                tracing::info!("vpn auto-started");
                            }
                            Err(err) => {
                                tracing::warn!(error = %format!("{err:#}"), "vpn auto-start failed");
                            }
                        },
                        Ok(None) => {
                            tracing::warn!("vpn auto-start skipped: no saved config");
                        }
                        Err(err) => {
                            tracing::warn!(error = %format!("{err:#}"), "vpn auto-start could not load config");
                        }
                    }
                });
            }
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
            set_window_fullscreen,
            import_vpn_config,
            enable_vpn,
            disable_vpn,
            clear_vpn_config,
            get_vpn_status,
            mullvad_sign_in,
            mullvad_locations,
            mullvad_select_relay,
            mullvad_sign_out
        ])
        .run(tauri::generate_context!())
        .expect("run tauri app");
}
