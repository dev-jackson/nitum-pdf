use crate::{
    application::{
        AppearanceStore, DocumentPicker, HardwareSignRequest, HardwareTokenProvider, IdentityStore,
        OpenPdf, PdfEngine, PdfSigning, SignRequest, TextClipboard,
    },
    domain::{
        CertificationPermission, DocumentRef, HardwareToken, PadesLevel, PageBitmap, PageSize,
        PdfPasswordRequired, PdfRect, SearchHit, SignatureAppearance, SignatureKind,
        SignatureReport, SigningIdentity,
    },
};
use anyhow::{Context, Result, bail};
use cryptoki::context::{CInitializeArgs, CInitializeFlags, Pkcs11};
use der::Decode;
use image::{ImageFormat as SourceImageFormat, ImageReader};
use pdfium_render::prelude::*;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};
use underskrift::{
    Arrangement, Border, Color, CryptoSigner, DocMdpPermissions, ImageConfig, ImageFormat,
    ImageScale, PdfSigner, SignatureLayout, SignatureRect, SigningOptions, SoftwareSigner,
    TextConfig, TextLine, VisibleSignatureConfig,
    inspect::inspect_signatures,
    ltv::{
        CrlClient, DssBuilder, OcspClient, ValidationStatus, VriEntry, add_document_security_store,
        compute_vri_key,
    },
    prepare_image,
    trust::{TrustStore, TrustStoreSet},
    verify::{CryptoValidity, DetectedPadesLevel, SignatureType, SignatureVerifier},
};
use x509_cert::Certificate;

mod pkcs11;
pub mod updater;
pub use pkcs11::Pkcs11Signer;

pub struct NativeHardwareTokenProvider;

impl HardwareTokenProvider for NativeHardwareTokenProvider {
    fn detected_modules(&self) -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        if let Some(configured) = std::env::var_os("NITUM_PDF_PKCS11_MODULE") {
            candidates.push(PathBuf::from(configured));
        }
        #[cfg(target_os = "linux")]
        candidates.extend(
            [
                "/usr/lib/x86_64-linux-gnu/p11-kit-proxy.so",
                "/usr/lib/aarch64-linux-gnu/p11-kit-proxy.so",
                "/usr/lib/x86_64-linux-gnu/opensc-pkcs11.so",
                "/usr/lib/aarch64-linux-gnu/opensc-pkcs11.so",
                "/usr/lib64/p11-kit-proxy.so",
            ]
            .map(PathBuf::from),
        );
        #[cfg(target_os = "macos")]
        candidates.extend(
            [
                "/Library/OpenSC/lib/opensc-pkcs11.so",
                "/usr/local/lib/opensc-pkcs11.so",
                "/opt/homebrew/lib/opensc-pkcs11.so",
            ]
            .map(PathBuf::from),
        );
        #[cfg(target_os = "windows")]
        if let Some(program_files) = std::env::var_os("ProgramFiles") {
            candidates.push(
                PathBuf::from(program_files).join("OpenSC Project/OpenSC/pkcs11/opensc-pkcs11.dll"),
            );
        }
        candidates.sort();
        candidates.dedup();
        candidates
            .into_iter()
            .filter(|path| path.is_file())
            .collect()
    }

    fn tokens(&self, module: &Path) -> Result<Vec<HardwareToken>> {
        if !module.is_file() {
            bail!("No se encontró el módulo PKCS#11 seleccionado.");
        }
        let context = Pkcs11::new(module).context("no se pudo cargar el módulo PKCS#11")?;
        context
            .initialize(CInitializeArgs::new(CInitializeFlags::OS_LOCKING_OK))
            .context("no se pudo inicializar el módulo PKCS#11")?;
        let mut tokens = Vec::new();
        for slot in context.get_slots_with_token()? {
            let info = context.get_token_info(slot)?;
            // Some modules (notably SoftHSM) report an empty provisioning slot
            // as token-present. It cannot open a user session and must never be
            // offered as a signing identity.
            if !info.token_initialized() {
                continue;
            }
            tokens.push(HardwareToken {
                label: info.label().trim().to_owned(),
                serial: info.serial_number().trim().to_owned(),
                module_path: module.to_owned(),
                slot_id: slot.id(),
            });
        }
        tokens.sort_by_key(|token| token.label.to_lowercase());
        Ok(tokens)
    }
}

pub struct NativeIdentityStore {
    directory: PathBuf,
}

pub struct NativeAppearanceStore {
    directory: PathBuf,
}

impl NativeAppearanceStore {
    pub fn new() -> Result<Self> {
        let base = directories::BaseDirs::new().context("no se encontró la carpeta de datos")?;
        #[cfg(target_os = "linux")]
        let directory = base.data_dir().join("pw-view-pdf/appearances");
        #[cfg(not(target_os = "linux"))]
        let directory = base.data_dir().join("Nitum PDF/appearances");
        Ok(Self { directory })
    }

