"""Making the same window work on old and new libadwaita.

Zorin OS 18 is Ubuntu 24.04 (GTK 4.14, libadwaita 1.5) and has everything.
Zorin OS 17 is Ubuntu 22.04 (GTK 4.6, libadwaita 1.1) and is missing most of
the modern widgets: Adw.Dialog, AlertDialog, ToolbarView, Banner, SwitchRow,
EntryRow, and even Gtk.FileDialog and CssProvider.load_from_string.

Rather than sprinkle version checks through the UI, every difference is handled
here and `app.py` only talks to these helpers.
"""

from __future__ import annotations

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")

from gi.repository import Adw, Gio, GLib, Gtk


def _adw_at_least(major: int, minor: int) -> bool:
    return (Adw.get_major_version(), Adw.get_minor_version()) >= (major, minor)


def _gtk_at_least(major: int, minor: int) -> bool:
    return (Gtk.get_major_version(), Gtk.get_minor_version()) >= (major, minor)


HAS_DIALOG = _adw_at_least(1, 5) and hasattr(Adw, "Dialog")
HAS_ALERT_DIALOG = _adw_at_least(1, 5) and hasattr(Adw, "AlertDialog")
HAS_MESSAGE_DIALOG = hasattr(Adw, "MessageDialog")
HAS_TOOLBAR_VIEW = hasattr(Adw, "ToolbarView")
HAS_BANNER = hasattr(Adw, "Banner")
HAS_SWITCH_ROW = hasattr(Adw, "SwitchRow")
HAS_ENTRY_ROW = hasattr(Adw, "EntryRow")
HAS_PASSWORD_ROW = hasattr(Adw, "PasswordEntryRow")
HAS_FILE_DIALOG = hasattr(Gtk, "FileDialog")

VERSIONS = (
    f"GTK {Gtk.get_major_version()}.{Gtk.get_minor_version()} · "
    f"libadwaita {Adw.get_major_version()}.{Adw.get_minor_version()}"
)


def load_css(provider: Gtk.CssProvider, css: str) -> None:
    """Three generations of the same call.

    GTK 4.12 added load_from_string. Before that it was load_from_data, whose
    PyGObject binding takes a length argument on newer GTK and refuses one on
    the version shipped with Ubuntu 22.04.
    """
    if hasattr(provider, "load_from_string"):
        provider.load_from_string(css)
        return
    data = css.encode("utf-8")
    try:
        provider.load_from_data(data, -1)
    except TypeError:
        provider.load_from_data(data)


def toolbar_view(top_bar: Gtk.Widget, content: Gtk.Widget) -> Gtk.Widget:
    if HAS_TOOLBAR_VIEW:
        view = Adw.ToolbarView()
        view.add_top_bar(top_bar)
        view.set_content(content)
        return view
    box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
    box.append(top_bar)
    content.set_vexpand(True)
    box.append(content)
    return box


class Banner(Gtk.Revealer):
    """Adw.Banner (1.4+) or a hand-rolled equivalent that looks the same."""

    def __init__(self, title: str = "", button_label: str | None = None):
        super().__init__(transition_type=Gtk.RevealerTransitionType.SLIDE_DOWN)
        self._native = Adw.Banner(title=title) if HAS_BANNER else None
        self._on_click = None

        if self._native is not None:
            if button_label:
                self._native.set_button_label(button_label)
            self._native.connect("button-clicked", self._clicked)
            self._native.set_revealed(True)
            self.set_child(self._native)
            return

        self._label = Gtk.Label(label=title, wrap=True, hexpand=True,
                                css_classes=["heading"])
        row = Gtk.Box(spacing=12, margin_top=8, margin_bottom=8,
                      margin_start=12, margin_end=12, css_classes=["banner-fallback"])
        row.append(self._label)
        if button_label:
            button = Gtk.Button(label=button_label, valign=Gtk.Align.CENTER)
            button.connect("clicked", self._clicked)
            row.append(button)
        self.set_child(row)

    def _clicked(self, *_args):
        if self._on_click:
            self._on_click()

    def connect_clicked(self, callback) -> None:
        self._on_click = callback

    def set_title(self, title: str) -> None:
        if self._native is not None:
            self._native.set_title(title)
        else:
            self._label.set_text(title)

    def set_revealed(self, revealed: bool) -> None:
        self.set_reveal_child(revealed)

    def get_revealed(self) -> bool:
        return self.get_reveal_child()


class Dialog:
    """A modal sheet on new libadwaita, a small modal window on old ones."""

    def __init__(self, title: str, width: int = 460):
        self.title = title
        if HAS_DIALOG:
            self._impl = Adw.Dialog(title=title, content_width=width)
        else:
            self._impl = Adw.Window(title=title, modal=True, default_width=width,
                                    default_height=1, hide_on_close=False)

    @property
    def widget(self):
        return self._impl

    def set_child(self, child: Gtk.Widget) -> None:
        self._impl.set_content(child) if isinstance(self._impl, Adw.Window) \
            else self._impl.set_child(child)

    def present(self, parent: Gtk.Window) -> None:
        if HAS_DIALOG:
            self._impl.present(parent)
        else:
            self._impl.set_transient_for(parent)
            self._impl.present()

    def close(self) -> None:
        self._impl.close()


