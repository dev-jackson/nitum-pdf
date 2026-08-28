#!/bin/sh
set -eu

version=${1:?indica la versión sin v}
machine=$(uname -m)
case "$machine" in
  x86_64) release_arch=x86_64 ;;
  arm64) release_arch=aarch64 ;;
  *) echo "Arquitectura macOS no compatible: $machine" >&2; exit 2 ;;
esac

project_dir=$(cd -- "$(dirname -- "$0")/../.." && pwd)
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT HUP INT TERM
app="$stage/root/Applications/Nitum PDF.app"

cargo build --manifest-path "$project_dir/native/Cargo.toml" --release --locked
"$project_dir/native/scripts/fetch-pdfium.sh" "$project_dir/native/target/release"
install -d "$app/Contents/MacOS" "$app/Contents/Resources"
install -m 0755 "$project_dir/native/target/release/nitum-pdf" "$app/Contents/MacOS/nitum-pdf"
install -m 0644 "$project_dir/native/target/release/libpdfium.dylib" "$app/Contents/Resources/libpdfium.dylib"
sed "s/@VERSION@/$version/g" "$project_dir/packaging/native/Info.plist.in" > "$app/Contents/Info.plist"

iconset="$stage/NitumPDF.iconset"
mkdir -p "$iconset"
# Downscaled from the 1024 px master of the application icon, which is rendered
# from data/com.nitum.Pdf.svg — the same artwork Linux installs. It used to be
# built from the blue family wordmark, so macOS showed an icon unrelated to the
# product. `sips` ships with macOS; rsvg-convert does not, so the master is a
# committed PNG rather than the SVG.
source_icon="$project_dir/data/com.nitum.Pdf.png"
for size in 16 32 128 256 512; do
  sips -z "$size" "$size" "$source_icon" --out "$iconset/icon_${size}x${size}.png" >/dev/null
  double=$((size * 2))
  sips -z "$double" "$double" "$source_icon" --out "$iconset/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$iconset" -o "$app/Contents/Resources/NitumPDF.icns"
xattr -cr "$app" 2>/dev/null || true

if [ -n "${NITUM_APPLE_SIGN_IDENTITY:-}" ]; then
  codesign --force --options runtime --timestamp --sign "$NITUM_APPLE_SIGN_IDENTITY" "$app/Contents/Resources/libpdfium.dylib"
  codesign --force --options runtime --timestamp --sign "$NITUM_APPLE_SIGN_IDENTITY" "$app/Contents/MacOS/nitum-pdf"
  codesign --force --options runtime --timestamp --sign "$NITUM_APPLE_SIGN_IDENTITY" "$app"
  codesign --verify --deep --strict --verbose=2 "$app"
fi

mkdir -p "$project_dir/dist"
package="$project_dir/dist/nitum-pdf-$version-macos-$release_arch.pkg"
unsigned="$stage/nitum-pdf-unsigned.pkg"
COPYFILE_DISABLE=1 pkgbuild --root "$stage/root" \
  --identifier com.nitum.pdf.pkg --version "$version" --install-location / "$unsigned"
if [ -n "${NITUM_APPLE_INSTALLER_IDENTITY:-}" ]; then
  productsign --sign "$NITUM_APPLE_INSTALLER_IDENTITY" --timestamp "$unsigned" "$package"
else
  mv "$unsigned" "$package"
fi
if [ -n "${NITUM_APPLE_NOTARY_PROFILE:-}" ]; then
  xcrun notarytool submit "$package" --keychain-profile "$NITUM_APPLE_NOTARY_PROFILE" --wait
  xcrun stapler staple "$package"
elif [ -n "${NITUM_APPLE_ID:-}" ] || [ -n "${NITUM_APPLE_TEAM_ID:-}" ] || [ -n "${NITUM_APPLE_APP_PASSWORD:-}" ]; then
  if [ -z "${NITUM_APPLE_ID:-}" ] || [ -z "${NITUM_APPLE_TEAM_ID:-}" ] || [ -z "${NITUM_APPLE_APP_PASSWORD:-}" ]; then
    echo "La notarización requiere Apple ID, Team ID y contraseña de aplicación." >&2
    exit 2
  fi
  xcrun notarytool submit "$package" --apple-id "$NITUM_APPLE_ID" \
    --team-id "$NITUM_APPLE_TEAM_ID" --password "$NITUM_APPLE_APP_PASSWORD" --wait
  xcrun stapler staple "$package"
fi