    pub fn at(directory: PathBuf) -> Self {
        Self { directory }
    }

    fn appearance(path: PathBuf) -> SignatureAppearance {
        SignatureAppearance {
            label: path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("Firma visual")
                .to_owned(),
            path,
        }
    }
}

impl AppearanceStore for NativeAppearanceStore {
    fn import(&self, source: &Path) -> Result<SignatureAppearance> {
        let extension = source
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_default();
        let source_format = match extension.as_str() {
            "png" => SourceImageFormat::Png,
            "jpg" | "jpeg" => SourceImageFormat::Jpeg,
            "gif" => SourceImageFormat::Gif,
            "bmp" => SourceImageFormat::Bmp,
            "tif" | "tiff" => SourceImageFormat::Tiff,
            "webp" => SourceImageFormat::WebP,
            _ => bail!("Selecciona una firma visual PNG, JPG, GIF, BMP, TIFF o WebP."),
        };
        let bytes = fs::read(source).context("no se pudo leer la firma visual")?;
        let image = ImageReader::with_format(std::io::Cursor::new(&bytes), source_format)
            .decode()
            .context("la imagen de firma no es válida")?;
        if image.width() > 8_192
            || image.height() > 8_192
            || u64::from(image.width()) * u64::from(image.height()) > 32_000_000
        {
            bail!("La firma visual supera el límite seguro de 32 megapíxeles.");
        }
        let mut normalized = std::io::Cursor::new(Vec::new());
        image
            .write_to(&mut normalized, SourceImageFormat::Png)
            .context("no se pudo normalizar la firma visual")?;
        let bytes = normalized.into_inner();
        prepare_image(&bytes, ImageFormat::Png).context("la imagen de firma no es válida")?;
        fs::create_dir_all(&self.directory)?;
        let stem = NativeIdentityStore::safe_stem(source);
        let mut index = 1_u32;
        loop {
            let suffix = if index == 1 {
                String::new()
            } else {
                format!("-{index}")
            };
            let target = self.directory.join(format!("{stem}{suffix}.png"));
            if target.exists() {
                if fs::read(&target).ok().as_deref() == Some(bytes.as_slice()) {
                    return Ok(Self::appearance(target));
                }
                index += 1;
                continue;
            }
            fs::write(&target, &bytes)?;
            #[cfg(unix)]
            fs::set_permissions(&target, {
                use std::os::unix::fs::PermissionsExt;
                fs::Permissions::from_mode(0o600)
            })?;
            return Ok(Self::appearance(target));
        }
    }

    fn list(&self) -> Result<Vec<SignatureAppearance>> {
        if !self.directory.exists() {
            return Ok(Vec::new());
        }
        let mut appearances = fs::read_dir(&self.directory)?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| {
                matches!(
                    path.extension()
                        .and_then(|value| value.to_str())
                        .map(str::to_ascii_lowercase)
                        .as_deref(),
                    Some("png")
                )
            })
            .map(Self::appearance)
            .collect::<Vec<_>>();
        appearances.sort_by_key(|appearance| appearance.label.to_lowercase());
        Ok(appearances)
    }
}

impl NativeIdentityStore {
    pub fn new() -> Result<Self> {
        let base = directories::BaseDirs::new().context("no se encontró la carpeta de datos")?;
        #[cfg(target_os = "linux")]
        let directory = base.data_dir().join("pw-view-pdf/identities");
        #[cfg(not(target_os = "linux"))]
        let directory = base.data_dir().join("Nitum PDF/identities");
        Ok(Self { directory })
    }

    pub fn at(directory: PathBuf) -> Self {
        Self { directory }
    }

    fn safe_stem(source: &Path) -> String {
        let raw = source
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("identidad");
        let clean: String = raw
            .chars()
            .filter(|value| value.is_alphanumeric() || matches!(value, ' ' | '-' | '_' | '.'))
            .collect();
        let clean = clean.trim_matches([' ', '.']);
        if clean.is_empty() {
            "identidad".to_owned()
        } else {
            clean.to_owned()
        }
    }

    fn identity(path: PathBuf) -> SigningIdentity {
        let label = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("Identidad digital")
            .to_owned();
        SigningIdentity { label, path }
    }
}

