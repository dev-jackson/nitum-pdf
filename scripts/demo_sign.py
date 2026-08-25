"""End-to-end check: make an identity, sign a real PDF, verify, print the result.

Usage: python scripts/demo_sign.py <input.pdf> <workdir> [--tsa] [--appearance=signature.png]
"""

from __future__ import annotations

import datetime
import sys
from pathlib import Path

from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.primitives.serialization import pkcs12
from cryptography.x509.oid import NameOID

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pwviewpdf import signing, trust  # noqa: E402
from pwviewpdf.geometry import PageGeometry  # noqa: E402

PASSWORD = b"demo"


def make_identity(directory: Path) -> tuple[Path, bytes]:
    key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    name = x509.Name([
        x509.NameAttribute(NameOID.COMMON_NAME, "Ada Lovelace"),
        x509.NameAttribute(NameOID.ORGANIZATION_NAME, "PW Servicios S.L."),
    ])
    now = datetime.datetime.now(datetime.timezone.utc)
    cert = (
        x509.CertificateBuilder()
        .subject_name(name).issuer_name(name)
        .public_key(key.public_key()).serial_number(x509.random_serial_number())
        .not_valid_before(now - datetime.timedelta(days=1))
        .not_valid_after(now + datetime.timedelta(days=365))
        .add_extension(x509.BasicConstraints(ca=True, path_length=None), critical=True)
        .add_extension(x509.KeyUsage(
            digital_signature=True, content_commitment=True, key_encipherment=False,
            data_encipherment=False, key_agreement=False, key_cert_sign=True,
            crl_sign=True, encipher_only=False, decipher_only=False), critical=True)
        .sign(key, hashes.SHA256())
    )
    directory.mkdir(parents=True, exist_ok=True)
    p12 = directory / "ada.p12"
    p12.write_bytes(pkcs12.serialize_key_and_certificates(
        name=b"ada", key=key, cert=cert, cas=None,
        encryption_algorithm=serialization.BestAvailableEncryption(PASSWORD),
    ))
    return p12, cert.public_bytes(serialization.Encoding.DER)


def main() -> int:
    source = Path(sys.argv[1])
    workdir = Path(sys.argv[2])
    want_tsa = "--tsa" in sys.argv
    appearance = next((Path(arg.split("=", 1)[1]) for arg in sys.argv
                       if arg.startswith("--appearance=")), None)

    p12, cert_der = make_identity(workdir / "identities")
    geometry = PageGeometry(0, 0, 595, 842)
    # Where a user would drag: over "Firma del prestador:" on page 1.
    box = geometry.rect_to_pdf((70, 400, 270, 460), 1.0)
    options = signing.SignOptions(
        page=0, box=box, reason="Conforme con el contenido", location="Madrid",
        want_timestamp=want_tsa, want_ltv=False, signature_image=appearance,
    )
    target = signing.suggest_output(workdir / source.name)
    result = signing.sign_with_pkcs12(source, target, p12, PASSWORD, options)

    print(f"firmado -> {result.output}")
    print(f"nivel   -> {result.level}")
    if result.downgrade_reason:
        print(f"degradado: {result.downgrade_reason[:120]}")
    for status in result.statuses:
        print(f"  {status.field_name}: {status.common_name} "
              f"| integro={status.intact} | confiable={status.trusted}")

    store = workdir / "trusted"
    trust.add(cert_der, "Ada Lovelace", store)
    after = signing.verify(result.output, trust_roots=trust.roots(store))
    print(f"tras confiar en el emisor: confiable={after[0].trusted}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
