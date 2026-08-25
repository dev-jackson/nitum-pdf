#!/usr/bin/env bash
# Run without installing anything system-wide. Handy for trying it out.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
[ -d "$here/.venv" ] || python3 -m venv --system-site-packages "$here/.venv"
"$here/.venv/bin/pip" install --quiet -e "$here[dev]"
exec "$here/.venv/bin/nitum-pdf" "$@"
