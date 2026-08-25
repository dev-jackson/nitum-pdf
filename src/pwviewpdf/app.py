"""GTK4 / libadwaita front end.

Two rules drive the layout: the document is the interface (chrome stays out of
the way), and signing is one visible button away instead of buried under a
"Tools" menu the way Acrobat buries it.
"""

from __future__ import annotations

import threading
from pathlib import Path

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")

from gi.repository import Adw, Gdk, Gio, GLib, Gtk  # noqa: E402

from . import compat, identities, signing, state, strings as S, trust, updater  # noqa: E402
from .document import Document, PasswordRequired  # noqa: E402

ZOOM_STEPS = (0.5, 0.67, 0.8, 1.0, 1.25, 1.5, 2.0, 3.0, 4.0)
PAGE_GAP = 16
RENDER_MARGIN = 1          # pages rendered above/below the viewport
CACHE_LIMIT = 24


class PageView(Gtk.Overlay):
    """One page: a bitmap plus the rubber band used to place a signature."""

    def __init__(self, index: int, on_rect_drawn, on_rect_too_small=lambda: None):
        super().__init__()
        self.index = index
        self._on_rect_drawn = on_rect_drawn
        self._on_rect_too_small = on_rect_too_small
        self.rect: tuple[float, float, float, float] | None = None
        self.highlights: list[tuple[float, float, float, float]] = []
        self.current_highlight: tuple[float, float, float, float] | None = None
        self.selectable = False

        self.picture = Gtk.Picture(can_shrink=False)
        self.picture.add_css_class("page-shadow")
        self.set_child(self.picture)

        self.canvas = Gtk.DrawingArea()
        self.canvas.set_draw_func(self._draw)
        self.add_overlay(self.canvas)

        drag = Gtk.GestureDrag()
        drag.connect("drag-begin", self._begin)
        drag.connect("drag-update", self._update)
        drag.connect("drag-end", self._end)
        self.canvas.add_controller(drag)
        self._origin: tuple[float, float] | None = None

    def set_size(self, width: int, height: int) -> None:
        for widget in (self, self.picture, self.canvas):
            widget.set_size_request(width, height)

    def set_texture(self, texture: Gdk.Texture | None) -> None:
        self.picture.set_paintable(texture)

    def clear_rect(self) -> None:
        self.rect = None
        self.canvas.queue_draw()

    def _begin(self, _gesture, x, y):
        if self.selectable:
            self._origin = (x, y)

    def _update(self, _gesture, dx, dy):
        if self._origin is None:
            return
        x0, y0 = self._origin
        width, height = self.canvas.get_width(), self.canvas.get_height()
        clamp = lambda value, limit: max(0.0, min(value, float(limit)))
        self.rect = (clamp(x0, width), clamp(y0, height),
                     clamp(x0 + dx, width), clamp(y0 + dy, height))
        self.canvas.queue_draw()

    def _end(self, gesture, dx, dy):
        if self._origin is None:
            return
        self._update(gesture, dx, dy)
        self._origin = None
        big_enough = self.rect and abs(self.rect[2] - self.rect[0]) > 24 \
            and abs(self.rect[3] - self.rect[1]) > 12
        if big_enough:
            self._on_rect_drawn(self)
        else:
            self.clear_rect()
            self._on_rect_too_small()

    def set_highlights(self, rects, current=None) -> None:
        self.highlights = list(rects)
        self.current_highlight = current
        self.canvas.queue_draw()

    def _draw(self, _area, cairo, _width, _height):
        for rect in self.highlights:
            x1, y1, x2, y2 = rect
            is_current = rect == self.current_highlight
            cairo.set_source_rgba(*( (1.0, 0.55, 0.0, 0.45) if is_current
                                     else (1.0, 0.85, 0.15, 0.38) ))
            cairo.rectangle(x1 - 1, y1 - 1, x2 - x1 + 2, y2 - y1 + 2)
            cairo.fill()
        if not self.rect:
            return
        x1, y1, x2, y2 = self.rect
        x, y, w, h = min(x1, x2), min(y1, y2), abs(x2 - x1), abs(y2 - y1)
        cairo.set_source_rgba(0.21, 0.52, 0.89, 0.20)
        cairo.rectangle(x, y, w, h)
        cairo.fill_preserve()
        cairo.set_source_rgba(0.21, 0.52, 0.89, 0.95)
        cairo.set_line_width(2)
        cairo.stroke()
        if w > 90 and h > 24:
            cairo.select_font_face("Sans")
            cairo.set_font_size(13)
            extents = cairo.text_extents(S.SIGN_HERE)
            cairo.move_to(x + (w - extents.width) / 2, y + (h + extents.height) / 2)
            cairo.show_text(S.SIGN_HERE)


