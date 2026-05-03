# Changelog

All notable changes to Clowder are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project adheres to [Semantic Versioning](https://semver.org/).

## [0.3.0] — 2026-05-03

### Security

- Defined Tauri capabilities and a Content Security Policy.
  - Added `src-tauri/capabilities/main.json` scoping permissions to the
    main window with only `core:default`.
  - Disabled `withGlobalTauri`, removing the `window.__TAURI__` global
    that exposed every IPC command to any script in the webview.
  - Added a production CSP and a separate dev CSP in `tauri.conf.json`
    (`script-src 'self'`, `object-src 'none'`, `base-uri 'self'`,
    `frame-ancestors 'none'`, with explicit allow-list for
    `clowder-media:` and `data:` on `img-src` / `media-src`).
- Restricted outbound media URLs to e621/e926 over HTTPS only.
  - Added a `validate_remote_url` helper enforcing HTTPS and a host
    whitelist (`e621.net`, `e926.net`, and their `static1` / `static2`
    CDNs).
  - Applied at four call sites (`media_url`, `download_file`, the
    `clowder-media://` URI scheme handler, and the previous
    `fetch_preview` site) for defense in depth.
- Isolated API credentials from non-credential domains.
  - The reqwest client used by `download_media` previously attached
    HTTP Basic Auth to every host. A new `host_accepts_credentials`
    predicate now restricts authentication to `e621.net` and `e926.net`
    (apex and subdomains only).
- Strengthened filename sanitization for downloads.
  - Applied Unicode NFC normalization.
  - Replaced visually-confusable slash variants (`U+2044`, `U+2215`,
    `U+29F8`, `U+29F5`, `U+FF0F`, `U+FF3C`).
  - Prefixed Windows reserved names (`CON`, `PRN`, `AUX`, `NUL`,
    `COM1`–`COM9`, `LPT1`–`LPT9`) with `_`.
  - Trimmed trailing dots and spaces; capped at 200 bytes on a UTF-8
    character boundary.
- Stopped leaking internal error chains to the UI.
  - Added a `report` helper that logs the full anyhow chain via
    `tracing::error!` and only passes through messages from a curated
    user-actionable allow-list (e.g. *"login required"*, *"invalid
    username or API key"*).
  - Other failures now surface as concise per-operation messages such
    as *"Search failed. Please try again."*

### Changed

- Image previews now stream through `clowder-media://` instead of being
  delivered as base64 data URLs.
  - Peak memory per preview drops from roughly 3× the original bytes
    (source + base64 expansion + IPC copy) to ~1×.
  - Eliminates base64 encode/decode CPU cost and improves first-paint
    time.
- Initialized `tracing-subscriber` at startup.
  - Default filter `clowder=info,warn`; override with the `CLOWDER_LOG`
    environment variable.
- Bumped development requirement to Node 24 LTS in `README.md`.

### Added

- 24 unit tests covering `sanitize_filename`, `capped_video_range`,
  `validate_remote_url`, `is_user_actionable`, `mime_for_url`,
  `is_video_url`, `host_accepts_credentials`, and `trim_body`. Run with
  `cargo test --all-targets`.
- GitHub Actions CI in `.github/workflows/ci.yml`:
  - Frontend build (Vite production build on Node 24).
  - Rust check (`cargo fmt --check`, `cargo clippy --all-targets --
    -D warnings`, `cargo test --all-targets`) with cached dependencies
    and Tauri system libraries pre-installed.
  - Advisory `npm audit` and `cargo audit` job (non-blocking).
- Dependabot config in `.github/dependabot.yml` scheduling weekly Cargo
  and npm dependency updates plus monthly GitHub Actions bumps.

### Removed

- `fetch_preview` Tauri command and its `PreviewResponse` payload.
  Frontend callers must use `mediaUrl(url)` instead.

### Validation

- Verified frontend production build with `npm run build`.
- Verified Rust formatting with `cargo fmt --check`.
- Verified Rust lints with `cargo clippy --all-targets -- -D warnings`.
- Verified Rust unit tests with `cargo test --all-targets` (24 passed).

## [0.2.0] — 2026-05-02

### Added

- Added e621 comment support in the original viewer.
  - Fetch and display comments for the current post.
  - Add new comments when signed in.
  - Hide the current user's own comments when supported by e621.
- Added tag editing for posts.
  - Supports e621-style tag string diffs, such as `new_tag -old_tag`.
  - Optional edit reason field is included.
  - Updates the active post after a successful edit.
- Added category-aware tag autocomplete.
  - Prefixes such as `artist:`, `copyright:`, `character:`, and other
    e621 tag categories now autocomplete correctly.
- Added custom video controls.
  - Bottom control bar with play/pause, progress, volume, fullscreen,
    speed menu, and copy URL.
  - Spacebar toggles play/pause while viewing videos.
  - Video playback state is preserved across fullscreen transitions.
- Added app window maximization on launch.

### Changed

- Reworked the original viewer layout to support comments, tag editing,
  and richer media controls.
- Replaced native video controls with a custom control bar closer to
  e621's video UI behavior.
- Split the original viewer into focused components:
  - `VideoControls`
  - `CommentsPanel`
  - `OriginalPostSidebar`

### Fixed

- Fixed fullscreen behavior that could reset video playback.
- Fixed Esc behavior while in video fullscreen so it exits fullscreen
  instead of closing the post.
- Fixed hidden video controls behavior so the control bar hides while
  playing when inactive or when the cursor leaves the video.
- Fixed a blank-screen regression caused by a component naming collision
  during the viewer refactor.
- Fixed textarea styling inheritance for the new comment and tag editing
  inputs.

### Validation

- Verified frontend production build with `npm run build`.
- Verified Tauri/Rust compilation with
  `cargo check --manifest-path src-tauri/Cargo.toml`.
- Verified whitespace and patch hygiene with `git diff --check`.

## [0.1.0] — 2026-05-01

### Added

- Tag search with live autocomplete and metatag suggestions.
- Adaptive post grid that fits page size to the viewport.
- Page navigation with prev/next controls and a page indicator.
- Full-resolution viewer for images, GIFs, and videos.
- Streamed video playback with byte-range seeking.
- Browser-style back and forward navigation, including mouse X1/X2
  buttons.
- One-click favorite and unfavorite.
- One-click download to `~/Downloads/Clowder`.
- Optional e621 sign-in with username and API key.
- Credential storage via the OS keychain (Windows Credential Manager,
  macOS Keychain, Linux Secret Service).
- Encrypted Client Hello (ECH) for API and media requests where
  supported.
- Native Windows NSIS installer.