impl IdentityStore for NativeIdentityStore {
    fn import(&self, source: &Path) -> Result<SigningIdentity> {
        let extension = source
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_default();
        if !matches!(extension.as_str(), "p12" | "pfx") {
            bail!("Selecciona una identidad .p12 o .pfx.");
        }
        fs::create_dir_all(&self.directory)
            .context("no se pudo crear el almacén de identidades")?;
        let stem = Self::safe_stem(source);
        let source_bytes = fs::read(source).context("no se pudo leer la identidad")?;
        let mut index = 1_u32;
        loop {
            let suffix = if index == 1 {
                String::new()
            } else {
                format!("-{index}")
            };
            let target = self.directory.join(format!("{stem}{suffix}.{extension}"));
            if target.exists() {
                if fs::read(&target).ok().as_deref() == Some(source_bytes.as_slice()) {
                    return Ok(Self::identity(target));
                }
                index += 1;
                continue;
            }
            fs::write(&target, &source_bytes).context("no se pudo importar la identidad")?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&target, fs::Permissions::from_mode(0o600))?;
            }
            return Ok(Self::identity(target));
        }
    }

    fn list(&self) -> Result<Vec<SigningIdentity>> {
        if !self.directory.exists() {
            return Ok(Vec::new());
        }
        let mut identities = fs::read_dir(&self.directory)?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| {
                matches!(
                    path.extension()
                        .and_then(|value| value.to_str())
                        .map(str::to_ascii_lowercase)
                        .as_deref(),
                    Some("p12" | "pfx")
                )
            })
            .map(Self::identity)
            .collect::<Vec<_>>();
        identities.sort_by_key(|identity| identity.label.to_lowercase());
        Ok(identities)
    }
}

pub struct NativeDocumentPicker {
    initial: Mutex<Option<PathBuf>>,
}

pub struct NativeTextClipboard;

impl TextClipboard for NativeTextClipboard {
    fn set_text(&self, text: String) -> Result<()> {
        arboard::Clipboard::new()
            .and_then(|mut clipboard| clipboard.set_text(text))
            .context("No se pudo usar el portapapeles del sistema.")
    }
}

impl NativeDocumentPicker {
    pub fn new(initial: Option<PathBuf>) -> Self {
        Self {
            initial: Mutex::new(initial),
        }
    }
}

impl DocumentPicker for NativeDocumentPicker {
    fn pick_pdf(&self) -> Option<DocumentRef> {
        if let Ok(mut initial) = self.initial.lock()
            && let Some(path) = initial.take()
        {
            return Some(DocumentRef::from_path(path));
        }
        rfd::FileDialog::new()
            .add_filter("Documento PDF", &["pdf"])
            .pick_file()
            .map(DocumentRef::from_path)
    }
}

pub struct NativePdfEngine {
    pdfium: &'static Pdfium,
}

impl NativePdfEngine {
    pub fn new() -> Result<Self> {
        let executable_dir = std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf));
        let mut candidate_dirs = Vec::new();
        if let Some(directory) = executable_dir {
            candidate_dirs.push(directory.clone());
            if let Some(parent) = directory.parent() {
                candidate_dirs.push(parent.to_path_buf());
            }
            candidate_dirs.push(directory.join("../Resources"));
        }
        let bindings = candidate_dirs
            .iter()
            .find_map(|directory| {
                Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(directory))
                    .ok()
            })
            .or_else(|| Pdfium::bind_to_system_library().ok())
            .context("No se encontró la biblioteca PDFium incluida con Nitum PDF.")?;
        let pdfium = Box::leak(Box::new(Pdfium::new(bindings)));
        Ok(Self { pdfium })
    }
}

impl PdfEngine for NativePdfEngine {
    fn open(&self, document: &Path, password: Option<&str>) -> Result<Box<dyn OpenPdf>> {
        let document = match self.pdfium.load_pdf_from_file(document, password) {
            Ok(document) => document,
            Err(PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::PasswordError)) => {
                return Err(PdfPasswordRequired.into());
            }
            Err(error) => {
                return Err(error)
                    .context("No se pudo abrir el PDF. Puede estar dañado o no ser compatible.");
            }
        };
        Ok(Box::new(NativePdfDocument { document }))
    }
}

struct NativePdfDocument {
    document: PdfDocument<'static>,
}

impl NativePdfDocument {
    fn page(&self, page_index: u32) -> Result<PdfPage<'_>> {
        let index =
            i32::try_from(page_index).context("El índice de página es demasiado grande.")?;
        self.document
            .pages()
            .get(index)
            .with_context(|| format!("La página {} no existe.", page_index + 1))
    }
}

impl OpenPdf for NativePdfDocument {
    fn page_count(&self) -> u32 {
        self.document.pages().len().max(0) as u32
    }

    fn page_size(&self, page_index: u32) -> Result<PageSize> {
        let page = self.page(page_index)?;
        Ok(PageSize {
            width_points: page.width().value,
            height_points: page.height().value,
            rotation_degrees: page.rotation()?.as_degrees() as i16,
        })
    }

    fn render_page(&self, page_index: u32, scale: f32) -> Result<PageBitmap> {
        if !(0.1..=8.0).contains(&scale) {
            bail!("El zoom debe estar entre 10 % y 800 %.");
        }
        let page = self.page(page_index)?;
        let bitmap = page
            .render_with_config(
                &PdfRenderConfig::new()
                    .scale_page_by_factor(scale)
                    .render_annotations(true)
                    .render_form_data(true),
            )
            .context("No se pudo renderizar la página.")?;
        Ok(PageBitmap {
            width: bitmap.width() as u32,
            height: bitmap.height() as u32,
            rgba: bitmap.as_rgba_bytes(),
        })
    }

