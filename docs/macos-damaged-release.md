# macOS "Damaged App" Release Issue

## Issue

Downloaded macOS GitHub release builds can show this Gatekeeper dialog:

```text
"Gmail Desktop" is damaged and can't be opened. You should move it to the Trash.
```

The same app can still run from a local build because local artifacts are not quarantined the same way downloaded GitHub release assets are.

## Root Cause

The Tauri macOS `.app` inside the release DMG was not fully signed as an app bundle. It only had an ad-hoc/linker signature on the executable.

This command:

```sh
codesign --verify --deep --strict --verbose=4 "/Volumes/Gmail Desktop/Gmail Desktop.app"
```

can fail with:

```text
code has no resources but signature indicates they must be present
```

And this command:

```sh
codesign -dv --verbose=4 "/Volumes/Gmail Desktop/Gmail Desktop.app"
```

can show:

```text
Signature=adhoc
Info.plist=not bound
Sealed Resources=none
```

That means the app bundle signature is incomplete. Gatekeeper can reject the downloaded and quarantined app as damaged.

## Fix

Add macOS ad-hoc bundle signing in `src-tauri/tauri.conf.json`:

```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "-"
    }
  }
}
```

Keep the existing `bundle` fields and add `macOS` under `bundle`.

Example:

```json
"bundle": {
  "active": true,
  "targets": "all",
  "icon": [
    "icons/32x32.png",
    "icons/128x128.png",
    "icons/128x128@2x.png",
    "icons/icon.icns",
    "icons/icon.ico"
  ],
  "category": "Productivity",
  "macOS": {
    "signingIdentity": "-"
  },
  "shortDescription": "Gmail desktop app",
  "longDescription": "A Tauri desktop wrapper for Gmail."
}
```

## Expected Result

After rebuilding, strict verification should pass:

```sh
codesign --verify --deep --strict --verbose=4 "path/to/Gmail Desktop.app"
```

Expected output:

```text
Gmail Desktop.app: valid on disk
Gmail Desktop.app: satisfies its Designated Requirement
```

The detailed signature output should include sealed resources:

```sh
codesign -dv --verbose=4 "path/to/Gmail Desktop.app"
```

Expected signs of the fix:

```text
Identifier=com.madratzz.gmail-desktop
Signature=adhoc
Info.plist entries=...
Sealed Resources version=2
```

## Validate the GitHub Release Artifact

Always verify the actual GitHub release DMG, not only the local app bundle:

```sh
hdiutil attach -readonly -nobrowse "Gmail.Desktop_1.0.0_aarch64.dmg"
codesign --verify --deep --strict --verbose=4 "/Volumes/Gmail Desktop/Gmail Desktop.app"
```

This does not notarize the app. It fixes the malformed/incomplete app-bundle signature that can cause macOS to report the downloaded app as damaged.
