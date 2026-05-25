# Clowder

![Release](https://img.shields.io/github/v/release/nyattic/Clowder?style=flat&color=6366f1)
![Downloads](https://img.shields.io/github/downloads/nyattic/Clowder/total?style=flat&color=10b981)
![Last Commit](https://img.shields.io/github/last-commit/nyattic/Clowder?style=flat&color=f59e0b)
![License](https://img.shields.io/badge/license-GPL--3.0-8b5cf6?style=flat)

A fast, native desktop viewer for e621.

Clowder is a lightweight cross-platform client built with Tauri and Svelte.
It runs in a small, native window with no web browser overhead, and stores
your credentials securely in the OS keychain.

## Features

- Tag search with live autocomplete
- Responsive post grid with infinite browsing
- Full-resolution viewer for images, GIFs, and videos
- Streamed video playback with seeking and a configurable chunk size
- Native macOS / Windows / Linux fullscreen for videos
- One-click favorite and unfavorite
- Read, post, and hide comments
- Edit post tags from the viewer
- Downloads with a customizable directory and filename template
- Optional sign-in with your e621 account (username + API key)
- Credentials stored via the system keychain (Keychain on macOS,
  Credential Manager on Windows, Secret Service on Linux)
- Chrome TLS / HTTP/2 fingerprint emulation (via `wreq` + `wreq-util`)
  to pass Cloudflare's automated challenges that block generic HTTP clients
- Theme, reduced-motion, and grid-density preferences

## Network

Clowder makes no attempt at per-app circumvention. If your country or ISP
blocks e621 (e.g. the Korean geo-block via Cloudflare), enable a system-wide
VPN before launching the app — any provider whose exit IP a regular browser
can reach e621 from will work.

Because Clowder impersonates a real Chrome browser at the TLS / HTTP/2 layer,
it does not get caught by Cloudflare's managed challenge on VPN IPs that a
normal browser can pass.

## Installation

Pre-built binaries for Windows (NSIS installer), macOS (`.dmg`), and
Linux (AppImage) are published on the
[Releases](https://github.com/nyattic/Clowder/releases) page.

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

1. Launch Clowder. If you're behind a geo-block, enable your system VPN
   first.
2. Type tags into the search bar. Suggestions appear as you type.
3. Click a post to open the full-resolution viewer.
4. (Optional) Open Settings to sign in with your e621 username and API
   key. Signing in unlocks favorites, comments, and tag editing.
5. From Settings you can also choose a download directory and filename
   template, and adjust theme, motion, and grid density.

You can find your API key on your e621 account page under
**Manage API Access**.

## License

Clowder is released under the [GNU General Public License v3.0](LICENSE).

This project is not affiliated with or endorsed by e621.
