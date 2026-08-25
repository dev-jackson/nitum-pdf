"""A reusable visual signature, deliberately separate from digital identity.

The image is only an appearance. The certificate and private key in signing.py
are what make the resulting signature verifiable.
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image

from .identities import APP_DIR

APPEARANCE_DIR = APP_DIR / "appearances"
SIGNATURE_IMAGE = APPEARANCE_DIR / "signature.png"


def saved() -> Path | None:
    return SIGNATURE_IMAGE if SIGNATURE_IMAGE.is_file() else None


def import_signature(source: str | Path, target: Path | None = None) -> Path:
    """Validate, crop and normalise a handwritten signature image."""
    target = target or SIGNATURE_IMAGE
    with Image.open(source) as original:
        image = original.convert("RGBA")
        if image.width < 16 or image.height < 8:
            raise ValueError("la imagen es demasiado pequeña")
        # Treat near-white pixels as transparent, which makes phone scans and
        # white-paper signatures blend naturally into the PDF stamp.
        pixels = image.load()
        for y in range(image.height):
            for x in range(image.width):
                red, green, blue, alpha = pixels[x, y]
                whiteness = min(red, green, blue)
                pixels[x, y] = (red, green, blue,
                                min(alpha, max(0, 255 - whiteness) * 3))
        alpha = image.getchannel("A")
        box = alpha.getbbox()
        if box is None:
            raise ValueError("la imagen no contiene una firma visible")
        image = image.crop(box)
        image.thumbnail((1600, 600), Image.Resampling.LANCZOS)
        target.parent.mkdir(parents=True, exist_ok=True)
        image.save(target, "PNG", optimize=True)
    target.chmod(0o600)
    return target
