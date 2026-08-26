"""Render the UI to PNG so its look can be reviewed without a human at the screen.

Usage: python scripts/screenshot.py <sample.pdf> <out-dir> [scene ...]
Scenes: empty, viewer, narrow, signature-center, placing, sign, status, search, dark
"""

from __future__ import annotations

import sys
from pathlib import Path

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Adw, Gdk, GLib, Gsk, Gtk  # noqa: E402

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pwviewpdf import appearances, identities, signing  # noqa: E402
from pwviewpdf.app import (  # noqa: E402
    CreateIdentityDialog, ImportIdentityDialog, PwViewPdf, SignDialog,
    SignatureCenterDialog, StatusDialog, ViewerWindow,
)

FAKE_IDENTITIES = [
    identities.Identity(kind="pkcs11", label="DNIe (OpenSC)",
                        path="/usr/lib/opensc-pkcs11.so", token_label="DNIe"),
    identities.Identity(kind="pkcs12", label="ada-lovelace",
                        path="/home/ada/.local/share/pw-view-pdf/identities/ada.p12"),
]

FAKE_STATUSES = [
    signing.SignatureStatus(
        field_name="Sig1", signer="Common Name: Ada Lovelace",
        common_name="Ada Lovelace", organization="PW Servicios S.L.",
        intact=True, trusted=False, signed_at="2026-08-24 15:41:02+02:00", detail="",
    ),
    signing.SignatureStatus(
        field_name="Sig2", signer="Common Name: Bruno Diaz",
        common_name="Bruno Díaz", organization="Cliente",
        intact=True, trusted=True, signed_at="2026-08-24 16:02:55+02:00", detail="",
    ),
]


def capture(window: Gtk.Window, out: Path) -> None:
    width, height = window.get_width(), window.get_height()
    paintable = Gtk.WidgetPaintable.new(window)
    snapshot = Gtk.Snapshot.new()
    paintable.snapshot(snapshot, width, height)
    node = snapshot.to_node()
    if node is None:
        raise RuntimeError("nothing to render yet")

    renderer = window.get_native().get_renderer()
    if renderer is None:
        renderer = Gsk.CairoRenderer.new()
        renderer.realize_for_display(Gdk.Display.get_default())
    renderer.render_texture(node, None).save_to_png(str(out))
    print("wrote", out)


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    signed = next((Path(a.split("=", 1)[1]) for a in sys.argv[1:]
                   if a.startswith("--signed=")), None)
    appearance = next((Path(a.split("=", 1)[1]) for a in sys.argv[1:]
                       if a.startswith("--appearance=")), None)
    if appearance is not None:
        appearances.SIGNATURE_IMAGE = appearance
    sample, out_dir = Path(args[0]), Path(args[1])
    scenes = args[2:] or [
        "empty", "viewer", "signature-center", "identity-import", "identity-create",
        "placing", "sign", "status", "search", "dark-signature-center", "dark-sign",
    ]
    out_dir.mkdir(parents=True, exist_ok=True)

    app = PwViewPdf()
    steps: list = []

    def build_scene(name: str, window: ViewerWindow) -> None:
        if name == "empty":
            return
        if name in ("signed", "verified") and signed is not None:
            window.open_document(signed)
            if name == "verified":
                window.show_verification()
            return
        window.open_document(sample)
        if name == "narrow":
            window.set_default_size(640, 760)
            return
        if name in ("signature-center", "dark-signature-center", "light-signature-center"):
            SignatureCenterDialog(window).present(window)
        elif name == "identity-import":
            ImportIdentityDialog(window, Path("/home/ada/mi-identidad.p12")).present(window)
        elif name == "identity-create":
            CreateIdentityDialog(window).present(window)
        elif name == "placing":
            window.start_signing()
            page = window._pages[0]
            page.rect = (60, 330, 250, 390)
            page.canvas.queue_draw()
        elif name in ("sign", "dark-sign", "light-sign"):
            window.start_signing()
            SignDialog(window, FAKE_IDENTITIES,
                       "Aparecerá en la página 1").present(window)
        elif name == "search":
            window.search_button.set_active(True)
            window.search_entry.set_text("contrato")
            window._on_search(window.search_entry)
        elif name == "status":
            StatusDialog(FAKE_STATUSES, signing.LEVEL_LT).present(window)

    def run_next(*_args) -> bool:
        if not steps:
            app.quit()
            return False
        name = steps.pop(0)
        window = ViewerWindow(app)
        if name.startswith("dark-"):
            window.set_theme("dark", remember=False)
        elif name.startswith("light-"):
            window.set_theme("light", remember=False)
        window.present()
        build_scene(name, window)

        def shoot() -> bool:
            try:
                capture(window, out_dir / f"{name}.png")
            except Exception as exc:
                print("FAILED", name, exc)
            window.close()
            GLib.timeout_add(200, run_next)
            return False

        # A forced color-scheme change causes a second style/layout pass in GTK.
        # Capture after that pass so the image represents the settled UI.
        GLib.timeout_add(1600 if name.startswith("dark-") else 900, shoot)
        return False

    def on_activate(_app):
        steps.extend(scenes)
        GLib.timeout_add(300, run_next)

    app.connect("activate", on_activate)
    return app.run([sys.argv[0]])


if __name__ == "__main__":
    raise SystemExit(main())
