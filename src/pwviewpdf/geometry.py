"""Mapping between rendered-image pixels and PDF user space.

The viewer draws pixels with the origin at the top-left; PDF places content with
the origin at the bottom-left, in points (1/72 inch), and a page may declare a
/Rotate of 90, 180 or 270 that only affects display. Getting this wrong is the
classic reason a signature stamp lands in the wrong corner, so it lives here on
its own and is unit-tested without touching a real PDF.
"""

from __future__ import annotations

from dataclasses import dataclass

Rect = tuple[float, float, float, float]


@dataclass(frozen=True)
class PageGeometry:
    """A page's crop box (in PDF points) plus its display rotation."""

    x0: float
    y0: float
    x1: float
    y1: float
    rotation: int = 0

    @property
    def width(self) -> float:
        return self.x1 - self.x0

    @property
    def height(self) -> float:
        return self.y1 - self.y0

    @property
    def quarter_turns(self) -> int:
        return (self.rotation // 90) % 4

    def image_size(self, scale: float) -> tuple[int, int]:
        """Size in pixels of the rendered page at `scale`."""
        w, h = self.width * scale, self.height * scale
        if self.quarter_turns % 2:
            w, h = h, w
        return round(w), round(h)

    def point_to_pdf(self, px: float, py: float, scale: float) -> tuple[float, float]:
        """Image pixel (top-left origin) -> PDF point (bottom-left origin)."""
        ix, iy = px / scale, py / scale
        turns = self.quarter_turns
        if turns == 0:
            u, v = ix, iy
        elif turns == 1:                    # page displayed rotated 90 deg clockwise
            u, v = iy, self.height - ix
        elif turns == 2:
            u, v = self.width - ix, self.height - iy
        else:                               # 270 deg
            u, v = self.width - iy, ix
        return self.x0 + u, self.y1 - v

    def rect_to_pdf(self, rect: Rect, scale: float) -> Rect:
        """Image-pixel rectangle -> normalised PDF rectangle (x1 < x2, y1 < y2)."""
        ax, ay = self.point_to_pdf(rect[0], rect[1], scale)
        bx, by = self.point_to_pdf(rect[2], rect[3], scale)
        return (min(ax, bx), min(ay, by), max(ax, bx), max(ay, by))

    def point_from_pdf(self, x: float, y: float, scale: float) -> tuple[float, float]:
        """PDF point (bottom-left origin) -> image pixel (top-left origin)."""
        u, v = x - self.x0, self.y1 - y
        turns = self.quarter_turns
        if turns == 0:
            ix, iy = u, v
        elif turns == 1:
            ix, iy = self.height - v, u
        elif turns == 2:
            ix, iy = self.width - u, self.height - v
        else:
            ix, iy = v, self.width - u
        return ix * scale, iy * scale

    def rect_from_pdf(self, rect: Rect, scale: float) -> Rect:
        ax, ay = self.point_from_pdf(rect[0], rect[1], scale)
        bx, by = self.point_from_pdf(rect[2], rect[3], scale)
        return (min(ax, bx), min(ay, by), max(ax, bx), max(ay, by))

    def clamp(self, rect: Rect) -> Rect:
        """Keep a PDF rectangle inside the page."""
        x1, y1, x2, y2 = rect
        return (
            max(self.x0, min(x1, self.x1)), max(self.y0, min(y1, self.y1)),
            max(self.x0, min(x2, self.x1)), max(self.y0, min(y2, self.y1)),
        )
