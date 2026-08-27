#!/bin/sh
set -eu

version=${1:?indica la versión sin v}
release_arch=${2:?indica x86_64 o aarch64}
dist_dir=${3:-dist}

case "$release_arch" in
  x86_64) legacy_arch=amd64 ;;
  aarch64) legacy_arch=arm64 ;;
  *) echo "Arquitectura de release no compatible: $release_arch" >&2; exit 2 ;;
esac

modern="nitum-pdf-$version-linux-$release_arch.deb"
legacy="nitum-pdf_${version}_${legacy_arch}.deb"
[ -f "$dist_dir/$modern" ] || {
  echo "No existe el paquete moderno $dist_dir/$modern" >&2
  exit 2
}

cp "$dist_dir/$modern" "$dist_dir/$legacy"
(
  cd "$dist_dir"
  sha256sum "$modern" > "$modern.sha256"
  sha256sum "$legacy" > "$legacy.sha256"
)
