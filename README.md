# Clowder

A fast, native desktop viewer for e621.

Clowder is a lightweight cross-platform client built with Tauri and Svelte.
It runs in a small, native window with no web browser overhead, and stores
your credentials securely in the OS keychain.

## Features

- Tag search with live autocomplete
- Responsive post grid with infinite browsing
- Full-resolution viewer for images, GIFs, and videos
- Streamed video playback with seeking support
- One-click favorite and unfavorite
- One-click download to `~/Downloads/Clowder`
- Optional sign-in with your e621 account (username + API key)
- Credentials stored via the system keychain (Keychain on macOS,
  Credential Manager on Windows, Secret Service on Linux)

## Installation

Pre-built binaries are not yet published. Build from source using the
instructions below.

## Building from source

### Prerequisites

- [Node.js](https://nodejs.org/) 20 or newer
- [Rust](https://rustup.rs/) (stable toolchain)
- Platform dependencies for [Tauri 2](https://v2.tauri.app/start/prerequisites/)

### Build

```sh
npm install
npm run tauri build
```

The packaged application will be available under `src-tauri/target/release/bundle/`.

### Run in development mode

```sh
npm install
npm run tauri dev
```

## Usage

1. Launch Clowder.
2. Type tags into the search bar. Suggestions appear as you type.
3. Click a post to open the full-resolution viewer.
4. (Optional) Open the account dialog to sign in with your e621 username
   and API key. Signing in unlocks favorites and personalized content.

You can find your API key on your e621 account page under
**Manage API Access**.

## License

Clowder is released under the [GNU General Public License v3.0 or later](LICENSE).

This project is not affiliated with or endorsed by e621.
