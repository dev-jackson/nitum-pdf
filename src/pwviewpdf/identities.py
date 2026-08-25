"""Finding something to sign with, without asking the user for a library path.

Three sources, merged into one list: PKCS#11 tokens announced by p11-kit, well
known vendor modules, and PKCS#12 files the user imported (typically exported
from the Windows certificate store).
"""

from __future__ import annotations

import os
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path

# Keep the original data directory so existing identities and trust decisions
# survive the Nitum PDF rebrand.
APP_DIR = Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local/share")) / "pw-view-pdf"
IDENTITY_DIR = APP_DIR / "identities"

WELL_KNOWN_MODULES = (
    "/usr/lib/opensc-pkcs11.so",
    "/usr/lib64/opensc-pkcs11.so",
    "/usr/lib/x86_64-linux-gnu/opensc-pkcs11.so",
    "/usr/lib/pkcs11/opensc-pkcs11.so",
    "/usr/lib/libeToken.so",
    "/usr/lib/libeTPkcs11.so",
    # NSS soft token: reuses whatever the user already imported in Firefox/Okular
    "/usr/lib64/libsoftokn3.so",
    "/usr/lib/x86_64-linux-gnu/nss/libsoftokn3.so",
)


@dataclass(frozen=True)
class Identity:
    kind: str                 # "pkcs12" or "pkcs11"
    label: str                # shown to the user
    path: str                 # .p12 file, or the PKCS#11 module
    token_label: str | None = None

    @property
    def is_token(self) -> bool:
        return self.kind == "pkcs11"

    @property
    def secret_prompt(self) -> str:
        return "PIN del token" if self.is_token else "Contraseña del certificado"


def _p11kit_modules(runner=subprocess.run) -> list[tuple[str, str | None]]:
    try:
        output = runner(
            ["p11-kit", "list-modules"], capture_output=True, text=True,
            timeout=10, check=True,
        ).stdout
    except (OSError, subprocess.SubprocessError):
        return []

    found: list[tuple[str, str | None]] = []
    module_path: str | None = None
    for line in output.splitlines():
        path_match = re.match(r"^\s*path:\s*(\S+)", line)
        if path_match:
            module_path = path_match.group(1)
            continue
        token_match = re.match(r"^\s*token:\s*(.+?)\s*$", line)
        if token_match and module_path:
            found.append((module_path, token_match.group(1)))
    return found


def discover(identity_dir: Path | None = None, runner=subprocess.run) -> list[Identity]:
    """Everything available right now. Tokens first: they are the strong credential."""
    identity_dir = identity_dir or IDENTITY_DIR
    identities: list[Identity] = []

    seen: set[str] = set()
    for path, token in _p11kit_modules(runner):
        seen.add(path)
        identities.append(Identity(
            kind="pkcs11", label=token or f"Token ({Path(path).name})",
            path=path, token_label=token,
        ))

    for path in WELL_KNOWN_MODULES:
        if path not in seen and Path(path).exists():
            identities.append(Identity(
                kind="pkcs11", label=f"Token o tarjeta ({Path(path).name})", path=path,
            ))

    for pattern in ("*.p12", "*.pfx"):
        for file in sorted(identity_dir.glob(pattern)):
            identities.append(Identity(kind="pkcs12", label=file.stem, path=str(file)))

    return identities


def import_pkcs12(source: str | Path, identity_dir: Path | None = None) -> Path:
    """Copy a .pfx/.p12 (e.g. exported from Windows) into the local store."""
    source = Path(source)
    identity_dir = identity_dir or IDENTITY_DIR
    identity_dir.mkdir(parents=True, exist_ok=True)
    target = identity_dir / source.name
    target.write_bytes(source.read_bytes())
    target.chmod(0o600)
    return target
