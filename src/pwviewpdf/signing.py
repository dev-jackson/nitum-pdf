"""Signing and verification. No toolkit imports here: this module is the product.

Every write is an incremental update, so signing a document that already carries
signatures leaves those signatures valid.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass, field
from pathlib import Path

from pyhanko import stamp
from pyhanko.pdf_utils import text as pdf_text
from pyhanko.pdf_utils.images import PdfImage
from pyhanko.pdf_utils import layout
from pyhanko.pdf_utils import misc
from pyhanko.pdf_utils.incremental_writer import IncrementalPdfFileWriter
from pyhanko.pdf_utils.reader import PdfFileReader
from pyhanko.sign import fields, signers, timestamps
from pyhanko.sign.fields import SigFieldSpec, SigSeedSubFilter
from pyhanko.sign.validation import validate_pdf_signature
from pyhanko_certvalidator import ValidationContext

from . import trust
from .geometry import Rect

DEFAULT_TSA_URL = "https://freetsa.org/tsr"

# Courier is the only standard font pyHanko can lay out without embedding one:
# its metrics are uniform, so no font file has to be shipped or embedded.
STAMP_HEADING = "Firmado digitalmente por"
STAMP_TIMESTAMP_FORMAT = "%d/%m/%Y %H:%M %Z"


def _stamp_style(options: "SignOptions") -> stamp.TextStampStyle:
    if options.signature_image:
        lines = [STAMP_HEADING, "%(signer)s", "%(ts)s"]
    elif options.appearance == "minimal":
        lines = ["%(signer)s", "%(ts)s"]
    else:
        lines = [STAMP_HEADING, "%(signer)s", "%(ts)s"]
        if options.reason:
            lines.append("Motivo: %(reason)s")
        if options.location:
            lines.append("Lugar: %(location)s")
    background = PdfImage(str(options.signature_image)) if options.signature_image else None
    split_background = layout.SimpleBoxLayoutRule(
        x_align=layout.AxisAlignment.ALIGN_MID,
        y_align=layout.AxisAlignment.ALIGN_MID,
        margins=layout.Margins(left=112, right=6, top=6, bottom=6),
    ) if background else layout.SimpleBoxLayoutRule(
        x_align=layout.AxisAlignment.ALIGN_MID,
        y_align=layout.AxisAlignment.ALIGN_MID,
        margins=layout.Margins(left=5, right=5, top=5, bottom=5),
    )
    split_text = layout.SimpleBoxLayoutRule(
        x_align=layout.AxisAlignment.ALIGN_MIN,
        y_align=layout.AxisAlignment.ALIGN_MID,
        margins=layout.Margins(left=6, right=100, top=5, bottom=5),
    ) if background else None
    return stamp.TextStampStyle(
        stamp_text="\n".join(lines),
        border_width=0,
        background=background,
        background_layout=split_background,
        background_opacity=0.88 if background else 1.0,
        inner_content_layout=split_text,
        timestamp_format=STAMP_TIMESTAMP_FORMAT,
        text_box_style=pdf_text.TextBoxStyle(font_size=7, leading=9),
    )

# Ordered attempts: we always try for the strongest signature and quietly step
# down if the machine is offline, instead of failing the way Acrobat does.
LEVEL_LT = "PAdES B-LT"
LEVEL_T = "PAdES B-T"
LEVEL_B = "PAdES B-B"


class DocumentUnreadable(Exception):
    """The file is not a PDF we can parse -- damaged, truncated or not a PDF."""


@dataclass
class SignOptions:
    page: int = 0                     # 0-based
    box: Rect | None = None           # None -> invisible signature
    reason: str | None = None
    location: str | None = None
    want_timestamp: bool = True
    want_ltv: bool = True
    tsa_url: str = DEFAULT_TSA_URL
    certify: bool = False             # first signature acts as author signature
    appearance: str = "details"       # "details" or "minimal"
    signature_image: Path | None = None


@dataclass
class SignatureStatus:
    field_name: str
    signer: str            # full subject, for the details view
    common_name: str       # what a person actually reads
    organization: str | None
    intact: bool          # bytes unchanged since signing
    trusted: bool         # chains to a trusted root
    signed_at: str | None
    detail: str
    certificate: bytes = b""      # DER, so the UI can offer to trust the issuer


@dataclass
class SignResult:
    output: Path
    level: str
    statuses: list[SignatureStatus] = field(default_factory=list)
    downgrade_reason: str | None = None


# Cryptographic libraries speak in error codes; the person holding the token
# only needs to know which of the few things that can go wrong went wrong.
_FRIENDLY_ERRORS = (
    ("ckr_pin_locked", "El token está bloqueado por demasiados intentos fallidos."),
    ("ckr_pin_incorrect", "El PIN del token no es correcto."),
    ("ckr_pin_invalid", "El PIN del token no es correcto."),
    ("ckr_token_not_present", "No se encuentra el token. ¿Está conectado?"),
    ("ckr_slot_id_invalid", "No se encuentra el token. ¿Está conectado?"),
    ("no token", "No se encuentra el token. ¿Está conectado?"),
    ("could not open pkcs#11", "No se pudo cargar el controlador PKCS#11 del token."),
    ("cannot load", "No se pudo cargar el controlador PKCS#11 del token."),
    ("mac verification failed", "La contraseña del certificado no es correcta."),
    ("invalid password", "La contraseña del certificado no es correcta."),
    ("contraseña", None),          # already ours: leave it alone
)


def friendly_error(exc: Exception) -> str:
    """Turn a library error into something a person can act on."""
    raw = str(exc)
    lowered = raw.lower()
    for needle, message in _FRIENDLY_ERRORS:
        if needle in lowered:
            return message or raw
    return raw


def quiet_validation_logs() -> None:
    """An untrusted self-signed certificate is an answer, not a crash.

    pyhanko-certvalidator logs the whole traceback at ERROR level for it, which
    would fill the user's terminal with noise on a perfectly normal outcome.
    """
    for name in ("pyhanko_certvalidator", "pyhanko.sign.validation",
                 "pyhanko.sign.validation.generic_cms"):
        logging.getLogger(name).setLevel(logging.CRITICAL)


def next_field_name(pdf: str | Path) -> str:
    with open(pdf, "rb") as handle:
        used = {sig.field_name for sig in _read(handle).embedded_signatures}
    index = 1
    while f"Sig{index}" in used:
        index += 1
    return f"Sig{index}"


def signature_count(pdf: str | Path) -> int:
    with open(pdf, "rb") as handle:
        return len(_read(handle).embedded_signatures)


def _metadata(field_name: str, options: SignOptions, level: str,
              certify: bool) -> signers.PdfSignatureMetadata:
    kwargs: dict = dict(
        field_name=field_name,
        reason=options.reason or None,
        location=options.location or None,
        subfilter=SigSeedSubFilter.PADES,
        certify=certify,
    )
    if level == LEVEL_LT:
        kwargs["embed_validation_info"] = True
        kwargs["validation_context"] = ValidationContext(allow_fetching=True)
    return signers.PdfSignatureMetadata(**kwargs)


def _write_signature(signer, source: Path, target: Path, options: SignOptions,
                     level: str) -> None:
    field_name = next_field_name(source)
    certify = options.certify and signature_count(source) == 0
    timestamper = (
        timestamps.HTTPTimeStamper(options.tsa_url)
        if level in (LEVEL_LT, LEVEL_T) else None
    )
    with open(source, "rb") as infile:
        writer = IncrementalPdfFileWriter(infile)
        if options.box is not None:
            fields.append_signature_field(writer, SigFieldSpec(
                sig_field_name=field_name, box=options.box, on_page=options.page,
            ))
        pdf_signer = signers.PdfSigner(
            _metadata(field_name, options, level, certify),
            signer=signer,
            timestamper=timestamper,
            stamp_style=_stamp_style(options),
        )
        appearance = {
            "reason": options.reason or "",
            "location": options.location or "",
        }
        with open(target, "wb") as outfile:
            pdf_signer.sign_pdf(writer, output=outfile,
                                appearance_text_params=appearance)


def _levels_to_try(options: SignOptions) -> list[str]:
    if options.want_ltv and options.want_timestamp:
        return [LEVEL_LT, LEVEL_T, LEVEL_B]
    if options.want_timestamp:
        return [LEVEL_T, LEVEL_B]
    return [LEVEL_B]


def sign(signer, source: str | Path, target: str | Path,
         options: SignOptions | None = None,
         trust_roots=None) -> SignResult:
    """Sign `source` into `target`, degrading gracefully when offline."""
    options = options or SignOptions()
    source, target = Path(source), Path(target)
    downgrade: str | None = None
    last_error: Exception | None = None

    for level in _levels_to_try(options):
        try:
            _write_signature(signer, source, target, options, level)
        except Exception as exc:      # network, TSA down, no CRL/OCSP reachable
            last_error = exc
            downgrade = str(exc)
            target.unlink(missing_ok=True)
            continue
        return SignResult(
            output=target, level=level,
            statuses=verify(target, trust_roots=trust_roots),
            downgrade_reason=downgrade if level != _levels_to_try(options)[0] else None,
        )

    raise last_error if last_error else RuntimeError("signing failed")


def sign_with_pkcs12(source, target, pfx_path: str | Path, passphrase: bytes,
                     options: SignOptions | None = None, trust_roots=None) -> SignResult:
    signer = signers.SimpleSigner.load_pkcs12(
        pfx_file=str(pfx_path), passphrase=passphrase
    )
    if signer is None:
        raise ValueError("No se pudo abrir el certificado (¿contraseña incorrecta?)")
    return sign(signer, source, target, options, trust_roots)


def sign_with_token(source, target, module_path: str, pin: str,
                    token_label: str | None = None, cert_label: str | None = None,
                    options: SignOptions | None = None, trust_roots=None) -> SignResult:
    """Sign with a smartcard/USB token through PKCS#11 (OpenSC, eToken, HSM...)."""
    from pyhanko.sign.pkcs11 import (
        PKCS11SignatureConfig, PKCS11SigningContext, TokenCriteria,
    )

    config = PKCS11SignatureConfig(
        module_path=module_path,
        token_criteria=TokenCriteria(label=token_label) if token_label else None,
        cert_label=cert_label,
    )
    with PKCS11SigningContext(config, pin) as signer:
        return sign(signer, source, target, options, trust_roots)