    fn page_text(&self, page_index: u32) -> Result<String> {
        self.page(page_index)?
            .text()
            .map(|text| text.all())
            .context("No se pudo extraer el texto de la página.")
    }

    fn text_in_rect(&self, page_index: u32, rect: PdfRect) -> Result<String> {
        let page = self.page(page_index)?;
        let page_text = page
            .text()
            .context("no se pudo leer el texto de la página")?;
        let mut selected_range: Option<(usize, usize)> = None;
        for character in page_text.chars().iter() {
            let Ok(bounds) = character.loose_bounds() else {
                continue;
            };
            let intersects = bounds.right().value >= rect.left
                && bounds.left().value <= rect.right
                && bounds.top().value >= rect.bottom
                && bounds.bottom().value <= rect.top;
            if intersects {
                selected_range = Some(match selected_range {
                    Some((start, _)) => (start, character.index()),
                    None => (character.index(), character.index()),
                });
            }
        }
        let Some((start, end)) = selected_range else {
            return Ok(String::new());
        };
        Ok(page_text
            .chars()
            .iter()
            .filter(|character| (start..=end).contains(&character.index()))
            .filter_map(|character| character.unicode_char())
            .collect())
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let query = query.trim();
        if query.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }
        let mut hits = Vec::new();
        for page_index in 0..self.page_count() {
            let page = self.page(page_index)?;
            let text = page
                .text()
                .context("No se pudo leer el texto de la página.")?;
            let search = text
                .search(query, &PdfSearchOptions::new())
                .context("No se pudo buscar en el PDF.")?;
            for segments in search.iter(PdfSearchDirection::SearchForward) {
                let mut bounds: Option<PdfRect> = None;
                for segment in segments.iter() {
                    let rect = segment.bounds();
                    bounds = Some(match bounds {
                        Some(current) => PdfRect {
                            left: current.left.min(rect.left().value),
                            bottom: current.bottom.min(rect.bottom().value),
                            right: current.right.max(rect.right().value),
                            top: current.top.max(rect.top().value),
                        },
                        None => PdfRect {
                            left: rect.left().value,
                            bottom: rect.bottom().value,
                            right: rect.right().value,
                            top: rect.top().value,
                        },
                    });
                }
                if let Some(bounds) = bounds {
                    hits.push(SearchHit { page_index, bounds });
                    if hits.len() >= limit {
                        return Ok(hits);
                    }
                }
            }
        }
        Ok(hits)
    }
}

/// The common name inside an X.501 distinguished name, or the whole name when
/// it carries no CN.
///
/// A distinguished name prints as `CN=Ana Pérez,O=Empresa,C=EC`, which is not
/// what belongs on a signature stamp: people expect to read their name.
fn common_name(name: &x509_cert::name::Name) -> String {
    const COMMON_NAME_OID: &str = "2.5.4.3";
    for group in name.0.iter() {
        for attribute in group.0.iter() {
            if attribute.oid.to_string() == COMMON_NAME_OID
                && let Ok(value) = attribute.value.decode_as::<der::asn1::Utf8StringRef<'_>>()
            {
                return value.as_str().to_owned();
            }
        }
    }
    name.to_string()
}

/// Local date and time as `dd/mm/aaaa hh:mm`.
///
/// The stamp is read by people, so it uses the order they write dates in rather
/// than the ISO ordering the PDF stores internally.
fn local_timestamp() -> String {
    let now = jiff::Zoned::now();
    format!(
        "{:02}/{:02}/{} {:02}:{:02}",
        now.day(),
        now.month(),
        now.year(),
        now.hour(),
        now.minute()
    )
}

/// The lines a visible signature carries when nobody supplies an image.
///
/// This used to be `TextConfig::default()`, whose `lines` field is empty, so a
/// visible signature was drawn as an empty rectangle: no name, no issuer, no
/// date. Everything it needs is already in the certificate doing the signing,
/// so the stamp fills itself in and the person has to supply nothing.
///
/// It deliberately does not claim the signature is valid. A drawing on a page
/// proves nothing — only verifying the document does — and saying otherwise on
/// the stamp would teach people to trust the wrong thing.
fn appearance_lines(
    certificate_der: &[u8],
    reason: Option<&str>,
    location: Option<&str>,
) -> Vec<TextLine> {
    let mut lines = Vec::new();

    match Certificate::from_der(certificate_der) {
        Ok(certificate) => {
            lines.push(
                TextLine::new(format!(
                    "Firmado por: {}",
                    common_name(&certificate.tbs_certificate.subject)
                ))
                .bold(),
            );
            lines.push(TextLine::new(format!(
                "Emitido por: {}",
                common_name(&certificate.tbs_certificate.issuer)
            )));
        }
        Err(_) => lines.push(TextLine::new("Firmado digitalmente").bold()),
    }

    lines.push(TextLine::new(format!("Fecha: {}", local_timestamp())));
    if let Some(reason) = reason.filter(|value| !value.trim().is_empty()) {
        lines.push(TextLine::new(format!("Motivo: {reason}")));
    }
    if let Some(location) = location.filter(|value| !value.trim().is_empty()) {
        lines.push(TextLine::new(format!("Ubicación: {location}")));
    }
    lines
}

