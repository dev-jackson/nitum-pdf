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
# Never mix host artifacts (possibly built for another architecture) with the
# package this container is about to verify.
rm -rf /work/pw-view-pdf/build /work/pw-view-pdf/dist

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
$venv/bin/python - <<'PY'
from PIL import Image, ImageDraw
image = Image.new("RGBA", (600, 180), (255, 255, 255, 0))
draw = ImageDraw.Draw(image)
draw.line((20, 125, 180, 75, 300, 120, 560, 35), fill="#123f91", width=10, joint="curve")
image.save("/tmp/nitum-demo-signature.png")
PY
xvfb-run -a $venv/bin/python scripts/demo_sign.py /tmp/contrato.pdf /tmp/demo \
    --appearance=/tmp/nitum-demo-signature.png 2>/dev/null | grep -E "^firmado|^nivel|Sig1|tras confiar"
xvfb-run -a $venv/bin/python scripts/screenshot.py /tmp/contrato.pdf /out \
    --signed=/tmp/demo/contrato-firmado.pdf --appearance=/tmp/nitum-demo-signature.png \
    empty viewer narrow search signature-center identity-import identity-create \
    placing sign status signed dark-signature-center dark-sign \
    2>&1 | grep -cE "^wrote" | sed 's/^/### screenshots: /'
echo "### desktop entry: $(test -f /usr/share/applications/org.pwview.PdfViewer.desktop && echo present)"
timeout 8 xvfb-run -a nitum-pdf /tmp/contrato.pdf >/dev/null 2>&1 && echo "### launcher exited cleanly" || echo "### launcher ran until timeout (window stayed open) = ok"
