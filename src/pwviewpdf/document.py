"""Thin wrapper over pdfium: page geometry, rendering and text search."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import pypdfium2 as pdfium

from .geometry import PageGeometry, Rect


@dataclass
class RenderedPage:
    """An RGBA bitmap ready to be handed to the toolkit.

    Always four channels: pdfium hands back three for fully opaque pages, and a
    3-byte stride makes Gdk.MemoryTexture refuse the buffer outright.
    """

    data: bytes
    width: int
    height: int
    stride: int
    scale: float

    @property
    def bytes_per_pixel(self) -> int:
        return self.stride // self.width


@dataclass
class SearchHit:
    """One occurrence, as a rectangle in PDF points."""

    page: int
    rect: Rect


class PasswordRequired(Exception):
    """The file is encrypted: we need a password before anything can be shown."""


class Document:
    def __init__(self, path: str | Path, password: str | None = None):
        self.path = Path(path)
        try:
            self._pdf = pdfium.PdfDocument(str(self.path), password=password)
        except pdfium.PdfiumError as exc:
            if "password" in str(exc).lower():
                raise PasswordRequired(str(exc)) from exc
            raise
        # Without a form environment pdfium silently skips widget annotations,
        # which is exactly what a visible signature is: the stamp would be in
        # the file but invisible on screen.
        try:
            self._pdf.init_forms()
        except Exception:
            pass
        self._geometry: dict[int, PageGeometry] = {}

    def close(self) -> None:
        self._pdf.close()

    def __len__(self) -> int:
        return len(self._pdf)

    @property
    def page_count(self) -> int:
        return len(self._pdf)

    def geometry(self, index: int) -> PageGeometry:
        cached = self._geometry.get(index)
        if cached is not None:
            return cached
        page = self._pdf[index]
        box = page.get_cropbox() or page.get_mediabox()
        x0, y0, x1, y1 = box
        geo = PageGeometry(
            x0=min(x0, x1), y0=min(y0, y1), x1=max(x0, x1), y1=max(y0, y1),
            rotation=page.get_rotation(),
        )
        self._geometry[index] = geo
        return geo

    def render(self, index: int, scale: float) -> RenderedPage:
        bitmap = self._pdf[index].render(
            scale=scale, rev_byteorder=True,
            force_bitmap_format=pdfium.raw.FPDFBitmap_BGRA,
        )
        try:
            return RenderedPage(
                data=bytes(bitmap.buffer), width=bitmap.width,
                height=bitmap.height, stride=bitmap.stride, scale=scale,
            )
        finally:
            bitmap.close()

    def scale_to_fit_width(self, index: int, pixels: int) -> float:
        geo = self.geometry(index)
        width = geo.height if geo.quarter_turns % 2 else geo.width
        return max(0.1, min(pixels / width, 8.0))

    def search(self, needle: str, limit: int = 500) -> list[SearchHit]:
        """Every occurrence of `needle`, with the box to highlight."""
        needle = needle.strip()
        if not needle:
            return []
        hits: list[SearchHit] = []
        for index in range(len(self._pdf)):
            textpage = self._pdf[index].get_textpage()
            try:
                searcher = textpage.search(needle, match_case=False)
                try:
                    while (match := searcher.get_next()) is not None:
                        char_index, char_count = match
                        hits.append(SearchHit(
                            page=index,
                            rect=self._match_box(textpage, char_index, char_count),
                        ))
                        if len(hits) >= limit:
                            return hits
                finally:
                    searcher.close()
            finally:
                textpage.close()
        return hits

    @staticmethod
    def _match_box(textpage, char_index: int, char_count: int) -> Rect:
        """Union of the rectangles pdfium reports for one match."""
        boxes = [textpage.get_rect(i)
                 for i in range(textpage.count_rects(char_index, char_count))]
        if not boxes:
            return (0.0, 0.0, 0.0, 0.0)
        lefts, bottoms, rights, tops = zip(*boxes)
        return (min(lefts), min(bottoms), max(rights), max(tops))

    def pages_with(self, needle: str) -> list[int]:
        seen: list[int] = []
        for hit in self.search(needle):
            if hit.page not in seen:
                seen.append(hit.page)
        return seen

    def page_text(self, index: int) -> str:
        textpage = self._pdf[index].get_textpage()
        try:
            return textpage.get_text_bounded()
        finally:
            textpage.close()
