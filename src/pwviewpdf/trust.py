"""The user's own list of trusted issuers.

Acrobat only trusts the AATL and makes you dig through preferences to add
anything else, which is why so many perfectly good signatures show a yellow
triangle forever. Here the decision is made where it comes up -- next to the
signature -- and it is explicit: the user sees the fingerprint before deciding.
"""

from __future__ import annotations

import hashlib
from pathlib import Path

from asn1crypto import pem
from asn1crypto import x509 as asn1_x509

from .identities import APP_DIR

TRUST_DIR = APP_DIR / "trusted"


def fingerprint(der: bytes) -> str:
    """SHA-256, grouped in pairs, the way every other tool prints it."""
    digest = hashlib.sha256(der).hexdigest().upper()
    return " ".join(digest[i:i + 2] for i in range(0, len(digest), 2))


def _slug(name: str) -> str:
    keep = [c if c.isalnum() or c in "-_" else "-" for c in name.strip().lower()]
    return "".join(keep).strip("-") or "certificado"


def add(der: bytes, name: str, directory: Path | None = None) -> Path:
    """Store a certificate as trusted. Named after its fingerprint: no overwrites."""
    directory = directory or TRUST_DIR
    directory.mkdir(parents=True, exist_ok=True)
    short = hashlib.sha256(der).hexdigest()[:16]
    target = directory / f"{_slug(name)}-{short}.crt"
    target.write_bytes(der)
    return target


def is_trusted(der: bytes, directory: Path | None = None) -> bool:
    directory = directory or TRUST_DIR
    return any(path.read_bytes() == der for path in _files(directory))


def remove(der: bytes, directory: Path | None = None) -> None:
    for path in _files(directory or TRUST_DIR):
        if path.read_bytes() == der:
            path.unlink()


def _files(directory: Path) -> list[Path]:
    if not directory.is_dir():
        return []
    return sorted(p for p in directory.iterdir() if p.suffix in (".crt", ".cer", ".pem", ".der"))


def roots(directory: Path | None = None) -> list[asn1_x509.Certificate]:
    """Everything the user has chosen to trust, ready for a ValidationContext."""
    certificates = []
    for path in _files(directory or TRUST_DIR):
        try:
            data = path.read_bytes()
            if pem.detect(data):
                _, _, data = pem.unarmor(data)
            certificates.append(asn1_x509.Certificate.load(data))
        except Exception:
            continue            # a broken file must not disable the whole list
    return certificates
