import pytest
from asn1crypto import x509 as asn1_x509

from pwviewpdf import signing
from pwviewpdf.document import Document
from pwviewpdf.geometry import PageGeometry

OFFLINE = signing.SignOptions(want_timestamp=False, want_ltv=False)


def roots(identity):
    return [asn1_x509.Certificate.load(identity["cert_der"])]


def sign(sample_pdf, identity, options=OFFLINE, trust_roots=None):
    target = signing.suggest_output(sample_pdf)
    return signing.sign_with_pkcs12(
        sample_pdf, target, identity["p12"], identity["password"],
        options, trust_roots=trust_roots,
    )


def test_signs_and_reports_intact(sample_pdf, identity):
    result = sign(sample_pdf, identity)
    assert result.output.exists()
    assert result.level == signing.LEVEL_B
    assert len(result.statuses) == 1
    assert result.statuses[0].intact is True


def test_self_signed_identity_is_not_trusted_by_default(sample_pdf, identity):
    assert sign(sample_pdf, identity).statuses[0].trusted is False


def test_identity_becomes_trusted_once_its_root_is_known(sample_pdf, identity):
    result = sign(sample_pdf, identity, trust_roots=roots(identity))
    assert result.statuses[0].trusted is True
    assert "Ada Lovelace" in result.statuses[0].signer


def test_status_exposes_a_readable_name(sample_pdf, identity):
    status = sign(sample_pdf, identity).statuses[0]
    assert status.common_name == "Ada Lovelace"
    assert status.organization == "pw-view-pdf tests"


def test_original_document_is_never_modified(sample_pdf, identity):
    before = sample_pdf.read_bytes()
    sign(sample_pdf, identity)
    assert sample_pdf.read_bytes() == before


def test_output_name_does_not_clobber(sample_pdf, identity):
    first = sign(sample_pdf, identity).output
    second = signing.suggest_output(sample_pdf)
    assert second != first and not second.exists()


def test_second_signature_keeps_the_first_one_valid(sample_pdf, identity):
    first = sign(sample_pdf, identity).output
    second = signing.suggest_output(first)
    result = signing.sign_with_pkcs12(
        first, second, identity["p12"], identity["password"], OFFLINE
    )
    assert [s.field_name for s in result.statuses] == ["Sig1", "Sig2"]
    assert all(s.intact for s in result.statuses)


def test_tampering_is_detected(sample_pdf, identity):
    signed = sign(sample_pdf, identity).output
    # Same-length edit inside the signed range: the file still parses, so the
    # only thing that can catch it is the signature itself.
    data = signed.read_bytes()
    assert b"595" in data
    signed.write_bytes(data.replace(b"595", b"594", 1))
    assert signing.verify(signed)[0].intact is False


