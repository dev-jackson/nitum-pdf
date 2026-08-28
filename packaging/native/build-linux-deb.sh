#!/bin/sh
set -eu

version=${1:?indica la versión sin v}
machine=$(uname -m)
case "$machine" in
  x86_64) release_arch=x86_64; deb_arch=amd64 ;;
  aarch64|arm64) release_arch=aarch64; deb_arch=arm64 ;;
  *) echo "Arquitectura Linux no compatible: $machine" >&2; exit 2 ;;
esac

project_dir=$(cd -- "$(dirname -- "$0")/../.." && pwd)
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT HUP INT TERM

cargo build --manifest-path "$project_dir/native/Cargo.toml" --release --locked
"$project_dir/native/scripts/fetch-pdfium.sh" "$project_dir/native/target/release"

install -d "$stage/DEBIAN" "$stage/usr/lib/nitum-pdf" "$stage/usr/bin"
install -d "$stage/usr/share/applications" "$stage/usr/share/icons/hicolor/scalable/apps"
sed -e "s/@VERSION@/$version/g" -e "s/@DEB_ARCH@/$deb_arch/g" \
  "$project_dir/packaging/native/debian-control.in" > "$stage/DEBIAN/control"
install -m 0755 "$project_dir/native/target/release/nitum-pdf" "$stage/usr/lib/nitum-pdf/nitum-pdf"
install -m 0644 "$project_dir/native/target/release/libpdfium.so" "$stage/usr/lib/nitum-pdf/libpdfium.so"
ln -s ../lib/nitum-pdf/nitum-pdf "$stage/usr/bin/nitum-pdf"
install -m 0644 "$project_dir/data/com.nitum.Pdf.desktop" "$stage/usr/share/applications/com.nitum.Pdf.desktop"
install -m 0644 "$project_dir/data/com.nitum.Pdf.svg" "$stage/usr/share/icons/hicolor/scalable/apps/com.nitum.Pdf.svg"

mkdir -p "$project_dir/dist"
dpkg-deb --root-owner-group --build "$stage" "$project_dir/dist/nitum-pdf-$version-linux-$release_arch.deb"
