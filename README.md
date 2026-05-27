# Clowder

![Release](https://img.shields.io/github/v/release/nyattic/Clowder?style=flat&color=6366f1)
![Downloads](https://img.shields.io/github/downloads/nyattic/Clowder/total?style=flat&color=10b981)
![Last Commit](https://img.shields.io/github/last-commit/nyattic/Clowder?style=flat&color=f59e0b)
![License](https://img.shields.io/badge/license-GPL--3.0-8b5cf6?style=flat)

A fast, native desktop viewer for e621.

Browse in a small, focused window — no web browser, no ads, and your
account credentials stay in your computer's secure password store.

## Features

- Quick tag search with autocomplete and search history
- Full-resolution viewer for images, GIFs, and videos
- Sign in with your e621 account to favorite, comment, and edit tags
- One-click downloads with a customizable filename
- Keyboard shortcuts throughout
- Light and dark themes

## Installation

Download the latest build for your platform from the
[Releases](https://github.com/nyattic/Clowder/releases) page:

- **Windows** — `.exe` installer
- **macOS** — `.dmg`
- **Linux** — `.AppImage`

If e621 is blocked in your country, turn on a VPN before launching.

## Usage

1. Launch Clowder.
2. Type tags into the search bar at the top.
3. Click a post to inspect it; double-click (or press Enter) to open
   it full-size.
4. (Optional) Open Settings and sign in with your e621 username and API
   key to favorite posts, comment, and edit tags.

You can find your API key on your e621 account page under
**Manage API Access**.

## Building from source

Requires [Node.js 24+](https://nodejs.org/), [Rust](https://rustup.rs/),
and the platform dependencies for [Tauri 2](https://v2.tauri.app/start/prerequisites/).

```sh
npm install
npm run tauri build
```

## Releasing (maintainers)

Releases are produced by GitHub Actions when a `v*` tag is pushed.
Auto-updates require a one-time setup:

1. Generate a signing key pair:
   ```sh
   npx tauri signer generate -w ~/.tauri/clowder.key
   ```
2. Copy the printed **public key** into
   `src-tauri/tauri.conf.json` under `plugins.updater.pubkey`.
3. In **GitHub → Settings → Secrets and variables → Actions**, add:
   - `TAURI_SIGNING_PRIVATE_KEY` — contents of `~/.tauri/clowder.key`
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — the passphrase you chose
     (or leave blank if you skipped it)

After that, cutting a release is just:
```sh
git tag v0.3.1 && git push --tags
```
The workflow builds for all three platforms, signs the artifacts,
publishes a GitHub Release, and writes the `latest.json` that
existing installs poll for updates.

> Note: Auto-updates work without OS code-signing, but users will
> still see Gatekeeper / SmartScreen warnings on first install until
> the build is signed by a recognised certificate authority.

## License

[GNU GPL v3.0](LICENSE). Not affiliated with or endorsed by e621.