class ViewerWindow(Adw.ApplicationWindow):
    def __init__(self, app, path: str | None = None):
        super().__init__(application=app, title=S.APP_NAME,
                         default_width=980, default_height=1040)
        self.document: Document | None = None
        self.path: Path | None = None
        self.scale = 1.0
        self.fit_width = True
        self.signing_mode = False
        self.pending: tuple[int, tuple[float, float, float, float]] | None = None
        self._pages: list[PageView] = []
        self._cache: dict[tuple[int, int], Gdk.Texture] = {}
        self._current_page = 0
        self._hits: list = []
        self._hit_index = 0

        self.toasts = Adw.ToastOverlay()
        self.banner = compat.Banner(S.SIGN_BANNER, S.SIGN_BANNER_INVISIBLE)
        self.banner.connect_clicked(self._open_sign_dialog)

        # Whether a document is signed is the first thing you should know about it,
        # not something you go looking for in a menu.
        self.signature_banner = compat.Banner(button_label=S.BANNER_DETAILS)
        self.signature_banner.connect_clicked(self.show_verification)

        self.pages_box = Gtk.Box(
            orientation=Gtk.Orientation.VERTICAL, spacing=PAGE_GAP,
            halign=Gtk.Align.CENTER, margin_top=PAGE_GAP, margin_bottom=PAGE_GAP,
        )
        self.scroller = Gtk.ScrolledWindow(vexpand=True, hexpand=True)
        self.scroller.set_child(self.pages_box)
        self.scroller.get_vadjustment().connect("value-changed", self._on_scroll)
        self.scroller.connect("notify::width", lambda *_: self._on_resize())

        self.empty = Adw.StatusPage(
            icon_name="document-open-symbolic",
            title=S.EMPTY_TITLE, description=S.EMPTY_BODY,
        )
        open_button = Gtk.Button(label=S.OPEN, halign=Gtk.Align.CENTER,
                                 css_classes=["suggested-action", "pill"])
        open_button.connect("clicked", lambda *_: self.choose_file())
        self.empty.set_child(open_button)

        self.stack = Gtk.Stack(transition_type=Gtk.StackTransitionType.CROSSFADE)
        self.stack.add_named(self.empty, "empty")
        self.stack.add_named(self.scroller, "pages")

        self.search_bar, self.search_entry = self._build_search()

        content = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        content.append(self.signature_banner)
        content.append(self.banner)
        content.append(self.search_bar)
        content.append(self.stack)
        self.toasts.set_child(content)

        self.set_content(compat.toolbar_view(self._build_header(), self.toasts))

        self._install_actions()
        self._install_css()
        self._install_drop_target()
        self._update_chrome()

        if path:
            self.open_document(Path(path))

    # -- chrome ------------------------------------------------------------

    def _build_header(self) -> Adw.HeaderBar:
        header = Adw.HeaderBar()

        open_button = Gtk.Button(icon_name="document-open-symbolic",
                                 tooltip_text=S.OPEN_TOOLTIP)
        open_button.connect("clicked", lambda *_: self.choose_file())
        header.pack_start(open_button)

        self.page_entry = Gtk.Entry(
            width_chars=3, max_width_chars=3, xalign=0.5, hexpand=False,
            input_purpose=Gtk.InputPurpose.DIGITS,
        )
        self.page_entry.connect("activate", self._on_page_entry)
        self.page_total = Gtk.Label(css_classes=["dim-label"])
        self.pager = Gtk.Box(spacing=6, valign=Gtk.Align.CENTER, hexpand=False)
        self.pager.append(self.page_entry)
        self.pager.append(self.page_total)
        header.pack_start(self.pager)

        zoom = Gtk.Box(css_classes=["linked"])
        for icon, delta, tip in (
            ("zoom-out-symbolic", -1, S.ZOOM_OUT),
            ("zoom-in-symbolic", 1, S.ZOOM_IN),
        ):
            button = Gtk.Button(icon_name=icon, tooltip_text=tip)
            button.connect("clicked", lambda _b, d=delta: self.zoom(d))
            zoom.append(button)
        self.zoom_label = Gtk.Button(label="100%", tooltip_text=S.FIT_WIDTH,
                                     css_classes=["flat"], width_request=64)
        self.zoom_label.connect("clicked", lambda *_: self.apply_fit_width())
        zoom.append(self.zoom_label)
        self.zoom_box = zoom
        header.pack_start(zoom)

        self.sign_button = Gtk.Button(label=S.SIGN, tooltip_text=S.SIGN_TOOLTIP,
                                      css_classes=["suggested-action"])
        self.sign_button.connect("clicked", lambda *_: self.start_signing())
        header.pack_end(self.sign_button)

        menu = Gio.Menu()
        menu.append(S.VERIFY, "win.verify")
        menu.append(S.FIT_WIDTH, "win.fit-width")
        menu.append(S.COPY_TEXT, "win.copy-text")
        menu.append(S.IMPORT_IDENTITY, "win.import-identity")
        menu.append(S.CHECK_UPDATES, "win.check-updates")
        self.menu_button = Gtk.MenuButton(icon_name="open-menu-symbolic", menu_model=menu)
        header.pack_end(self.menu_button)

        self.search_button = Gtk.ToggleButton(icon_name="edit-find-symbolic")
        self.search_button.connect("toggled", self._toggle_search)
        header.pack_end(self.search_button)
        return header

    def _build_search(self):
        entry = Gtk.SearchEntry(placeholder_text=S.SEARCH_PLACEHOLDER, width_chars=32)
        entry.connect("activate", self._on_search)
        entry.connect("search-changed", self._on_search)
        entry.connect("next-match", lambda *_: self.step_hit(1))
        entry.connect("previous-match", lambda *_: self.step_hit(-1))

        self.hit_label = Gtk.Label(css_classes=["dim-label"], width_chars=8)
        previous = Gtk.Button(icon_name="go-up-symbolic", tooltip_text=S.SEARCH_PREV)
        previous.connect("clicked", lambda *_: self.step_hit(-1))
        following = Gtk.Button(icon_name="go-down-symbolic", tooltip_text=S.SEARCH_NEXT)
        following.connect("clicked", lambda *_: self.step_hit(1))
        arrows = Gtk.Box(css_classes=["linked"])
        arrows.append(previous)
        arrows.append(following)

        row = Gtk.Box(spacing=8)
        row.append(entry)
        row.append(self.hit_label)
        row.append(arrows)

        bar = Gtk.SearchBar(child=row, show_close_button=False)
        bar.connect_entry(entry)
        return bar, entry

    def _install_css(self) -> None:
        provider = Gtk.CssProvider()
        compat.load_css(provider, """
        .page-shadow { background: #ffffff;
            box-shadow: 0 1px 3px rgba(0,0,0,.28), 0 6px 18px rgba(0,0,0,.14); }
        .success { color: #1a7f37; }
        .warning { color: #9a6700; }
        .error   { color: #b42318; }
        banner.success > revealer > widget { background: #d3f2dc; color: #10331d; }
        banner.warning > revealer > widget { background: #fdf0c8; color: #3d2c00; }
        banner.error   > revealer > widget { background: #f9d7d3; color: #4a100a; }
        .banner-fallback { background: alpha(currentColor, .08); }
        revealer.success .banner-fallback { background: #d3f2dc; color: #10331d; }
        revealer.warning .banner-fallback { background: #fdf0c8; color: #3d2c00; }
        revealer.error   .banner-fallback { background: #f9d7d3; color: #4a100a; }
        """)
        Gtk.StyleContext.add_provider_for_display(
            Gdk.Display.get_default(), provider,
            Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION,
        )

    def _install_actions(self) -> None:
        for name, handler, accels in (
            ("open", lambda *_: self.choose_file(), ["<Control>o"]),
            ("sign", lambda *_: self.start_signing(), ["<Control>f"]),
            ("verify", lambda *_: self.show_verification(), ["<Control>v"]),
            ("fit-width", lambda *_: self.apply_fit_width(), ["<Control>0"]),
            ("import-identity", lambda *_: self.choose_identity_file(), []),
            ("copy-text", lambda *_: self.copy_page_text(), ["<Control>c"]),
            ("check-updates", lambda *_: self.check_for_updates(manual=True), []),
            ("zoom-in", lambda *_: self.zoom(1), ["<Control>plus", "<Control>equal"]),
            ("zoom-out", lambda *_: self.zoom(-1), ["<Control>minus"]),
            ("search", lambda *_: self.search_button.set_active(True), ["<Control>k"]),
            ("escape", lambda *_: self.cancel_signing(), ["Escape"]),
        ):
            action = Gio.SimpleAction.new(name, None)
            action.connect("activate", handler)
            self.add_action(action)
            if accels:
                self.get_application().set_accels_for_action(f"win.{name}", accels)

    def check_for_updates(self, manual: bool = False) -> None:
        if manual:
            self.toast(S.UPDATE_CHECKING)

        def work():
            try:
                release = updater.latest_release()
                GLib.idle_add(self._update_result, release, manual)
            except Exception as exc:
                if manual:
                    GLib.idle_add(self.toast, S.UPDATE_FAILED.format(reason=str(exc)))

        threading.Thread(target=work, daemon=True).start()

    def _update_result(self, release, manual: bool) -> None:
        if release is None:
            if manual:
                self.toast(S.UPDATE_LATEST)
            return
        compat.alert(
            self,
            S.UPDATE_AVAILABLE_TITLE.format(version=release.version),
            S.UPDATE_AVAILABLE_BODY,
            [("later", S.CANCEL), ("install", S.UPDATE_INSTALL)],
            on_response=lambda response: response == "install"
            and self._download_update(release),
            default_response="install",
        )

    def _download_update(self, release) -> None:
        self.toast(S.UPDATE_DOWNLOADING.format(version=release.version))

        def work():
            try:
                package = updater.download_and_verify(release)
                GLib.idle_add(self._install_update, package)
            except Exception as exc:
                GLib.idle_add(self.toast, S.UPDATE_FAILED.format(reason=str(exc)))

        threading.Thread(target=work, daemon=True).start()

    def _install_update(self, package: Path) -> None:
        self.toast(S.UPDATE_READY)
        try:
            updater.install_deb(package)
        except Exception as exc:
            self.toast(S.UPDATE_INSTALL_FAILED.format(reason=str(exc)))

    def _install_drop_target(self) -> None:
        target = Gtk.DropTarget.new(Gio.File, Gdk.DragAction.COPY)
        target.connect("drop", lambda _t, file, *_: self._drop(file))
        self.add_controller(target)

    def _drop(self, file: Gio.File) -> bool:
        path = file.get_path()
        if path and path.lower().endswith(".pdf"):
            self.open_document(Path(path))
            return True
        return False

    # -- document ----------------------------------------------------------

    def choose_file(self) -> None:
        compat.open_file(self, S.FILE_DIALOG_TITLE,
                         lambda path: self.open_document(Path(path)),
                         patterns=("*.pdf",), filter_name=S.FILE_FILTER)

    def open_document(self, path: Path, password: str | None = None,
                      retry: bool = False) -> None:
        try:
            document = Document(path, password=password)
        except PasswordRequired:
            self._ask_password(path, wrong=retry)
            return
        except Exception as exc:
            self.show_error(S.ERROR_TITLE, signing.friendly_error(exc))
            return

        if self.document:
            self.document.close()
        self.document = document
        self.path = path
        self.cancel_signing()
        self._cache.clear()
        self.set_title(path.name)
        self.stack.set_visible_child_name("pages")
        self._build_pages()
        self._update_chrome()
        self._refresh_signature_banner()

    def _ask_password(self, path: Path, wrong: bool = False) -> None:
        entry = Gtk.PasswordEntry(show_peek_icon=True, activates_default=True,
                                  margin_top=6)
        compat.alert(
            self,
            S.PASSWORD_WRONG if wrong else S.PASSWORD_TITLE,
            S.PASSWORD_BODY.format(name=path.name),
            [("cancel", S.CANCEL), ("open", S.OPEN_ACTION)],
            on_response=lambda response: response == "open"
            and self.open_document(path, entry.get_text(), retry=True),
            extra_child=entry, default_response="open",
        )

    def _refresh_signature_banner(self) -> None:
        """Verification touches the disk and the trust store: keep it off the UI thread."""
        self.signature_banner.set_revealed(False)
        if self.path is None:
            return
        path = self.path

        def work():
            try:
                statuses = signing.verify(path)
            except Exception:
                statuses = []
            GLib.idle_add(self._show_signature_banner, path, statuses)

        threading.Thread(target=work, daemon=True).start()

    def _show_signature_banner(self, path: Path, statuses: list) -> None:
        if self.path != path or not statuses:
            return
        if any(not s.intact for s in statuses):
            title, css = S.BANNER_BROKEN, "error"
        elif any(not s.trusted for s in statuses):
            title, css = S.BANNER_UNVERIFIED, "warning"
        else:
            title, css = S.BANNER_ALL_GOOD, "success"
        self.signature_banner.set_title(title)
        for name in ("success", "warning", "error"):
            self.signature_banner.remove_css_class(name)
            if self.signature_banner._native is not None:
                self.signature_banner._native.remove_css_class(name)
        self.signature_banner.add_css_class(css)
        if self.signature_banner._native is not None:
            self.signature_banner._native.add_css_class(css)
        self.signature_banner.set_revealed(True)

    def _build_pages(self) -> None:
        while (child := self.pages_box.get_first_child()) is not None:
            self.pages_box.remove(child)
        self._pages = []
        for index in range(self.document.page_count):
            view = PageView(index, self._on_rect_drawn,
                            lambda: self.toast(S.SIGN_TOO_SMALL))
            self._pages.append(view)
            self.pages_box.append(view)
        self._apply_scale(self._preferred_scale())
        self.page_total.set_text(S.PAGE_OF.format(total=self.document.page_count))
        self.page_entry.set_text("1")

    def _preferred_scale(self) -> float:
        if not self.fit_width or self.document is None:
            return self.scale
        width = max(320, self.scroller.get_width() - 48)
        return self.document.scale_to_fit_width(0, width)

    def _apply_scale(self, scale: float) -> None:
        self.scale = scale
        if hasattr(self, "zoom_label"):
            self.zoom_label.set_label(S.ZOOM_LABEL.format(percent=round(scale * 100)))
        for view in self._pages:
            width, height = self.document.geometry(view.index).image_size(scale)
            view.set_size(width, height)
            view.set_texture(None)
            view.clear_rect()
        self._cache.clear()
        GLib.idle_add(self._render_visible)
        GLib.idle_add(self._paint_hits)

    def _on_resize(self) -> None:
        if self.document and self.fit_width:
            new_scale = self._preferred_scale()
            if abs(new_scale - self.scale) > 0.01:
                self._apply_scale(new_scale)

    def _visible_range(self) -> tuple[int, int]:
        adjustment = self.scroller.get_vadjustment()
        top, bottom = adjustment.get_value(), adjustment.get_value() + adjustment.get_page_size()
        offset, first, last = PAGE_GAP, 0, 0
        for view in self._pages:
            height = view.get_size_request().height
            if offset + height >= top and not last:
                first = view.index
                last = view.index
            if offset <= bottom:
                last = view.index
            offset += height + PAGE_GAP
        return max(0, first - RENDER_MARGIN), min(len(self._pages) - 1, last + RENDER_MARGIN)

    def _on_scroll(self, _adjustment) -> None:
        first, last = self._visible_range()
        if first != self._current_page:
            self._current_page = first
            if not self.page_entry.has_focus():
                self.page_entry.set_text(str(first + 1))
        self._render_visible()

    def _render_visible(self) -> bool:
        if self.document is None:
            return False
        first, last = self._visible_range()
        key_scale = int(self.scale * 100)
        for index in range(first, last + 1):
            key = (index, key_scale)
            texture = self._cache.get(key)
            if texture is None:
                page = self.document.render(index, self.scale)
                texture = Gdk.MemoryTexture.new(
                    page.width, page.height, Gdk.MemoryFormat.R8G8B8A8,
                    GLib.Bytes.new(page.data), page.stride,
                )
                if len(self._cache) > CACHE_LIMIT:
                    self._cache.pop(next(iter(self._cache)))
                self._cache[key] = texture
            self._pages[index].set_texture(texture)
        return False

    def zoom(self, direction: int) -> None:
        if self.document is None:
            return
        self.fit_width = False
        closest = min(range(len(ZOOM_STEPS)), key=lambda i: abs(ZOOM_STEPS[i] - self.scale))
        self._apply_scale(ZOOM_STEPS[max(0, min(closest + direction, len(ZOOM_STEPS) - 1))])

    def apply_fit_width(self) -> None:
        self.fit_width = True
        if self.document:
            self._apply_scale(self._preferred_scale())

    def _on_page_entry(self, entry) -> None:
        try:
            target = int(entry.get_text()) - 1
        except ValueError:
            return
        self.scroll_to_page(target)

    def scroll_to_page(self, index: int) -> None:
        if self.document is None:
            return
        index = max(0, min(index, self.document.page_count - 1))
        offset = PAGE_GAP
        for view in self._pages[:index]:
            offset += view.get_size_request().height + PAGE_GAP
        self.scroller.get_vadjustment().set_value(offset)

    def _toggle_search(self, button) -> None:
        self.search_bar.set_search_mode(button.get_active())
        if button.get_active():
            self.search_entry.grab_focus()

    def _on_search(self, entry) -> None:
        if self.document is None:
            return
        needle = entry.get_text().strip()
        self._hits = self.document.search(needle) if len(needle) >= 2 else []
        self._hit_index = 0
        self._paint_hits()
        if needle and not self._hits and len(needle) >= 2:
            self.hit_label.set_text("0")
        elif self._hits:
            self.scroll_to_page(self._hits[0].page)

    def step_hit(self, direction: int) -> None:
        if not self._hits:
            return
        self._hit_index = (self._hit_index + direction) % len(self._hits)
        self._paint_hits()
        self.scroll_to_page(self._hits[self._hit_index].page)

    def _paint_hits(self) -> None:
        if self.document is None:
            return
        current = self._hits[self._hit_index] if self._hits else None
        for view in self._pages:
            geometry = self.document.geometry(view.index)
            rects, current_rect = [], None
            for position, hit in enumerate(self._hits):
                if hit.page != view.index:
                    continue
                rect = geometry.rect_from_pdf(hit.rect, self.scale)
                rects.append(rect)
                if current is not None and position == self._hit_index:
                    current_rect = rect
            view.set_highlights(rects, current_rect)
        self.hit_label.set_text(
            S.SEARCH_HITS.format(current=self._hit_index + 1, total=len(self._hits))
            if self._hits else ""
        )

    # -- signing -----------------------------------------------------------

    def start_signing(self) -> None:
        if self.document is None:
            return
        if self.signing_mode:                 # second press: invisible signature
            self._open_sign_dialog()
            return
        self.signing_mode = True
        self.pending = None
        for view in self._pages:
            view.selectable = True
        self.banner.set_revealed(True)

    def cancel_signing(self) -> None:
        self.signing_mode = False
        self.pending = None
        self.banner.set_revealed(False)
        for view in self._pages:
            view.selectable = False
            view.clear_rect()

    def _on_rect_drawn(self, view: PageView) -> None:
        for other in self._pages:
            if other is not view:
                other.clear_rect()
        self.pending = (view.index, view.rect)
        self._open_sign_dialog()

    def _open_sign_dialog(self) -> None:
        available = identities.discover()
        if not available:
            self.show_error(
                S.NO_IDENTITIES_TITLE,
                S.NO_IDENTITIES_BODY.format(path=identities.IDENTITY_DIR),
            )
            return
        if self.pending:
            where = S.SIGN_WHERE_PAGE.format(page=self.pending[0] + 1)
        else:
            where = S.SIGN_WHERE_INVISIBLE
        SignDialog(self, available, where).present(self)

    def perform_signature(self, identity, secret: str, reason: str,
                          location: str, strong: bool) -> None:
        assert self.path is not None
        box = None
        if self.pending:
            index, rect = self.pending
            geometry = self.document.geometry(index)
            box = geometry.clamp(geometry.rect_to_pdf(rect, self.scale))
        options = signing.SignOptions(
            page=self.pending[0] if self.pending else 0,
            box=box, reason=reason or None, location=location or None,
            want_timestamp=strong, want_ltv=strong,
        )
        target = signing.suggest_output(self.path)
        source = self.path
        self.sign_button.set_sensitive(False)
        self.toast(S.SIGN_WORKING)

        def work():
            try:
                if identity.is_token:
                    result = signing.sign_with_token(
                        source, target, identity.path, secret,
                        token_label=identity.token_label, options=options,
                    )
                else:
                    result = signing.sign_with_pkcs12(
                        source, target, identity.path, secret.encode(), options,
                    )
                GLib.idle_add(self._signing_succeeded, result)
            except Exception as exc:
                GLib.idle_add(self._signing_failed, exc)

        threading.Thread(target=work, daemon=True).start()

    def _signing_succeeded(self, result: signing.SignResult) -> None:
        self.sign_button.set_sensitive(True)
        self.cancel_signing()
        self.open_document(result.output)
        self.toast(S.SIGNED_TOAST.format(name=result.output.name))
        if result.downgrade_reason:
            self.show_error(S.DOWNGRADED_TITLE,
                            S.DOWNGRADED_BODY.format(reason=result.downgrade_reason))
        else:
            StatusDialog(result.statuses, result.level, self).present(self)

    def _signing_failed(self, exc: Exception) -> None:
        self.sign_button.set_sensitive(True)
        self.show_error(S.SIGN_FAILED_TITLE, signing.friendly_error(exc))

    def show_verification(self) -> None:
        if self.path is None:
            return
        try:
            statuses = signing.verify(self.path)
        except Exception as exc:
            self.show_error(S.ERROR_TITLE, str(exc))
            return
        if not statuses:
            self.show_error(S.NO_SIGNATURES_TITLE, S.NO_SIGNATURES_BODY)
            return
        StatusDialog(statuses, window=self).present(self)

    def copy_page_text(self) -> None:
        if self.document is None:
            return
        text = self.document.page_text(self._current_page)
        self.get_clipboard().set(text)
        self.toast(S.COPIED)

    def choose_identity_file(self) -> None:
        def imported(path: str) -> None:
            stored = identities.import_pkcs12(Path(path))
            self.toast(S.IMPORT_DONE.format(name=stored.name))

        compat.open_file(self, S.IMPORT_IDENTITY, imported,
                         patterns=("*.p12", "*.pfx"), filter_name="PKCS#12")

    # -- feedback ----------------------------------------------------------

    def toast(self, text: str) -> None:
        self.toasts.add_toast(Adw.Toast(title=text, timeout=4))

    def show_error(self, title: str, body: str) -> None:
        compat.alert(self, title, body, [("ok", S.UNDERSTOOD)])

    def _update_chrome(self) -> None:
        has_document = self.document is not None
        for widget in (self.sign_button, self.search_button):
            widget.set_sensitive(has_document)
        for widget in (self.pager, self.zoom_box):
            widget.set_visible(has_document)
        if has_document:
            self.zoom_label.set_label(S.ZOOM_LABEL.format(percent=round(self.scale * 100)))
        self.stack.set_visible_child_name("pages" if has_document else "empty")


