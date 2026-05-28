# Maintainer: madratzz
pkgname=gmail-desktop
pkgver=1.0.0
pkgrel=1
pkgdesc='A Tauri desktop wrapper for Gmail'
arch=('x86_64')
url='https://github.com/madratzz/tauri-google-mail'
license=('MIT')
depends=('webkit2gtk-4.1' 'libayatana-appindicator')
makedepends=('rust' 'nodejs' 'npm' 'base-devel')
source=("${pkgname}-${pkgver}.tar.gz::https://github.com/madratzz/tauri-google-mail/archive/v${pkgver}.tar.gz")
sha256sums=('SKIP')

build() {
  cd "tauri-google-mail-${pkgver}"
  npm ci
  npm run tauri build -- --bundles deb
}

package() {
  cd "tauri-google-mail-${pkgver}"
  install -Dm755 "src-tauri/target/release/gmail-desktop" "$pkgdir/usr/bin/gmail-desktop"
  install -Dm644 "src-tauri/icons/icon.png" "$pkgdir/usr/share/pixmaps/gmail-desktop.png"
  install -Dm644 /dev/stdin "$pkgdir/usr/share/applications/gmail-desktop.desktop" <<EOF
[Desktop Entry]
Name=Gmail Desktop
Exec=gmail-desktop
Icon=gmail-desktop
Type=Application
Categories=Network;Email;
Comment=A Tauri desktop wrapper for Gmail
EOF
}