pub struct NativePadesSigning {
    runtime: tokio::runtime::Runtime,
    trust_stores: TrustStoreSet,
}

struct SigningJob<'a> {
    source: &'a Path,
    target: &'a Path,
    level: PadesLevel,
    reason: Option<&'a str>,
    location: Option<&'a str>,
    placement: Option<crate::domain::SignaturePlacement>,
    appearance: Option<&'a Path>,
    certification: Option<CertificationPermission>,
}

impl NativePadesSigning {
    pub fn new() -> Result<Self> {
        let native = rustls_native_certs::load_native_certs();
        let mut trust = TrustStore::new().with_label("sistema");
        for certificate in native.certs {
            let _ = trust.add_der_certificate(certificate.as_ref());
        }
        if trust.is_empty() {
            bail!(
                "No se encontraron autoridades certificadoras en el almacén de confianza del sistema."
            );
        }
        Ok(Self {
            runtime: tokio::runtime::Runtime::new()
                .context("no se pudo iniciar el servicio criptográfico")?,
            trust_stores: TrustStoreSet::new()
                .with_sig_store(trust.clone())
                .with_tsa_store(trust),
        })
    }

    fn verify_bytes(&self, bytes: &[u8]) -> Result<Vec<SignatureReport>> {
        let inspection =
            inspect_signatures(bytes).context("no se pudo inspeccionar la estructura de firmas")?;
        if inspection.num_signatures == 0 {
            return Ok(Vec::new());
        }
        let report = SignatureVerifier::new(&self.trust_stores)
            .verify_pdf(bytes)
            .context("no se pudieron comprobar las firmas")?;
        let mut reports = report
            .signatures
            .into_iter()
            .enumerate()
            .map(|(index, signature)| {
                let safe_revision = signature.covers_whole_document_revision == Some(true)
                    && signature.extended_by_non_safe_updates != Some(true);
                let kind = match signature.signature_type {
                    SignatureType::Pades
                    | SignatureType::Pkcs7Detached
                    | SignatureType::Pkcs7Sha1 => SignatureKind::Approval,
                    SignatureType::DocTimestamp => SignatureKind::DocumentTimestamp,
                    SignatureType::Unknown(_) => SignatureKind::Unknown,
                };
                let pades_level = match signature.pades_level {
                    DetectedPadesLevel::BB => Some(PadesLevel::BaselineB),
                    DetectedPadesLevel::BT => Some(PadesLevel::BaselineT),
                    DetectedPadesLevel::BLT => Some(PadesLevel::BaselineLt),
                    DetectedPadesLevel::BLTA => Some(PadesLevel::BaselineLta),
                    DetectedPadesLevel::NotPades | DetectedPadesLevel::Unknown => None,
                };
                let certification = inspection
                    .signatures
                    .get(index)
                    .and_then(|item| item.doc_mdp_permissions)
                    .and_then(|permission| match permission {
                        1 => Some(CertificationPermission::NoChanges),
                        2 => Some(CertificationPermission::FormFilling),
                        3 => Some(CertificationPermission::FormFillingAndAnnotations),
                        _ => None,
                    });
                SignatureReport {
                    kind,
                    signer_name: signature.signer_name.unwrap_or_else(|| match kind {
                        SignatureKind::DocumentTimestamp => {
                            "Autoridad de sellado de tiempo".to_owned()
                        }
                        _ => "Firmante sin nombre".to_owned(),
                    }),
                    pades_level,
                    certification,
                    cryptographically_intact: matches!(
                        signature.cryptographic_validity,
                        CryptoValidity::Valid
                    ) && signature.digest_matches
                        && (signature.integrity_ok || safe_revision),
                    chain_trusted: signature.chain_trusted,
                    covers_whole_document: signature
                        .covers_whole_document_revision
                        .unwrap_or(signature.covers_whole_document),
                }
            })
            .collect::<Vec<_>>();
        if let Some(dss) = inspection.dss.as_ref() {
            let extracted = underskrift::verify::extractor::extract_signatures(bytes)
                .context("no se pudieron relacionar las firmas con sus entradas VRI")?;
            let timestamps = reports
                .iter()
                .filter(|report| {
                    report.kind == SignatureKind::DocumentTimestamp
                        && report.cryptographically_intact
                })
                .count();
            let level = if timestamps >= 2 {
                PadesLevel::BaselineLta
            } else {
                PadesLevel::BaselineLt
            };
            for (report, signature) in reports.iter_mut().zip(extracted.iter()) {
                if report.kind == SignatureKind::Approval {
                    let key = compute_vri_key(&signature.cms_bytes);
                    if dss
                        .vri
                        .iter()
                        .any(|entry| entry.hash_key == key && entry.num_certs > 0)
                    {
                        report.pades_level = Some(level);
                    }
                }
            }
        }
        Ok(reports)
    }

