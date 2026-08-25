import pytest

from pwviewpdf.geometry import PageGeometry

A4 = PageGeometry(0, 0, 595, 842)


def test_image_size_tracks_scale():
    assert A4.image_size(1.0) == (595, 842)
    assert A4.image_size(2.0) == (1190, 1684)


def test_rotated_page_swaps_image_axes():
    rotated = PageGeometry(0, 0, 595, 842, rotation=90)
    assert rotated.image_size(1.0) == (842, 595)


def test_top_left_pixel_maps_to_top_left_of_page():
    assert A4.point_to_pdf(0, 0, 1.0) == (0, 842)


def test_bottom_right_pixel_maps_to_origin():
    assert A4.point_to_pdf(595, 842, 1.0) == (595, 0)


def test_scale_is_undone():
    assert A4.point_to_pdf(200, 400, 2.0) == (100, 842 - 200)


@pytest.mark.parametrize("rotation", [0, 90, 180, 270])
def test_rect_stays_inside_page_for_every_rotation(rotation):
    geo = PageGeometry(0, 0, 595, 842, rotation=rotation)
    width, height = geo.image_size(1.0)
    rect = geo.rect_to_pdf((10, 10, width - 10, height - 10), 1.0)
    assert 0 <= rect[0] < rect[2] <= 595
    assert 0 <= rect[1] < rect[3] <= 842


def test_rotation_90_maps_image_top_left_to_page_bottom_left():
    # With /Rotate 90 the page is displayed turned clockwise, so what the user
    # sees as the top-left corner is the page's bottom-left corner.
    geo = PageGeometry(0, 0, 595, 842, rotation=90)
    assert geo.point_to_pdf(0, 0, 1.0) == (0, 0)


def test_crop_box_offset_is_added():
    geo = PageGeometry(20, 30, 615, 872)
    assert geo.point_to_pdf(0, 0, 1.0) == (20, 872)


def test_rect_is_normalised_when_dragged_upwards():
    rect = A4.rect_to_pdf((300, 400, 100, 200), 1.0)
    assert rect[0] < rect[2] and rect[1] < rect[3]


def test_clamp_keeps_rect_on_page():
    assert A4.clamp((-50, -50, 900, 900)) == (0, 0, 595, 842)


@pytest.mark.parametrize("rotation", [0, 90, 180, 270])
@pytest.mark.parametrize("scale", [1.0, 2.0])
def test_pixel_to_pdf_round_trips(rotation, scale):
    geo = PageGeometry(20, 30, 615, 872, rotation=rotation)
    rect = (140.0, 200.0, 380.0, 260.0)
    back = geo.rect_from_pdf(geo.rect_to_pdf(rect, scale), scale)
    assert back == pytest.approx(rect)


def test_pdf_rect_maps_onto_the_rendered_image():
    geo = PageGeometry(0, 0, 595, 842)
    # a box at the bottom-left of the page must land at the bottom-left in pixels
    left, top, right, bottom = geo.rect_from_pdf((0, 0, 100, 50), 1.0)
    assert (left, right) == (0, 100)
    assert (top, bottom) == (792, 842)
