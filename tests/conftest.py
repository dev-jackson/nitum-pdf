import datetime
import sys
from pathlib import Path

import pytest
from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.primitives.serialization import pkcs12
from cryptography.x509.oid import NameOID

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

P12_PASSWORD = b"test-password"


def _self_signed(common_name="Ada Lovelace"):
    key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    subject = x509.Name([
        x509.NameAttribute(NameOID.COMMON_NAME, common_name),
        x509.NameAttribute(NameOID.ORGANIZATION_NAME, "pw-view-pdf tests"),
    ])
    now = datetime.datetime.now(datetime.timezone.utc)
    cert = (
        x509.CertificateBuilder()
        .subject_name(subject).issuer_name(subject)
        .public_key(key.public_key()).serial_number(x509.random_serial_number())
        .not_valid_before(now - datetime.timedelta(days=1))
        .not_valid_after(now + datetime.timedelta(days=365))
        .add_extension(x509.BasicConstraints(ca=True, path_length=None), critical=True)
        .add_extension(
            x509.KeyUsage(
                digital_signature=True, content_commitment=True, key_encipherment=False,
                data_encipherment=False, key_agreement=False, key_cert_sign=True,
                crl_sign=True, encipher_only=False, decipher_only=False,
            ),
            critical=True,
        )
        .sign(key, hashes.SHA256())
    )
    return key, cert


@pytest.fixture(scope="session")
def identity(tmp_path_factory):
    """A self-signed .p12 plus its certificate in DER form (for trust_roots)."""
    key, cert = _self_signed()
    path = tmp_path_factory.mktemp("id") / "tester.p12"
    path.write_bytes(pkcs12.serialize_key_and_certificates(
        name=b"tester", key=key, cert=cert, cas=None,
        encryption_algorithm=serialization.BestAvailableEncryption(P12_PASSWORD),
    ))
    return {"p12": path, "password": P12_PASSWORD,
            "cert_der": cert.public_bytes(serialization.Encoding.DER)}


@pytest.fixture
def sample_pdf(tmp_path):
    import pypdfium2 as pdfium

    pdf = pdfium.PdfDocument.new()
    for _ in range(3):
        pdf.new_page(595, 842)          # A4 in points
    path = tmp_path / "sample.pdf"
    pdf.save(str(path))
    pdf.close()
    return path


@pytest.fixture
def text_pdf(tmp_path):
    """A two-page PDF with real text, for search and rendering tests."""
    import importlib.util

    spec = importlib.util.spec_from_file_location(
        "make_sample", Path(__file__).resolve().parents[1] / "scripts" / "make_sample.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.build(tmp_path / "contrato.pdf")


@pytest.fixture
def encrypted_pdf(text_pdf, tmp_path):
    """The same document, encrypted with a user password."""
    from pyhanko.pdf_utils.reader import PdfFileReader
    from pyhanko.pdf_utils.writer import copy_into_new_writer

    target = tmp_path / "encrypted.pdf"
    with open(text_pdf, "rb") as source:
        writer = copy_into_new_writer(PdfFileReader(source))
        writer.encrypt("secreto")
        with open(target, "wb") as out:
            writer.write(out)
    return target