    fn collect_chain_validation(
        &self,
        builder: &mut DssBuilder,
        certificates: &[Certificate],
        owner: &str,
    ) -> Result<()> {
        if certificates.len() == 1
            && certificates[0].tbs_certificate.subject != certificates[0].tbs_certificate.issuer
        {
            bail!(
                "La cadena de {owner} no incluye su emisor completo; no se generará un B-LT incompleto."
            );
        }
        self.runtime
            .block_on(builder.collect_validation_data(
                certificates,
                &OcspClient::new(),
                &CrlClient::new(),
            ))
            .with_context(|| format!("no se pudieron recopilar los datos OCSP/CRL de {owner}"))?;

        for pair in certificates.windows(2) {
            let certificate = &pair[0];
            let issuer = &pair[1];
            let mut valid_evidence = false;
            for response in &builder.ocsp_responses {
                if let Ok(status) = underskrift::ltv::ocsp::check_revocation(
                    response,
                    certificate,
                    issuer,
                    None,
                    None,
                ) {
                    match status {
                        ValidationStatus::Valid { .. } => valid_evidence = true,
                        ValidationStatus::Revoked { .. } => {
                            bail!("Un certificado de {owner} figura como revocado en OCSP.")
                        }
                        _ => {}
                    }
                }
            }
            for crl in &builder.crls {
                if let Ok(status) =
                    underskrift::ltv::crl::check_revocation(crl, certificate, issuer, None)
                {
                    match status {
                        ValidationStatus::Valid { .. } => valid_evidence = true,
                        ValidationStatus::Revoked { .. } => {
                            bail!("Un certificado de {owner} figura como revocado en la CRL.")
                        }
                        _ => {}
                    }
                }
            }
            if !valid_evidence {
                bail!(
                    "Los datos OCSP/CRL recibidos no validan la cadena de {owner}; no se generará un B-LT incompleto."
                );
            }
        }
        Ok(())
    }

    fn extend_with_ltv(&self, signed: Vec<u8>, archive_timestamp: bool) -> Result<Vec<u8>> {
        let verification = SignatureVerifier::new(&self.trust_stores)
            .verify_pdf(&signed)
            .context("no se pudo preparar la validación de largo plazo")?;
        let signature = verification
            .signatures
            .iter()
            .rev()
            .find(|signature| signature.signature_type == SignatureType::Pades)
            .context("no se encontró la firma PAdES que debe extenderse")?;
        let mut certificate_der = Vec::new();
        certificate_der.push(
            signature
                .signer_cert_der
                .clone()
                .context("la firma no contiene el certificado del firmante")?,
        );
        certificate_der.extend(signature.chain_certs_der.clone());
        let certificates = certificate_der
            .iter()
            .map(|bytes| {
                Certificate::from_der(bytes)
                    .context("la cadena de certificados de la firma no es válida")
            })
            .collect::<Result<Vec<_>>>()?;
        let mut builder = DssBuilder::new();
        self.collect_chain_validation(&mut builder, &certificates, "la firma")?;
        let extracted = underskrift::verify::extractor::extract_signatures(&signed)
            .context("no se pudo identificar la firma para su VRI")?;
        let cms = extracted
            .iter()
            .rev()
            .find(|item| item.signature_type == SignatureType::Pades)
            .map(|item| item.cms_bytes.as_slice())
            .context("no se encontró el contenido CMS para su VRI")?;
        builder.add_vri_entry(
            compute_vri_key(cms),
            VriEntry {
                certs: builder.certificates.clone(),
                ocsps: builder.ocsp_responses.clone(),
                crls: builder.crls.clone(),
            },
        );

        // Baseline-LT archives validation material for every validation object,
        // including the RFC 3161 timestamp that upgrades B-B to B-T.
        for timestamp in extracted
            .iter()
            .filter(|item| item.signature_type == SignatureType::DocTimestamp)
        {
            let tsa_chain =
                underskrift::verify::cms_verify::extract_cms_signer_chain(&timestamp.cms_bytes)
                    .context("el sello RFC 3161 no contiene una cadena TSA utilizable")?;
            self.collect_chain_validation(
                &mut builder,
                &tsa_chain,
                "la autoridad de sellado de tiempo",
            )?;
            builder.add_vri_entry(
                compute_vri_key(&timestamp.cms_bytes),
                VriEntry {
                    certs: builder.certificates.clone(),
                    ocsps: builder.ocsp_responses.clone(),
                    crls: builder.crls.clone(),
                },
            );
        }
        let with_dss = add_document_security_store(&signed, &builder)
            .context("no se pudo incorporar el almacén DSS")?;
        if !archive_timestamp {
            return Ok(with_dss);
        }
        let tsa_url = std::env::var("NITUM_PDF_TSA_URL")
            .unwrap_or_else(|_| "http://timestamp.digicert.com".to_owned());
        self.runtime
            .block_on(underskrift::core::doc_timestamp::add_document_timestamp(
                &with_dss,
                &underskrift::tsp::TsaClient::new(&tsa_url),
                &underskrift::core::doc_timestamp::DocTimestampOptions::default(),
            ))
            .context("no se pudo añadir el sello de archivo PAdES B-LTA")
    }