def _read(handle) -> PdfFileReader:
    try:
        return PdfFileReader(handle)
    except (misc.PdfReadError, ValueError, OSError) as exc:
        raise DocumentUnreadable(
            "El documento está dañado o no es un PDF válido."
        ) from exc


def verify(pdf: str | Path, trust_roots=None) -> list[SignatureStatus]:
    """State of every signature. Called right after signing, never optional.

    With no explicit roots we use the issuers the user has chosen to trust.
    """
    if trust_roots is None:
        trust_roots = trust.roots() or None
    context = ValidationContext(trust_roots=trust_roots, allow_fetching=False)
    results: list[SignatureStatus] = []
    with open(pdf, "rb") as handle:
        for embedded in _read(handle).embedded_signatures:
            status = validate_pdf_signature(embedded, context)
            subject = status.signing_cert.subject.native
            results.append(SignatureStatus(
                field_name=embedded.field_name,
                signer=status.signing_cert.subject.human_friendly,
                common_name=subject.get("common_name") or status.signing_cert.subject.human_friendly,
                organization=subject.get("organization_name"),
                intact=bool(status.intact),
                trusted=bool(getattr(status, "trusted", False)),
                signed_at=str(status.signer_reported_dt) if status.signer_reported_dt else None,
                detail=status.summary(),
                certificate=status.signing_cert.dump(),
            ))
    return results


def suggest_output(source: str | Path, suffix: str = "-firmado") -> Path:
    """Never clobber the original: `contrato.pdf` -> `contrato-firmado.pdf`."""
    source = Path(source)
    candidate = source.with_name(f"{source.stem}{suffix}.pdf")
    index = 2
    while candidate.exists():
        candidate = source.with_name(f"{source.stem}{suffix}-{index}.pdf")
        index += 1
    return candidate
