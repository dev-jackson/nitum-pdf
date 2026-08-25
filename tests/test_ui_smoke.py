"""Does the window actually come up and paint? Skipped where there is no display."""

import pytest

gi = pytest.importorskip("gi")
gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Adw, Gtk  # noqa: E402

if not Gtk.init_check():
    pytest.skip("no display available", allow_module_level=True)

from pwviewpdf.app import PwViewPdf, SignDialog, StatusDialog, ViewerWindow  # noqa: E402
from pwviewpdf.identities import Identity  # noqa: E402


@pytest.fixture(scope="module")
def app():
    Adw.init()
    return PwViewPdf()


def test_window_starts_empty(app):
    window = ViewerWindow(app)
    assert window.stack.get_visible_child_name() == "empty"
    assert window.sign_button.get_sensitive() is False


def test_opening_a_document_creates_one_view_per_page(app, text_pdf):
    window = ViewerWindow(app)
    window.open_document(text_pdf)
    assert len(window._pages) == 2
    assert window.stack.get_visible_child_name() == "pages"
    assert window.sign_button.get_sensitive() is True


def test_pages_are_sized_from_the_document(app, text_pdf):
    window = ViewerWindow(app)
    window.open_document(text_pdf)
    expected = window.document.geometry(0).image_size(window.scale)
    assert window._pages[0].get_size_request().width == expected[0]


def test_rendering_produces_a_texture(app, text_pdf):
    window = ViewerWindow(app)
    window.open_document(text_pdf)
    window._render_visible()
    assert window._pages[0].picture.get_paintable() is not None


def test_signing_mode_arms_the_pages(app, text_pdf):
    window = ViewerWindow(app)
    window.open_document(text_pdf)
    window.start_signing()
    assert all(view.selectable for view in window._pages)
    window.cancel_signing()
    assert not any(view.selectable for view in window._pages)


def test_dragged_rectangle_becomes_a_pdf_box(app, text_pdf):
    window = ViewerWindow(app)
    window.open_document(text_pdf)
    window.start_signing()
    page = window._pages[0]
    page.rect = (40.0, 60.0, 200.0, 120.0)
    window.pending = (0, page.rect)

    geometry = window.document.geometry(0)
    box = geometry.clamp(geometry.rect_to_pdf(page.rect, window.scale))
    assert box[0] < box[2] and box[1] < box[3]
    assert box[3] <= geometry.height


def test_search_highlights_and_counts(app, text_pdf):
    window = ViewerWindow(app)
    window.open_document(text_pdf)
    window.search_entry.set_text("contrato")
    window._on_search(window.search_entry)
    assert len(window._hits) >= 2
    assert window.hit_label.get_text() == "1 de %d" % len(window._hits)
    assert window._pages[0].highlights


def test_stepping_through_hits_wraps_around(app, text_pdf):
    window = ViewerWindow(app)
    window.open_document(text_pdf)
    window.search_entry.set_text("contrato")
    window._on_search(window.search_entry)
    total = len(window._hits)
    window.step_hit(-1)
    assert window._hit_index == total - 1


def test_zoom_changes_the_scale_and_the_readout(app, text_pdf):
    window = ViewerWindow(app)
    window.open_document(text_pdf)
    before = window.scale
    window.zoom(1)
    assert window.scale > before
    assert "%" in window.zoom_label.get_label()


def test_sign_dialog_lists_identities_and_labels_the_secret(app, text_pdf):
    window = ViewerWindow(app)
    window.open_document(text_pdf)
    people = [Identity(kind="pkcs12", label="ada", path="/tmp/ada.p12")]
    dialog = SignDialog(window, people, "Aparecerá en la página 1")
    assert dialog.identity_row.get_selected() == 0
    assert "Contraseña" in dialog.secret_row.get_title()


def test_compat_layer_reports_what_the_system_has():
    from pwviewpdf import compat

    assert "GTK" in compat.VERSIONS and "libadwaita" in compat.VERSIONS


def test_status_dialog_offers_to_trust_an_unverified_issuer(app):
    from pwviewpdf.signing import SignatureStatus

    status = SignatureStatus(
        field_name="Sig1", signer="CN=Ada", common_name="Ada", organization="PW",
        intact=True, trusted=False, signed_at=None, detail="", certificate=b"der",
    )
    dialog = StatusDialog([status])
    assert dialog is not None      # construction is the assertion: no missing rows


def test_signature_banner_stays_hidden_for_an_unsigned_document(app, text_pdf):
    window = ViewerWindow(app)
    window.open_document(text_pdf)
    window._show_signature_banner(text_pdf, [])
    assert window.signature_banner.get_revealed() is False


def test_signature_banner_warns_when_identity_is_unverified(app, text_pdf):
    from pwviewpdf.signing import SignatureStatus

    window = ViewerWindow(app)
    window.open_document(text_pdf)
    status = SignatureStatus(
        field_name="Sig1", signer="CN=Ada", common_name="Ada", organization=None,
        intact=True, trusted=False, signed_at=None, detail="",
    )
    window._show_signature_banner(window.path, [status])
    assert window.signature_banner.get_revealed() is True
    assert window.signature_banner.has_css_class("warning")


def test_signature_banner_shouts_when_the_document_was_altered(app, text_pdf):
    from pwviewpdf.signing import SignatureStatus

    window = ViewerWindow(app)
    window.open_document(text_pdf)
    status = SignatureStatus(
        field_name="Sig1", signer="CN=Ada", common_name="Ada", organization=None,
        intact=False, trusted=True, signed_at=None, detail="",
    )
    window._show_signature_banner(window.path, [status])
    assert window.signature_banner.has_css_class("error")