def alert(parent: Gtk.Window, heading: str, body: str,
          responses: list[tuple[str, str]] | None = None,
          on_response=None, extra_child: Gtk.Widget | None = None,
          default_response: str | None = None,
          destructive: str | None = None) -> None:
    """One confirmation dialog, whichever generation of the API is available."""
    responses = responses or [("ok", "Entendido")]

    if HAS_ALERT_DIALOG:
        dialog = Adw.AlertDialog(heading=heading, body=body)
        present = lambda: dialog.present(parent)
    elif HAS_MESSAGE_DIALOG:
        dialog = Adw.MessageDialog(transient_for=parent, modal=True,
                                   heading=heading, body=body)
        present = dialog.present
    else:
        _legacy_alert(parent, heading, body, responses, on_response,
                      extra_child, default_response)
        return

    for key, label in responses:
        dialog.add_response(key, label)
    if extra_child is not None:
        dialog.set_extra_child(extra_child)
    if default_response:
        dialog.set_default_response(default_response)
    if destructive:
        dialog.set_response_appearance(destructive, Adw.ResponseAppearance.DESTRUCTIVE)
    if on_response is not None:
        dialog.connect("response", lambda _d, response: on_response(response))
    present()


def _legacy_alert(parent, heading, body, responses, on_response, extra_child,
                  default_response) -> None:
    """libadwaita 1.1 (Ubuntu 22.04): plain GTK window, same shape of API."""
    window = Gtk.Window(transient_for=parent, modal=True, title=heading,
                        default_width=420, resizable=False)
    box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12, margin_top=18,
                  margin_bottom=18, margin_start=18, margin_end=18)
    box.append(Gtk.Label(label=heading, wrap=True, xalign=0, css_classes=["title-4"]))
    box.append(Gtk.Label(label=body, wrap=True, xalign=0))
    if extra_child is not None:
        box.append(extra_child)

    buttons = Gtk.Box(spacing=8, halign=Gtk.Align.END)
    for key, label in responses:
        button = Gtk.Button(label=label)
        if key == default_response:
            button.add_css_class("suggested-action")

        def clicked(_button, key=key):
            window.close()
            if on_response is not None:
                on_response(key)

        button.connect("clicked", clicked)
        buttons.append(button)
    box.append(buttons)
    window.set_child(box)
    window.present()


def switch_row(title: str, subtitle: str, active: bool = True):
    """Returns (row, getter). Adw.SwitchRow needs 1.4."""
    if HAS_SWITCH_ROW:
        row = Adw.SwitchRow(title=title, subtitle=subtitle, active=active)
        return row, row.get_active
    row = Adw.ActionRow(title=title, subtitle=subtitle)
    switch = Gtk.Switch(active=active, valign=Gtk.Align.CENTER)
    row.add_suffix(switch)
    row.set_activatable_widget(switch)
    return row, switch.get_active


def entry_row(title: str):
    """Returns (row, getter)."""
    if HAS_ENTRY_ROW:
        row = Adw.EntryRow(title=title)
        return row, row.get_text
    entry = Gtk.Entry(valign=Gtk.Align.CENTER, hexpand=True)
    row = Adw.ActionRow(title=title)
    row.add_suffix(entry)
    row.set_activatable_widget(entry)
    return row, entry.get_text


def password_row(title: str, on_activate=None, initial_text: str = ""):
    """Returns (row, getter, set_title)."""
    if HAS_PASSWORD_ROW:
        row = Adw.PasswordEntryRow(title=title)
        row.set_text(initial_text)
        if on_activate is not None:
            row.connect("entry-activated", lambda *_: on_activate())
        return row, row.get_text, row.set_title
    entry = Gtk.PasswordEntry(show_peek_icon=True, valign=Gtk.Align.CENTER,
                              hexpand=True)
    entry.set_text(initial_text)
    if on_activate is not None:
        entry.connect("activate", lambda *_: on_activate())
    row = Adw.ActionRow(title=title)
    row.add_suffix(entry)
    row.set_activatable_widget(entry)
    return row, entry.get_text, row.set_title


def set_entry_text(row: Gtk.Widget, text: str) -> None:
    """Set text on native and fallback entry rows without leaking implementation."""
    if hasattr(row, "set_text"):
        row.set_text(text)
        return
    child = row.get_first_child()
    while child is not None:
        if hasattr(child, "set_text"):
            child.set_text(text)
            return
        child = child.get_next_sibling()


def open_file(parent: Gtk.Window, title: str, callback, patterns=("*.pdf",),
              filter_name: str = "PDF") -> None:
    """Gtk.FileDialog is GTK 4.10; before that it is FileChooserNative."""
    file_filter = Gtk.FileFilter(name=filter_name)
    for pattern in patterns:
        file_filter.add_pattern(pattern)

    if HAS_FILE_DIALOG:
        filters = Gio.ListStore.new(Gtk.FileFilter)
        filters.append(file_filter)
        dialog = Gtk.FileDialog(title=title, filters=filters)

        def done(source, result):
            try:
                file = source.open_finish(result)
            except GLib.Error:
                return
            if file and file.get_path():
                callback(file.get_path())

        dialog.open(parent, None, done)
        return

    chooser = Gtk.FileChooserNative(title=title, transient_for=parent, modal=True,
                                    action=Gtk.FileChooserAction.OPEN)
    chooser.add_filter(file_filter)

    def responded(native, response):
        if response == Gtk.ResponseType.ACCEPT:
            file = native.get_file()
            if file and file.get_path():
                callback(file.get_path())
        native.destroy()

    chooser.connect("response", responded)
    chooser.show()
