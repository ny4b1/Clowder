# Changelog

All notable changes to Clowder are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project adheres to [Semantic Versioning](https://semver.org/).

## [0.3.0] — 2026-05-03

### Security

- Locked down the Tauri permission model.
  - Defined an explicit capabilities file scoping the main window to default core permissions only.
  - Disabled the global Tauri injection so the webview no longer exposes IPC on `window`.
  - Added a strict Content Security Policy in production with a separate, looser policy for dev.
- Restricted outbound media URLs to e621/e926 over HTTPS only.
  - Thumbnail, preview, full-resolution, and download URLs are now validated against a host whitelist before any network call.
  - Plain `http://` is no longer accepted; only HTTPS.
  - The `clowder-media://` proxy re-validates after token decoding for defense in depth.
- Stopped attaching e621 credentials to non-e621 hosts.
  - HTTP Basic Auth is now scoped to `e621.net` and `e926.net` (apex and subdomains only).
- Strengthened download filename sanitization.
  - Applied Unicode NFC normalization.
  - Replaced visually-confusable slash variants used in spoofed filenames.
  - Prefixed Windows reserved names (`CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`, `LPT1`–`LPT9`) with `_`.
  - Trimmed trailing dots and spaces; capped length at 200 bytes on a UTF-8 boundary.
- Stopped leaking internal error chains to the UI.
  - Internal errors are logged with full context.
  - Toasts now show concise, user-friendly messages such as *"Search failed. Please try again."* or *"Sign in failed."*

### Changed

- Image previews now stream through the in-app media proxy instead of being delivered as base64 data URLs.
  - Peak memory per preview drops from roughly 3× the original bytes to ~1×.
  - Eliminates base64 encode/decode cost and improves first-paint time.
- Initialized structured logging at startup.
  - Default filter shows `info` and `warn` events from Clowder.
  - Override via the `CLOWDER_LOG` environment variable.
- Bumped development requirement to Node 24 LTS.

### Added

- Unit tests for URL validation, filename sanitization, video range capping, MIME guessing, error message classification, and credential host matching.
- GitHub Actions CI.
  - Frontend production build.
  - Rust formatting, lint, and test runs.
  - Advisory `npm audit` and `cargo audit` (non-blocking).
- Dependabot config for weekly Cargo and npm updates and monthly GitHub Actions bumps.

### Removed

- Removed the `fetch_preview` Tauri command. Frontend callers now use the existing `media_url` command.

### Validation

- Verified frontend production build with `npm run build`.
- Verified Rust formatting with `cargo fmt --check`.
- Verified Rust lints with `cargo clippy --all-targets -- -D warnings`.
- Verified Rust unit tests with `cargo test --all-targets`.

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
