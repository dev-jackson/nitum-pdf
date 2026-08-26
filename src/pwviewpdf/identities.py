"""Finding something to sign with, without asking the user for a library path.

Three sources, merged into one list: PKCS#11 tokens announced by p11-kit, well
known vendor modules, and PKCS#12 files the user imported (typically exported
from the Windows certificate store).
"""

from __future__ import annotations

import os
import re
import subprocess
import datetime
from dataclasses import dataclass
from pathlib import Path

from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.primitives.serialization import pkcs12
from cryptography.x509.oid import NameOID

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


@dataclass(frozen=True)
class IdentityDetails:
    label: str
    organization: str | None
    expires: datetime.datetime


def inspect_pkcs12(source: str | Path, password: str) -> IdentityDetails:
    """Open a signing identity now, so failures never surface after the form."""
    try:
        key, certificate, _chain = pkcs12.load_key_and_certificates(
            Path(source).read_bytes(), password.encode() if password else None,
        )
    except (OSError, ValueError) as exc:
        raise ValueError(
            "No se pudo abrir la identidad. Comprueba que sea un archivo .p12/.pfx "
            "y que la contraseña sea correcta."
        ) from exc
    if key is None or certificate is None:
        raise ValueError(
            "Este archivo no contiene una clave privada y no puede usarse para firmar."
        )
    common_names = certificate.subject.get_attributes_for_oid(NameOID.COMMON_NAME)
    organizations = certificate.subject.get_attributes_for_oid(NameOID.ORGANIZATION_NAME)
    label = common_names[0].value if common_names else Path(source).stem
    organization = organizations[0].value if organizations else None
    expires = getattr(certificate, "not_valid_after_utc", None)
    if expires is None:
        expires = certificate.not_valid_after.replace(tzinfo=datetime.timezone.utc)
    if expires <= datetime.datetime.now(datetime.timezone.utc):
        raise ValueError(
            f"La identidad de {label} venció el {expires.date().isoformat()}. "
            "Solicita o importa una identidad vigente."
        )
    return IdentityDetails(label, organization, expires)


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


def _safe_name(label: str) -> str:
    clean = re.sub(r"[^\w .-]+", "", label, flags=re.UNICODE).strip(" .")
    return clean or "identidad-digital"


def import_pkcs12(source: str | Path, identity_dir: Path | None = None,
                  label: str | None = None) -> Path:
    """Copy a .pfx/.p12 (e.g. exported from Windows) into the local store."""
    source = Path(source)
    identity_dir = identity_dir or IDENTITY_DIR
    identity_dir.mkdir(parents=True, exist_ok=True)
    target = identity_dir / f"{_safe_name(label or source.stem)}{source.suffix.lower()}"
    index = 2
    while target.exists() and target.read_bytes() != source.read_bytes():
        target = identity_dir / f"{_safe_name(label or source.stem)}-{index}{source.suffix.lower()}"
        index += 1
    target.write_bytes(source.read_bytes())
    target.chmod(0o600)
    return target


def create_local(name: str, email: str, password: str,
                 identity_dir: Path | None = None) -> Path:
    """Create Acrobat-compatible PKCS#12 identity for personal/internal use."""
    name = name.strip()
    if not name:
        raise ValueError("Escribe el nombre que debe aparecer en la firma.")
    if len(password) < 8:
        raise ValueError("La contraseña debe tener al menos 8 caracteres.")
    identity_dir = identity_dir or IDENTITY_DIR
    identity_dir.mkdir(parents=True, exist_ok=True)
    key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    attributes = [x509.NameAttribute(NameOID.COMMON_NAME, name)]
    if email.strip():
        attributes.append(x509.NameAttribute(NameOID.EMAIL_ADDRESS, email.strip()))
    subject = x509.Name(attributes)
    now = datetime.datetime.now(datetime.timezone.utc)
    certificate = (
        x509.CertificateBuilder()
        .subject_name(subject).issuer_name(subject).public_key(key.public_key())
        .serial_number(x509.random_serial_number())
        .not_valid_before(now - datetime.timedelta(minutes=5))
        .not_valid_after(now + datetime.timedelta(days=3650))
        .add_extension(x509.BasicConstraints(ca=False, path_length=None), critical=True)
        .add_extension(x509.KeyUsage(
            digital_signature=True, content_commitment=True, key_encipherment=False,
            data_encipherment=False, key_agreement=False, key_cert_sign=False,
            crl_sign=False, encipher_only=False, decipher_only=False,
        ), critical=True)
        .sign(key, hashes.SHA256())
    )
    data = pkcs12.serialize_key_and_certificates(
        name=name.encode(), key=key, cert=certificate, cas=None,
        encryption_algorithm=serialization.BestAvailableEncryption(password.encode()),
    )
    target = identity_dir / f"{_safe_name(name)}.p12"
    index = 2
    while target.exists():
        target = identity_dir / f"{_safe_name(name)}-{index}.p12"
        index += 1
    target.write_bytes(data)
    target.chmod(0o600)
    return target
