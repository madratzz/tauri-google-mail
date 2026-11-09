# Gmail Desktop

A small Rust/Tauri desktop wrapper for Gmail.

The app opens `https://mail.google.com/` in a native desktop webview with native Gmail, Edit, View, Window, and Icon menus.

- **Unread count** appears in the window title, the dock/taskbar badge (macOS and supported Linux desktops), and the tray tooltip. A desktop notification is shown when new mail arrives while the window is in the background.
- **System tray**: closing the window hides it to the tray; click the tray icon to toggle it, or use the tray menu to compose or quit.
- **`mailto:` links** open a Gmail compose window, both inside Gmail and from other apps once Gmail Desktop is set as your default mail app. A second launch focuses the running window.
- **Links**: Google links that open in a new tab appear in an in-app peek overlay (press Esc or **Close** to dismiss, **Pop out** for a standalone window). Links to other sites open in your default browser.
- **Zoom** with Ctrl/Cmd `+`, `-`, and `0`.
- **Window size and position** are remembered between launches, as is the Icon menu choice (color, dark, or white).
- **Downloads** (attachments) are saved to your Downloads folder.
- An offline banner appears when the connection drops, and Gmail reloads when it returns.

## Installation

Download the latest release from the [Releases page](https://github.com/madratzz/tauri-google-mail/releases).

### macOS

Download `Gmail.Desktop_x.x.x_aarch64.dmg` (Apple Silicon) or `Gmail.Desktop_x.x.x_x64.dmg` (Intel), open it, and drag the app to your Applications folder.

> If macOS blocks the app on first launch, right-click the app and choose Open.
>
> If macOS says `"Gmail Desktop" is damaged and can't be opened`, the downloaded build was not signed and notarized. Delete that copy and install a signed release. For your own local unsigned test build only, you can remove the quarantine flag after copying it to Applications:
>
> ```sh
> xattr -dr com.apple.quarantine "/Applications/Gmail Desktop.app"
> ```

### Windows

Download `Gmail.Desktop_x.x.x_x64-setup.exe` and run the installer, or download the `.msi` for a standard Windows Installer package.

### Linux (Ubuntu / Debian)

Download the `.deb` package and install it:

```sh
sudo dpkg -i gmail-desktop_x.x.x_amd64.deb
```

### Linux (Arch)

Download the `.pkg.tar.zst` package and install it with pacman:

```sh
sudo pacman -U gmail-desktop-x.x.x-1-x86_64.pkg.tar.zst
```

### Linux (AppImage)

Download the `.AppImage`, make it executable, and run it:

```sh
chmod +x gmail-desktop_x.x.x_amd64.AppImage
./gmail-desktop_x.x.x_amd64.AppImage
```

---

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