def test_damaged_file_is_reported_instead_of_crashing(sample_pdf, identity):
    signed = sign(sample_pdf, identity).output
    signed.write_bytes(signed.read_bytes()[: len(signed.read_bytes()) // 2])
    with pytest.raises(signing.DocumentUnreadable):
        signing.verify(signed)


def test_visible_signature_lands_where_the_user_dragged(sample_pdf, identity):
    geo = PageGeometry(0, 0, 595, 842)
    box = geo.rect_to_pdf((100, 100, 300, 180), 1.0)   # pixels at scale 1.0
    options = signing.SignOptions(page=1, box=box, want_timestamp=False, want_ltv=False)
    signed = sign(sample_pdf, identity, options).output

    rect = _widget_rect(signed, page_index=1)
    assert rect == pytest.approx(box, abs=1.0)


def test_invisible_signature_has_no_visible_area(sample_pdf, identity):
    signed = sign(sample_pdf, identity).output
    rect = _widget_rect(signed, page_index=0)
    assert rect is None or rect == (0.0, 0.0, 0.0, 0.0)


def test_falls_back_to_a_plain_signature_when_the_tsa_is_unreachable(sample_pdf, identity):
    options = signing.SignOptions(
        want_timestamp=True, want_ltv=False,
        tsa_url="http://127.0.0.1:9/nope",     # discard port: always refused
    )
    result = sign(sample_pdf, identity, options)
    assert result.level == signing.LEVEL_B
    assert result.downgrade_reason           # the UI must be able to explain why
    assert result.statuses[0].intact is True


def test_wrong_password_is_reported_clearly(sample_pdf, identity):
    with pytest.raises(Exception) as excinfo:
        signing.sign_with_pkcs12(
            sample_pdf, signing.suggest_output(sample_pdf),
            identity["p12"], b"wrong", OFFLINE,
        )
    assert "contraseña" in str(excinfo.value).lower() or "mac" in str(excinfo.value).lower()


def test_visible_signature_is_actually_drawn(sample_pdf, identity):
    # Regression: pdfium needs a form environment to paint widget annotations.
    geo = PageGeometry(0, 0, 595, 842)
    box = geo.rect_to_pdf((70, 400, 270, 460), 1.0)
    options = signing.SignOptions(page=0, box=box, want_timestamp=False, want_ltv=False)
    signed = sign(sample_pdf, identity, options).output

    assert _ink(sample_pdf, 0, 70, 400, 270, 460) == 0
    assert _ink(signed, 0, 70, 400, 270, 460) > 200


def test_saved_visual_signature_is_embedded_in_digital_stamp(sample_pdf, identity, tmp_path):
    from PIL import Image, ImageDraw

    visual = tmp_path / "signature.png"
    image = Image.new("RGBA", (500, 180), (255, 255, 255, 0))
    ImageDraw.Draw(image).line((30, 120, 460, 45), fill="navy", width=12)
    image.save(visual)
    box = PageGeometry(0, 0, 595, 842).rect_to_pdf((70, 400, 370, 500), 1.0)
    options = signing.SignOptions(
        page=0, box=box, signature_image=visual,
        want_timestamp=False, want_ltv=False,
    )
    signed = sign(sample_pdf, identity, options).output
    assert _ink(signed, 0, 70, 400, 370, 500) > 500


def test_first_signature_can_certify_the_document(sample_pdf, identity):
    options = signing.SignOptions(certify=True, want_timestamp=False, want_ltv=False)
    signed = sign(sample_pdf, identity, options).output
    from pyhanko.pdf_utils.reader import PdfFileReader
    with open(signed, "rb") as handle:
        embedded = PdfFileReader(handle).embedded_signatures[0]
        assert embedded.docmdp_level is not None


def test_invisible_signature_adds_no_ink(sample_pdf, identity):
    signed = sign(sample_pdf, identity).output
    assert _ink(signed, 0, 0, 0, 595, 842) == 0


def _ink(pdf, page_index, x1, y1, x2, y2) -> int:
    """Count dark pixels inside an image-space rectangle of a rendered page."""
    doc = Document(pdf)
    try:
        page = doc.render(page_index, 1.0)
    finally:
        doc.close()
    dark = 0
    for y in range(int(y1), int(y2)):
        row = page.data[y * page.stride:(y + 1) * page.stride]
        for x in range(int(x1), int(x2)):
            if row[x * 4] < 200:
                dark += 1
    return dark


def test_signed_document_still_renders(sample_pdf, identity):
    signed = sign(sample_pdf, identity).output
    doc = Document(signed)
    assert doc.page_count == 3
    assert doc.render(0, 1.0).width == 595
    doc.close()


def _widget_rect(pdf, page_index):
    """Annotation rectangle of the signature widget on `page_index`, if any."""
    from pyhanko.pdf_utils.reader import PdfFileReader

    with open(pdf, "rb") as handle:
        reader = PdfFileReader(handle)
        page = reader.root["/Pages"]["/Kids"][page_index]
        for annot in page.get("/Annots", []):
            annot = annot.get_object()
            if annot.get("/FT") == "/Sig" or annot.get("/Subtype") == "/Widget":
                return tuple(float(v) for v in annot["/Rect"])
    return None


@pytest.mark.parametrize("raw,expected", [
    ("CKR_PIN_INCORRECT", "El PIN del token no es correcto."),
    ("Error: CKR_TOKEN_NOT_PRESENT", "No se encuentra el token. ¿Está conectado?"),
    ("MAC verification failed", "La contraseña del certificado no es correcta."),
    ("CKR_PIN_LOCKED", "El token está bloqueado por demasiados intentos fallidos."),
])
def test_library_errors_become_actionable_messages(raw, expected):
    assert signing.friendly_error(Exception(raw)) == expected


def test_unknown_errors_are_passed_through(sample_pdf):
    assert signing.friendly_error(Exception("boom")) == "boom"
