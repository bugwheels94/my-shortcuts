#!/bin/bash
set -e

VERSION=$1

mkdir -p tmp-assets

mv "mac-out/macos/my-shortcuts.app" "tmp-assets/my-shortcuts-${VERSION}.app"
mv "mac-out/dmg/my-shortcuts_0.0.0_aarch64.dmg" "tmp-assets/my-shortcuts-${VERSION}.dmg"
mv "windows-out/msi/my-shortcuts_0.0.0_x64_en-US.msi" "tmp-assets/my-shortcuts-${VERSION}.msi"
mv "windows-out/nsis/my-shortcuts_0.0.0_x64-setup.exe" "tmp-assets/my-shortcuts-${VERSION}.exe"
mv "ubuntu-out/deb/my-shortcuts_0.0.0_amd64.deb" "tmp-assets/my-shortcuts-${VERSION}.deb"
mv "ubuntu-out/rpm/my-shortcuts-0.0.0-1.x86_64.rpm" "tmp-assets/my-shortcuts-${VERSION}.rpm"
mv "ubuntu-out/appimage/my-shortcuts_0.0.0_amd64.AppImage" "tmp-assets/my-shortcuts-${VERSION}.AppImage"
