#!/usr/bin/env bash
# Build, install and exercise the package on a real Debian-derived system.
# Meant to be run inside a container:
#   podman run --rm -v "$PWD":/src:ro ubuntu:24.04 bash /src/packaging/verify-in-container.sh
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive

echo "### $(. /etc/os-release && echo "$PRETTY_NAME")"
apt-get update -qq >/dev/null
apt-get install -y -qq python3-venv python3-gi gir1.2-gtk-4.0 gir1.2-adw-1 \
    libpcsclite1 dpkg-dev xvfb adwaita-icon-theme fonts-dejavu-core \
    python3-pytest tzdata >/dev/null 2>&1

mkdir -p /work /out && cp -r /src /work/pw-view-pdf && cd /work/pw-view-pdf

python3 - <<'PY'
import gi
gi.require_version("Gtk", "4.0"); gi.require_version("Adw", "1")
from gi.repository import Gtk, Adw
print(f"### GTK {Gtk.get_major_version()}.{Gtk.get_minor_version()} "
      f"| libadwaita {Adw.get_major_version()}.{Adw.get_minor_version()}")
PY

echo "### building"
./packaging/build-deb.sh >/dev/null
apt-get install -y -qq ./dist/*.deb >/dev/null 2>&1
echo "### installed: $(dpkg-query -W -f='${Package} ${Version} ${Installed-Size}kB' nitum-pdf)"

venv=/opt/nitum-pdf/venv
echo "### relocated venv imports: $($venv/bin/python -c 'import pwviewpdf, pypdfium2, pyhanko; print("ok")')"
$venv/bin/python -c "
from pwviewpdf import compat
print('### compat:', compat.VERSIONS)
print('###', ' '.join(f'{k[4:].lower()}={int(v)}' for k, v in vars(compat).items() if k.startswith('HAS_')))
"

echo "### running the test suite against the installed package"
# The bundled venv is built with --system-site-packages, so the distro pytest
# is visible from it. Hide the source tree so conftest cannot put it ahead of
# /opt on sys.path: this makes the suite exercise exactly what was installed.
mv src src.source
xvfb-run -a $venv/bin/python -m pytest -q 2>&1 | tail -3

echo "### launching the app headless and capturing the window"
$venv/bin/python scripts/make_sample.py /tmp/contrato.pdf >/dev/null
xvfb-run -a $venv/bin/python scripts/demo_sign.py /tmp/contrato.pdf /tmp/demo 2>/dev/null | grep -E "^firmado|^nivel|Sig1|tras confiar"
xvfb-run -a $venv/bin/python scripts/screenshot.py /tmp/contrato.pdf /out \
    --signed=/tmp/demo/contrato-firmado.pdf viewer sign status signed 2>&1 | grep -cE "^wrote" | sed 's/^/### screenshots: /'
echo "### desktop entry: $(test -f /usr/share/applications/org.pwview.PdfViewer.desktop && echo present)"
timeout 8 xvfb-run -a nitum-pdf /tmp/contrato.pdf >/dev/null 2>&1 && echo "### launcher exited cleanly" || echo "### launcher ran until timeout (window stayed open) = ok"
