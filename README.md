# Gmail Desktop

A small Rust/Tauri desktop wrapper for Gmail.

The app opens `https://mail.google.com/` in a native desktop webview and adds a simple native menu with reload, back, forward, and quit actions.

The Icon menu can switch the active window icon between the color, dark, and white Gmail variants.

Links that open in a new tab appear as an in-app peek overlay. The overlay toolbar lets you pop the link out into a full standalone window or close it.

## Requirements

- Node.js 18+
- npm
- Rust and Cargo
- Tauri platform prerequisites for your OS

For macOS, Tauri generally needs Xcode Command Line Tools:

```sh
xcode-select --install
```

Install Rust with:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Run

```sh
npm install
npm run dev
```

## Build

```sh
npm run build
```

The bundled app will be created under `src-tauri/target/release/bundle`.

## Notes

Gmail authentication happens inside the Tauri webview. If Google blocks sign-in because it does not trust the embedded browser on your platform, use Gmail in your system browser instead.

## Icon Attribution

Gmail icon from [selfh.st/icons](https://github.com/selfhst/icons), surfaced by [Dashboard Icons](https://dashboardicons.com/icons/external/gmail), licensed under CC BY 4.0.
