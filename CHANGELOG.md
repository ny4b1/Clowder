# Changelog

All notable changes to Clowder are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project adheres to [Semantic Versioning](https://semver.org/).

## [0.3.0] — 2026-05-03

Security-focused release. Hardens the Tauri permission model, locks down
outbound network targets, eliminates a credential-disclosure path in the
media proxy, and replaces the in-memory base64 preview pipeline with a
streaming proxy. Adds a unit test suite and a GitHub Actions CI pipeline.

### Security

- **Tauri capabilities and CSP defined.** A new `src-tauri/capabilities/main.json`
  scopes permissions to the main window with only `core:default`.
  `withGlobalTauri` is disabled, removing the `window.__TAURI__` global.
  A production CSP and a separate dev CSP are configured in
  `tauri.conf.json` (`script-src 'self'`, `object-src 'none'`,
  `base-uri 'self'`, `frame-ancestors 'none'`, explicit allow-list for
  `clowder-media:` and `data:` on `img-src` / `media-src`).
- **Media URLs restricted to e621/e926 over HTTPS.** A new
  `validate_remote_url` helper enforces both an HTTPS scheme and a host
  whitelist (`e621.net`, `e926.net`, and their `static1` / `static2`
  CDNs). The check is applied at four call sites for defense in depth:
  `media_url`, `fetch_preview` (now removed; see Performance),
  `download_file`, and the `clowder-media://` URI scheme handler.
- **Credential isolation in the media client.** The reqwest client used
  by `download_media` previously attached HTTP Basic Auth to every host.
  A new `host_accepts_credentials` predicate now restricts authentication
  to `e621.net` and `e926.net` (apex and subdomains only). Credentials
  cannot leak to other hosts even if the host whitelist is bypassed.
- **Stronger filename sanitization for downloads.** The sanitizer now
  applies Unicode NFC normalization, replaces visually-confusable slash
  variants (`U+2044`, `U+2215`, `U+29F8`, `U+29F5`, `U+FF0F`,
  `U+FF3C`), prefixes Windows reserved names (`CON`, `PRN`, `AUX`,
  `NUL`, `COM1`–`COM9`, `LPT1`–`LPT9`) with `_`, trims trailing dots
  and spaces, and caps filenames at 200 bytes on a UTF-8 character
  boundary.
- **Internal errors no longer reach the UI verbatim.** A new `report`
  helper logs the full anyhow chain through `tracing::error!` and only
  passes through messages from a curated user-actionable allow-list
  (e.g. *"login required"*, *"invalid username or API key"*). Other
  failures surface as concise per-operation messages such as
  *"Search failed. Please try again."*

### Performance

- **Image previews now stream through `clowder-media://`.** The
  `fetch_preview` Tauri command and its base64 data URL pipeline have
  been removed. Previews are loaded by passing the e621 thumbnail URL
  through `media_url`, which returns a proxy URL the webview consumes
  directly. Peak memory per preview drops from roughly 3× the original
  bytes (source + base64 expansion + IPC copy) to ~1×, and the base64
  encode/decode CPU cost is eliminated.

### Internal

- **Structured logging.** `tracing-subscriber` is now initialized at
  startup with an `env-filter`. Default filter is `clowder=info,warn`;
  override via the `CLOWDER_LOG` environment variable.
- **Unit test suite.** 24 new `#[cfg(test)]` tests cover
  `sanitize_filename`, `capped_video_range`, `validate_remote_url`,
  `is_user_actionable`, `mime_for_url`, `is_video_url`,
  `host_accepts_credentials`, and `trim_body`. Run with
  `cargo test --all-targets`.
- **GitHub Actions CI.** A new `.github/workflows/ci.yml` runs three
  jobs on every push to `main` and on every pull request:
  - **Frontend build** — Vite production build on Node 24.
  - **Rust check** — `cargo fmt --check`, `cargo clippy --all-targets
    -- -D warnings`, `cargo test --all-targets` with cached
    dependencies and Tauri system libraries pre-installed.
  - **Dependency audit** — advisory `npm audit` and `cargo audit`
    runs (non-blocking).
- **Dependabot.** A `.github/dependabot.yml` schedules weekly
  Cargo and npm dependency updates, plus monthly GitHub Actions
  bumps.
- **Build prerequisites bumped to Node 24 LTS** in `README.md`.

### Removed

- **`fetch_preview` Tauri command** and its `PreviewResponse`
  payload. Frontend callers must use `mediaUrl(url)` instead.
  This is an internal IPC change; no external API.

### Migration notes

No user action required. On first launch after upgrading, previously
saved credentials in the OS keychain continue to work. The narrower
host whitelist may surface as failed previews if the e621 CDN serves
posts from an unlisted hostname; report such cases as bugs.

## [0.2.0] — 2026-05-02

Adds e621 comments, tag editing, video controls, and macOS / AppImage
bundle targets.

## [0.1.0] — 2026-05-01

Initial release.
