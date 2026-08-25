from PIL import Image, ImageDraw
import pytest

from pwviewpdf import appearances


def test_signature_image_is_cropped_normalised_and_private(tmp_path):
    source = tmp_path / "paper.png"
    image = Image.new("RGB", (800, 300), "white")
    draw = ImageDraw.Draw(image)
    draw.line((250, 150, 550, 100), fill="navy", width=8)
    image.save(source)
    target = tmp_path / "saved" / "signature.png"

    result = appearances.import_signature(source, target)

    with Image.open(result) as saved:
        assert saved.mode == "RGBA"
        assert saved.width < 800 and saved.height < 300
        assert saved.getchannel("A").getbbox() is not None
    assert result.stat().st_mode & 0o777 == 0o600


def test_blank_image_is_rejected(tmp_path):
    source = tmp_path / "blank.png"
    Image.new("RGB", (100, 50), "white").save(source)
    with pytest.raises(ValueError, match="visible"):
        appearances.import_signature(source, tmp_path / "out.png")
