# Clowder

![Release](https://img.shields.io/github/v/release/nyabi021/Clowder?style=flat&color=6366f1)
![Downloads](https://img.shields.io/github/downloads/nyabi021/Clowder/total?style=flat&color=10b981)
![Last Commit](https://img.shields.io/github/last-commit/nyabi021/Clowder?style=flat&color=f59e0b)
![License](https://img.shields.io/badge/license-GPL--3.0--or--later-8b5cf6?style=flat)

A fast, native desktop viewer for e621.

Clowder is a lightweight cross-platform client built with Tauri and Svelte.
It runs in a small, native window with no web browser overhead, and stores
your credentials securely in the OS keychain.

## Features

- Tag search with live autocomplete
- Responsive post grid with infinite browsing
- Full-resolution viewer for images, GIFs, and videos
- Streamed video playback with seeking and a configurable chunk size
- One-click favorite and unfavorite
- Read, post, and hide comments
- Edit post tags from the viewer
- Downloads with a customizable directory and filename template
- Optional sign-in with your e621 account (username + API key)
- Credentials stored via the system keychain (Keychain on macOS,
  Credential Manager on Windows, Secret Service on Linux)
- DNS-over-HTTPS with selectable provider (Cloudflare, Google, Quad9,
  AdGuard) and an optional fail-closed mode for ECH
- Theme, reduced-motion, and grid-density preferences

## Installation

Pre-built binaries for Windows (NSIS installer), macOS (`.dmg`), and
Linux (AppImage) are published on the
[Releases](https://github.com/nyabi021/Clowder/releases) page.

## Building from source

### Prerequisites

- [Node.js](https://nodejs.org/) 24 LTS or newer
- [Rust](https://rustup.rs/) (stable toolchain)
- Platform dependencies for [Tauri 2](https://v2.tauri.app/start/prerequisites/)

### Build

```sh
npm install
npm run tauri build
```

Bundles are written to `src-tauri/target/release/bundle/`:

- macOS: `.app` and `.dmg` under `bundle/macos/` and `bundle/dmg/`
- Windows: NSIS installer under `bundle/nsis/`
- Linux: AppImage under `bundle/appimage/`

### Run in development mode

```sh
npm install
npm run tauri dev
```

## Usage

1. Launch Clowder.
2. Type tags into the search bar. Suggestions appear as you type.
3. Click a post to open the full-resolution viewer.
4. (Optional) Open Settings to sign in with your e621 username and API
   key. Signing in unlocks favorites, comments, and tag editing.
5. From Settings you can also choose a download directory and filename
   template, pick a DNS-over-HTTPS provider, and adjust theme, motion,
   and grid density.

You can find your API key on your e621 account page under
**Manage API Access**.

## License

Clowder is released under the [GNU General Public License v3.0 or later](LICENSE).

This project is not affiliated with or endorsed by e621.