class SignDialog:
    """One screen, four decisions, and only the first one is mandatory."""

    def __init__(self, window: ViewerWindow, available: list[identities.Identity],
                 where: str | None = None):
        self.window = window
        self.available = available
        self.dialog = compat.Dialog(S.SIGN_DIALOG_TITLE, width=460)

        self.identity_row = Adw.ComboRow(
            title=S.SIGN_IDENTITY,
            subtitle=where or "",
            model=Gtk.StringList.new([i.label for i in available]),
        )
        self.identity_row.connect("notify::selected", lambda *_: self._sync_secret_title())
        last = state.load().get("last_identity")
        for position, identity in enumerate(available):
            if identity.label == last:
                self.identity_row.set_selected(position)
                break

        self.secret_row, self._secret_text, self._set_secret_title = compat.password_row(
            available[self.identity_row.get_selected()].secret_prompt,
            on_activate=self._submit,
        )
        self.reason_row, self._reason_text = compat.entry_row(S.SIGN_REASON)
        self.location_row, self._location_text = compat.entry_row(S.SIGN_LOCATION)
        self.strong_row, self._strong_active = compat.switch_row(
            S.SIGN_STRONG, S.SIGN_STRONG_HINT, active=True,
        )

        group = Adw.PreferencesGroup()
        for row in (self.identity_row, self.secret_row, self.reason_row,
                    self.location_row, self.strong_row):
            group.add(row)

        sign_button = Gtk.Button(label=S.SIGN_BUTTON, css_classes=["suggested-action"])
        sign_button.connect("clicked", self._submit)

        header = Adw.HeaderBar(show_end_title_buttons=False)
        header.pack_end(sign_button)

        page = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12,
                       margin_top=12, margin_bottom=18, margin_start=18, margin_end=18)
        page.append(group)
        self.dialog.set_child(compat.toolbar_view(header, page))

    def present(self, parent) -> None:
        self.dialog.present(parent)

    def _sync_secret_title(self) -> None:
        self._set_secret_title(
            self.available[self.identity_row.get_selected()].secret_prompt
        )

    def _submit(self, *_args) -> None:
        identity = self.available[self.identity_row.get_selected()]
        state.remember("last_identity", identity.label)
        self.dialog.close()
        self.window.perform_signature(
            identity, self._secret_text(), self._reason_text(),
            self._location_text(), self._strong_active(),
        )


