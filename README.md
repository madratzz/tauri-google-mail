# Gmail Desktop

A small Rust/Tauri desktop wrapper for Gmail.

The app opens `https://mail.google.com/` in a native desktop webview and adds a simple native menu with reload, back, forward, and quit actions.

The Icon menu can switch the active window icon between the color, dark, and white Gmail variants.

Links that open in a new tab appear as an in-app peek overlay. The overlay toolbar lets you pop the link out into a full standalone window or close it.

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

## Release Signing

macOS release DMGs must be signed and notarized. Add these GitHub Actions secrets before publishing a release:

- `APPLE_CERTIFICATE`: base64-encoded Developer ID Application certificate exported as a `.p12`
- `APPLE_CERTIFICATE_PASSWORD`: password for the exported `.p12`
- `APPLE_SIGNING_IDENTITY`: Developer ID Application signing identity
- `APPLE_ID`: Apple ID used for notarization
- `APPLE_PASSWORD`: app-specific password for the Apple ID
- `APPLE_TEAM_ID`: Apple Developer Team ID

The release workflow builds separate Intel and Apple Silicon macOS artifacts and fails early if the signing secrets are missing.

## Notes

Gmail authentication happens inside the Tauri webview. If Google blocks sign-in because it does not trust the embedded browser on your platform, use Gmail in your system browser instead.

## Icon Attribution

Gmail icon from [selfh.st/icons](https://github.com/selfhst/icons), surfaced by [Dashboard Icons](https://dashboardicons.com/icons/external/gmail), licensed under CC BY 4.0.
