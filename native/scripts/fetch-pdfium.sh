#!/bin/sh
set -eu

version=7881
os=$(uname -s)
cpu=$(uname -m)

case "$os:$cpu" in
  Darwin:arm64) asset=pdfium-mac-arm64.tgz; expected=52e94ca5aa8847934330daf3f8150c190682c5ca93831468794f8b90d4392e40; library=lib/libpdfium.dylib ;;
  Darwin:x86_64) asset=pdfium-mac-x64.tgz; expected=6dedf83990e0e3d6b7c93c9e7589c5a126b0ae14b7464d76120cff7a26afb18b; library=lib/libpdfium.dylib ;;
  Linux:aarch64) asset=pdfium-linux-arm64.tgz; expected=ee7f7b7d5468958336a818c1cd580bdd20972846b7377b13f9a923d92d1d4674; library=lib/libpdfium.so ;;
  Linux:x86_64) asset=pdfium-linux-x64.tgz; expected=1470e21b8b4a3b4ad7f85684e2da11d94f3b69a86d81dee11b9b6709d927ac1d; library=lib/libpdfium.so ;;
  MINGW*:aarch64|MSYS*:aarch64) asset=pdfium-win-arm64.tgz; expected=d3035d4d2cacac6ecd1a2ece197a3d702a1b2a58466276b9f870b8cb278a9d84; library=bin/pdfium.dll ;;
  MINGW*:x86_64|MSYS*:x86_64) asset=pdfium-win-x64.tgz; expected=73cc0de638ac2095e7445bf56a38200a5b7c7ca0e9f4ba144598f2457377ac08; library=bin/pdfium.dll ;;
  *) echo "Plataforma PDFium no compatible: $os $cpu" >&2; exit 2 ;;
esac

destination=${1:-native/target/debug}
temporary=$(mktemp -d)
trap 'rm -rf "$temporary"' EXIT HUP INT TERM
archive="$temporary/$asset"
url="https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F$version/$asset"

curl --fail --location --silent --show-error "$url" --output "$archive"
if command -v sha256sum >/dev/null 2>&1; then
  actual=$(sha256sum "$archive" | awk '{print $1}')
elif command -v shasum >/dev/null 2>&1; then
  actual=$(shasum -a 256 "$archive" | awk '{print $1}')
else
  echo "No se encontró una herramienta SHA-256 compatible." >&2
  exit 2
fi
if [ "$actual" != "$expected" ]; then
  echo "PDFium no superó la comprobación SHA-256." >&2
  exit 3
fi

tar -xzf "$archive" -C "$temporary"
mkdir -p "$destination"
cp "$temporary/$library" "$destination/"
echo "PDFium $version instalado en $destination"