    fn write_new_file(target: &Path, bytes: &[u8]) -> Result<()> {
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(target)
            .with_context(|| format!("no se pudo crear {}", target.display()))?;
        if let Err(error) = output.write_all(bytes).and_then(|()| output.sync_all()) {
            drop(output);
            let _ = fs::remove_file(target);
            return Err(error).context("no se pudo guardar el PDF firmado");
        }
        Ok(())
    }

    fn load_software_identity(path: &Path, password: &str) -> Result<SoftwareSigner> {
        if let Ok(signer) = SoftwareSigner::from_pkcs12_file(path, password) {
            return Ok(signer);
        }
        let archive = fs::read(path).context("no se pudo leer la identidad .p12/.pfx")?;
        let mut contents = bergshamra_pkcs12::parse_pkcs12(&archive, password)
            .context("la contraseña no es correcta o el formato PKCS#12 no es compatible")?;
        let private_key = contents
            .private_keys
            .pop()
            .context("la identidad no contiene una clave privada")?;
        SoftwareSigner::from_rsa_der(&private_key, contents.certificates)
            .context("la identidad moderna no contiene una clave RSA compatible")
    }

    fn sign_with(
        &self,
        request: SigningJob<'_>,
        identity: &dyn CryptoSigner,
    ) -> Result<Vec<SignatureReport>> {
        if request.source == request.target {
            bail!("El PDF original nunca se sobrescribe al firmar.");
        }
        let source = fs::read(request.source).context("no se pudo leer el PDF")?;
        if request.certification.is_some() && !self.verify_bytes(&source)?.is_empty() {
            bail!("Para certificar un documento, la certificación tiene que ser su primera firma.");
        }
        let pades_level = match request.level {
            PadesLevel::BaselineB => underskrift::PadesLevel::BB,
            PadesLevel::BaselineT => underskrift::PadesLevel::BT,
            PadesLevel::BaselineLt | PadesLevel::BaselineLta => underskrift::PadesLevel::BT,
        };
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .context("el reloj del sistema no es válido")?
            .as_nanos();
        let tsa_url = (request.level != PadesLevel::BaselineB).then(|| {
            std::env::var("NITUM_PDF_TSA_URL")
                .unwrap_or_else(|_| "http://timestamp.digicert.com".to_owned())
        });
        // The stamp writes itself from the certificate that is signing, so a
        // visible signature carries the signer, the issuer and the date whether
        // or not the person also chose an image.
        let signature_text = TextConfig {
            lines: appearance_lines(identity.certificate_der(), request.reason, request.location),
            ..TextConfig::default()
        };
        let appearance_layout = if let Some(path) = request.appearance {
            let data = fs::read(path).context("no se pudo leer la firma visual guardada")?;
            let format = match path
                .extension()
                .and_then(|value| value.to_str())
                .map(str::to_ascii_lowercase)
                .as_deref()
            {
                Some("png") => ImageFormat::Png,
                Some("jpg" | "jpeg") => ImageFormat::Jpeg,
                _ => bail!("la firma visual guardada no tiene un formato compatible"),
            };
            Some(SignatureLayout::ImageAndText {
                image: ImageConfig {
                    data,
                    format,
                    scale: ImageScale::FitPreserveAspect,
                },
                text: signature_text,
                arrangement: Arrangement::ImageLeftTextRight,
            })
        } else {
            request
                .placement
                .map(|_| SignatureLayout::TextOnly(signature_text))
        };
        let options = SigningOptions {
            pades_level,
            field_name: format!("NitumSignature{unique}"),
            reason: request.reason.map(str::to_owned),
            location: request.location.map(str::to_owned),
            page: request
                .placement
                .map_or(0, |placement| placement.page_index),
            visible_signature: request.placement.zip(appearance_layout).map(
                |(placement, layout)| VisibleSignatureConfig {
                    page: placement.page_index,
                    rect: SignatureRect::Absolute {
                        llx: placement.left,
                        lly: placement.bottom,
                        urx: placement.left + placement.width,
                        ury: placement.bottom + placement.height,
                    },
                    layout,
                    // The stamp sits on top of the document, so it needs its own
                    // ground: without one the text underneath showed straight
                    // through and neither could be read. A thin rule marks where
                    // the stamp ends, the way Acrobat and FirmaEC both do.
                    background_color: Some(Color::new(1.0, 1.0, 1.0)),
                    border: Some(Border {
                        width: 0.75,
                        color: Color::new(0.62, 0.13, 0.16),
                    }),
                },
            ),
            tsa_url,
            certify: request.certification.is_some(),
            certify_permissions: match request.certification {
                Some(CertificationPermission::NoChanges) | None => DocMdpPermissions::NoChanges,
                Some(CertificationPermission::FormFilling) => {
                    DocMdpPermissions::FormFillingAndSigning
                }
                Some(CertificationPermission::FormFillingAndAnnotations) => {
                    DocMdpPermissions::FormFillingSigningAndAnnotation
                }
            },
            ..SigningOptions::default()
        };
        let signed = self
            .runtime
            .block_on(PdfSigner::new().options(options).sign(&source, identity))
            .context("no se pudo firmar el PDF")?;
        let signed = match request.level {
            PadesLevel::BaselineLt => self.extend_with_ltv(signed, false)?,
            PadesLevel::BaselineLta => self.extend_with_ltv(signed, true)?,
            _ => signed,
        };
        let reports = self.verify_bytes(&signed)?;
        let approval = reports
            .iter()
            .rev()
            .find(|report| report.kind == SignatureKind::Approval);
        if approval.is_none_or(|report| !report.cryptographically_intact) {
            bail!("La comprobación posterior a la firma no fue válida.");
        }
        if request.level == PadesLevel::BaselineT
            && (approval.is_none_or(|report| report.pades_level != Some(PadesLevel::BaselineT))
                || !reports.iter().any(|report| {
                    report.kind == SignatureKind::DocumentTimestamp
                        && report.cryptographically_intact
                }))
        {
            bail!("El sello de tiempo RFC 3161 no superó la comprobación posterior.");
        }
        if matches!(
            request.level,
            PadesLevel::BaselineLt | PadesLevel::BaselineLta
        ) {
            let expected = request.level;
            let required_timestamps = if request.level == PadesLevel::BaselineLta {
                2
            } else {
                1
            };
            let valid_timestamps = reports
                .iter()
                .filter(|report| {
                    report.kind == SignatureKind::DocumentTimestamp
                        && report.cryptographically_intact
                        && report.chain_trusted
                })
                .count();
            if approval.is_none_or(|report| report.pades_level != Some(expected))
                || valid_timestamps < required_timestamps
            {
                bail!("La validación de largo plazo PAdES no superó la comprobación posterior.");
            }
        }
        if let Some(permission) = request.certification {
            let expected = match permission {
                CertificationPermission::NoChanges => 1,
                CertificationPermission::FormFilling => 2,
                CertificationPermission::FormFillingAndAnnotations => 3,
            };
            let inspection = inspect_signatures(&signed)
                .context("no se pudo comprobar la certificación del documento")?;
            if inspection.num_signatures != 1
                || inspection.catalog_doc_mdp_obj_num
                    != inspection
                        .signatures
                        .first()
                        .and_then(|signature| signature.obj_num)
                || inspection
                    .signatures
                    .first()
                    .and_then(|signature| signature.doc_mdp_permissions)
                    != Some(expected)
            {
                bail!("La certificación no quedó incorporada correctamente en el documento.");
            }
        }
        Self::write_new_file(request.target, &signed)?;
        Ok(reports)
    }
}

impl PdfSigning for NativePadesSigning {
    fn sign(&self, request: SignRequest<'_>) -> Result<Vec<SignatureReport>> {
        let identity = Self::load_software_identity(request.identity, request.secret)?;
        self.sign_with(
            SigningJob {
                source: request.source,
                target: request.target,
                level: request.level,
                reason: request.reason,
                location: request.location,
                placement: request.placement,
                appearance: request.appearance,
                certification: request.certification,
            },
            &identity,
        )
    }

    fn sign_hardware(&self, request: HardwareSignRequest<'_>) -> Result<Vec<SignatureReport>> {
        let identity = Pkcs11Signer::open(request.token, request.pin)?;
        self.sign_with(
            SigningJob {
                source: request.source,
                target: request.target,
                level: request.level,
                reason: request.reason,
                location: request.location,
                placement: request.placement,
                appearance: request.appearance,
                certification: request.certification,
            },
            &identity,
        )
    }

    fn verify(&self, document: &Path) -> Result<Vec<SignatureReport>> {
        self.verify_bytes(&fs::read(document).context("no se pudo leer el PDF")?)
    }
}
