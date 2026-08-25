"""Generate a small, text-bearing PDF to exercise the viewer (no dependencies)."""

from __future__ import annotations

import sys
from pathlib import Path

PAGES = [
    [
        (72, 760, 22, "Contrato de prestacion de servicios"),
        (72, 720, 11, "Entre PW Servicios S.L. y el cliente abajo firmante."),
        (72, 690, 11, "Primera. Objeto del contrato."),
        (72, 670, 10, "El prestador se compromete a entregar el software descrito"),
        (72, 655, 10, "en el anexo I dentro de los plazos acordados."),
        (72, 620, 11, "Segunda. Precio y forma de pago."),
        (72, 600, 10, "El importe total asciende a 12.500 EUR mas impuestos."),
        (72, 585, 10, "El pago se realizara en tres plazos iguales."),
        (72, 550, 11, "Tercera. Confidencialidad."),
        (72, 530, 10, "Ambas partes guardaran secreto de la informacion recibida."),
        (72, 200, 10, "Firma del prestador:"),
        (330, 200, 10, "Firma del cliente:"),
    ],
    [
        (72, 760, 18, "Anexo I. Alcance"),
        (72, 720, 10, "1. Visor de documentos con desplazamiento continuo."),
        (72, 700, 10, "2. Firma digital con certificado o token PKCS#11."),
        (72, 680, 10, "3. Sello de tiempo y validacion a largo plazo."),
        (72, 660, 10, "4. Verificacion clara del estado de cada firma."),
        (72, 620, 10, "Palabra de prueba para la busqueda: salamandra."),
    ],
]


def escape(text: str) -> str:
    return text.replace("\\", r"\\").replace("(", r"\(").replace(")", r"\)")


def content_stream(lines) -> bytes:
    parts = ["BT"]
    for x, y, size, text in lines:
        parts.append(f"/F1 {size} Tf 1 0 0 1 {x} {y} Tm ({escape(text)}) Tj")
    parts.append("ET")
    return "\n".join(parts).encode("latin-1")


def build(path: Path) -> Path:
    objects: list[bytes] = []

    def add(body: bytes) -> int:
        objects.append(body)
        return len(objects)

    font = add(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica "
               b"/Encoding /WinAnsiEncoding >>")
    page_ids, content_ids = [], []
    for lines in PAGES:
        stream = content_stream(lines)
        content_ids.append(add(
            b"<< /Length " + str(len(stream)).encode() + b" >>\nstream\n"
            + stream + b"\nendstream"
        ))
        page_ids.append(None)

    pages_id = len(objects) + len(PAGES) + 1
    for index, content_id in enumerate(content_ids):
        page_ids[index] = add(
            f"<< /Type /Page /Parent {pages_id} 0 R /MediaBox [0 0 595 842] "
            f"/Resources << /Font << /F1 {font} 0 R >> >> "
            f"/Contents {content_id} 0 R >>".encode()
        )
    kids = " ".join(f"{pid} 0 R" for pid in page_ids)
    pages = add(f"<< /Type /Pages /Count {len(page_ids)} /Kids [{kids}] >>".encode())
    assert pages == pages_id, (pages, pages_id)
    catalog = add(f"<< /Type /Catalog /Pages {pages} 0 R >>".encode())

    out = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
    offsets = [0]
    for number, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out += f"{number} 0 obj\n".encode() + body + b"\nendobj\n"

    xref_at = len(out)
    out += f"xref\n0 {len(objects) + 1}\n".encode()
    out += b"0000000000 65535 f \n"
    for offset in offsets[1:]:
        out += f"{offset:010d} 00000 n \n".encode()
    out += (f"trailer\n<< /Size {len(objects) + 1} /Root {catalog} 0 R >>\n"
            f"startxref\n{xref_at}\n%%EOF\n").encode()

    path.write_bytes(bytes(out))
    return path


if __name__ == "__main__":
    target = Path(sys.argv[1] if len(sys.argv) > 1 else "contrato.pdf")
    print(build(target))
