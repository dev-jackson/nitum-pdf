#!/usr/bin/env bash
# Build a .deb for Zorin OS / Ubuntu / any Debian derivative.
#
#   sudo apt install python3-venv python3-gi gir1.2-gtk-4.0 gir1.2-adw-1
#   ./packaging/build-deb.sh            -> dist/nitum-pdf_0.2.0_amd64.deb
#
# The package carries its own virtualenv under /opt/nitum-pdf with the Python
# dependencies that Ubuntu does not ship (pypdfium2, pyHanko). PyGObject and GTK
# come from the distro, so the app looks native and follows the system theme.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
version="$(sed -n 's/^version = "\(.*\)"/\1/p' "$here/pyproject.toml" | head -1)"
arch="$(dpkg --print-architecture)"
stage="$here/build/deb"
outdir="$here/dist"
prefix="/opt/nitum-pdf"

echo "==> Nitum PDF $version ($arch)"
rm -rf "$stage"
mkdir -p "$stage/DEBIAN" "$stage$prefix" "$stage/usr/bin" \
         "$stage/usr/share/applications" \
         "$stage/usr/share/icons/hicolor/scalable/apps" \
         "$stage/usr/share/doc/nitum-pdf" "$outdir"

echo "==> creating the bundled virtualenv"
python3 -m venv --system-site-packages "$stage$prefix/venv"
"$stage$prefix/venv/bin/pip" install --quiet --upgrade pip wheel
"$stage$prefix/venv/bin/pip" install --quiet "$here"

# The venv is relocated on install, so nothing may depend on the staging path.
sed -i "s|$stage$prefix|$prefix|g" "$stage$prefix/venv/pyvenv.cfg" || true
rm -rf "$stage$prefix/venv/bin/pip"* "$stage$prefix/venv/bin/wheel" \
       "$stage$prefix/venv/bin/activate"*
find "$stage$prefix/venv" -name '__pycache__' -type d -prune -exec rm -rf {} +

cat > "$stage/usr/bin/nitum-pdf" <<'LAUNCHER'
#!/bin/sh
# Calling the interpreter directly keeps the venv relocatable.
exec /opt/nitum-pdf/venv/bin/python -m pwviewpdf "$@"
LAUNCHER
chmod 755 "$stage/usr/bin/nitum-pdf"
ln -s nitum-pdf "$stage/usr/bin/pw-view-pdf"

install -m 644 "$here/data/org.pwview.PdfViewer.desktop" \
    "$stage/usr/share/applications/"
install -m 644 "$here/data/org.pwview.PdfViewer.svg" \
    "$stage/usr/share/icons/hicolor/scalable/apps/"
install -m 644 "$here/README.md" "$stage/usr/share/doc/nitum-pdf/"

installed_kb="$(du -sk "$stage" | cut -f1)"
cat > "$stage/DEBIAN/control" <<CONTROL
Package: nitum-pdf
Version: $version
Section: utils
Priority: optional
Architecture: $arch
Depends: python3 (>= 3.10), python3-gi, gir1.2-gtk-4.0, gir1.2-adw-1, libpcsclite1, tzdata
Recommends: opensc-pkcs11, p11-kit-modules, pcscd
Installed-Size: $installed_kb
Maintainer: Nitum <dev-jackson@users.noreply.github.com>
Description: Nitum PDF, visor y firmador de PDF simple y seguro
 Ver y firmar documentos PDF sin pelearse con la configuracion: certificados
 .p12/.pfx y tokens PKCS#11, sello de tiempo y validez a largo plazo activados
 por defecto, y un panel que explica por separado si el documento esta integro
 y si la identidad del firmante esta verificada.
CONTROL

cat > "$stage/DEBIAN/postinst" <<'POSTINST'
#!/bin/sh
set -e
if [ "$1" = configure ]; then
    update-desktop-database -q /usr/share/applications 2>/dev/null || true
    gtk-update-icon-cache -q -f /usr/share/icons/hicolor 2>/dev/null || true
fi
POSTINST
chmod 755 "$stage/DEBIAN/postinst"

cat > "$stage/DEBIAN/postrm" <<'POSTRM'
#!/bin/sh
set -e
rm -rf /opt/nitum-pdf/venv/__pycache__ 2>/dev/null || true
update-desktop-database -q /usr/share/applications 2>/dev/null || true
POSTRM
chmod 755 "$stage/DEBIAN/postrm"

deb="$outdir/nitum-pdf_${version}_${arch}.deb"
dpkg-deb --build --root-owner-group "$stage" "$deb" >/dev/null
echo "==> $deb"
dpkg-deb --info "$deb" | sed -n '2,8p'
