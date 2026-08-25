import pytest

from pwviewpdf.document import Document


def test_reports_page_count(sample_pdf):
    doc = Document(sample_pdf)
    assert doc.page_count == 3
    doc.close()


def test_geometry_matches_a4(sample_pdf):
    doc = Document(sample_pdf)
    geo = doc.geometry(0)
    assert (round(geo.width), round(geo.height)) == (595, 842)
    doc.close()


def test_render_size_follows_geometry(sample_pdf):
    doc = Document(sample_pdf)
    page = doc.render(0, 1.0)
    assert (page.width, page.height) == doc.geometry(0).image_size(1.0)
    assert len(page.data) == page.stride * page.height
    doc.close()


def test_render_is_always_four_channels(sample_pdf):
    # Opaque pages come back as 3-channel BGR from pdfium, which the toolkit
    # rejects. Rendering must normalise that away.
    doc = Document(sample_pdf)
    page = doc.render(0, 1.0)
    assert page.bytes_per_pixel == 4
    assert page.stride == page.width * 4
    doc.close()


def test_render_scales_up(sample_pdf):
    doc = Document(sample_pdf)
    assert doc.render(0, 2.0).width == 2 * doc.render(0, 1.0).width
    doc.close()


def test_fit_width_scale_fills_the_viewport(sample_pdf):
    doc = Document(sample_pdf)
    scale = doc.scale_to_fit_width(0, 1190)
    assert doc.render(0, scale).width == 1190
    doc.close()


def test_search_on_a_blank_document_finds_nothing(sample_pdf):
    doc = Document(sample_pdf)
    assert doc.search("contrato") == []
    assert doc.search("   ") == []
    doc.close()


def test_search_finds_text_and_reports_where(text_pdf):
    doc = Document(text_pdf)
    hits = doc.search("salamandra")
    assert len(hits) == 1
    assert hits[0].page == 1
    x0, y0, x1, y1 = hits[0].rect
    assert x1 > x0 and y1 > y0
    doc.close()


def test_search_is_case_insensitive(text_pdf):
    doc = Document(text_pdf)
    assert len(doc.search("SALAMANDRA")) == 1
    doc.close()


def test_search_reports_every_occurrence(text_pdf):
    doc = Document(text_pdf)
    assert len(doc.search("contrato")) >= 2
    doc.close()


def test_hit_rectangle_lands_on_the_rendered_page(text_pdf):
    doc = Document(text_pdf)
    hit = doc.search("salamandra")[0]
    geometry = doc.geometry(hit.page)
    left, top, right, bottom = geometry.rect_from_pdf(hit.rect, 1.0)
    width, height = geometry.image_size(1.0)
    assert 0 <= left < right <= width
    assert 0 <= top < bottom <= height
    doc.close()


def test_pages_with_lists_pages_once(text_pdf):
    doc = Document(text_pdf)
    assert doc.pages_with("contrato") == [0]
    doc.close()


def test_encrypted_pdf_asks_for_a_password(encrypted_pdf):
    from pwviewpdf.document import PasswordRequired

    with pytest.raises(PasswordRequired):
        Document(encrypted_pdf)


def test_encrypted_pdf_opens_with_the_password(encrypted_pdf):
    doc = Document(encrypted_pdf, password="secreto")
    assert doc.page_count == 2
    doc.close()