class StatusDialog:
    """Integrity and identity, always as two separate answers."""

    def __init__(self, statuses: list[signing.SignatureStatus], level: str | None = None,
                 window: "ViewerWindow | None" = None):
        self.window = window
        self.dialog = compat.Dialog(S.VERIFY_TITLE, width=520)
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=18,
                      margin_top=12, margin_bottom=18, margin_start=18, margin_end=18)
        box.append(self._summary(statuses))

        for status in statuses:
            description = S.SIGNED_AT.format(when=status.signed_at or "\u2014")
            if status.organization:
                description = S.ORGANIZATION.format(org=status.organization) + " · " + description
            group = Adw.PreferencesGroup(
                title=S.SIGNED_BY.format(signer=status.common_name),
                description=description,
            )
            group.add(self._row(
                status.intact,
                (S.INTACT_YES, S.INTACT_YES_BODY) if status.intact else (S.INTACT_NO, S.INTACT_NO_BODY),
                bad_is_error=True,
            ))
            trust_row = self._row(
                status.trusted,
                (S.TRUST_YES, S.TRUST_YES_BODY) if status.trusted else (S.TRUST_NO, S.TRUST_NO_BODY),
                bad_is_error=False,
            )
            if not status.trusted and status.certificate:
                button = Gtk.Button(label=S.TRUST_BUTTON, valign=Gtk.Align.CENTER)
                button.connect("clicked", lambda _b, st=status: self._ask_to_trust(st))
                trust_row.add_suffix(button)
            group.add(trust_row)
            box.append(group)

        if level:
            human = S.LEVEL_HUMAN.get(level, "")
            box.append(Gtk.Label(
                xalign=0, wrap=True, css_classes=["dim-label", "caption"],
                label=f"{human}\n{S.LEVEL.format(level=level)}",
            ))

        self.dialog.set_child(compat.toolbar_view(Adw.HeaderBar(), box))

    def present(self, parent) -> None:
        self.dialog.present(parent)

    @staticmethod
    def _summary(statuses: list[signing.SignatureStatus]) -> Gtk.Widget:
        broken = [s for s in statuses if not s.intact]
        unverified = [s for s in statuses if s.intact and not s.trusted]
        if broken:
            text, css, icon = S.SUMMARY_BROKEN, "error", "dialog-error-symbolic"
        elif unverified:
            text = S.SUMMARY_UNVERIFIED.format(n=len(unverified))
            css, icon = "warning", "dialog-warning-symbolic"
        else:
            text = S.SUMMARY_ALL_GOOD.format(n=len(statuses))
            css, icon = "success", "object-select-symbolic"
        row = Gtk.Box(spacing=10)
        row.append(Gtk.Image(icon_name=icon, css_classes=[css], pixel_size=20))
        row.append(Gtk.Label(label=text, xalign=0, wrap=True, css_classes=["heading"]))
        return row

    def _ask_to_trust(self, status: signing.SignatureStatus) -> None:
        parent = self.window if self.window is not None else None
        compat.alert(
            parent,
            S.TRUST_CONFIRM_TITLE.format(name=status.common_name),
            S.TRUST_CONFIRM_BODY.format(fingerprint=trust.fingerprint(status.certificate)),
            [("cancel", S.CANCEL), ("trust", S.TRUST_CONFIRM_ACTION)],
            on_response=lambda response: self._trusted(status) if response == "trust" else None,
            destructive="trust",
        )

    def _trusted(self, status) -> None:
        trust.add(status.certificate, status.common_name)
        self.dialog.close()
        if self.window is not None:
            self.window.toast(S.TRUST_DONE)
            self.window.show_verification()

    @staticmethod
    def _row(good: bool, text: tuple[str, str], bad_is_error: bool) -> Adw.ActionRow:
        title, subtitle = text
        row = Adw.ActionRow(title=title, subtitle=subtitle)
        if hasattr(row, "set_subtitle_lines"):
            row.set_subtitle_lines(3)
        if good:
            icon, css = "object-select-symbolic", "success"
        elif bad_is_error:
            icon, css = "dialog-error-symbolic", "error"
        else:
            icon, css = "dialog-warning-symbolic", "warning"
        row.add_prefix(Gtk.Image(icon_name=icon, css_classes=[css]))
        return row


class PwViewPdf(Adw.Application):
    def __init__(self, path: str | None = None):
        super().__init__(application_id="org.pwview.PdfViewer",
                         flags=Gio.ApplicationFlags.NON_UNIQUE)
        self.path = path

    def do_activate(self):
        window = ViewerWindow(self, self.path)
        window.present()
        if updater.should_check():
            GLib.timeout_add_seconds(3, lambda: (window.check_for_updates(), False)[1])


def main(argv: list[str] | None = None) -> int:
    import sys

    signing.quiet_validation_logs()
    argv = argv if argv is not None else sys.argv
    path = argv[1] if len(argv) > 1 else None
    return PwViewPdf(path).run([argv[0]])
