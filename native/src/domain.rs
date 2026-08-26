use std::{
    fmt,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PdfPasswordRequired;

impl fmt::Display for PdfPasswordRequired {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Este PDF necesita una contraseña válida.")
    }
}

impl std::error::Error for PdfPasswordRequired {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageSize {
    pub width_points: f32,
    pub height_points: f32,
    pub rotation_degrees: i16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PageBitmap {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PdfRect {
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SearchHit {
    pub page_index: u32,
    pub bounds: PdfRect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PadesLevel {
    BaselineB,
    BaselineT,
    BaselineLt,
    BaselineLta,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertificationPermission {
    NoChanges,
    FormFilling,
    FormFillingAndAnnotations,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignatureKind {
    Approval,
    DocumentTimestamp,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SignaturePlacement {
    pub page_index: u32,
    pub left: f32,
    pub bottom: f32,
    pub width: f32,
    pub height: f32,
}

impl SignaturePlacement {
    pub fn from_normalized_point(page_index: u32, page: PageSize, x: f32, y_from_top: f32) -> Self {
        let width = 220.0_f32.min(page.width_points.max(1.0));
        let height = 72.0_f32.min(page.height_points.max(1.0));
        let left = (x.clamp(0.0, 1.0) * page.width_points)
            .clamp(0.0, (page.width_points - width).max(0.0));
        let bottom =
            (page.height_points - y_from_top.clamp(0.0, 1.0) * page.height_points - height)
                .clamp(0.0, (page.height_points - height).max(0.0));
        Self {
            page_index,
            left,
            bottom,
            width,
            height,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignatureReport {
    pub kind: SignatureKind,
    pub signer_name: String,
    pub pades_level: Option<PadesLevel>,
    pub certification: Option<CertificationPermission>,
    pub cryptographically_intact: bool,
    pub chain_trusted: bool,
    pub covers_whole_document: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SigningIdentity {
    pub label: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignatureAppearance {
    pub label: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HardwareToken {
    pub label: String,
    pub serial: String,
    pub module_path: PathBuf,
    pub slot_id: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppRelease {
    pub version: String,
    pub package_name: String,
    pub package_url: String,
    pub checksum_url: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentRef {
    path: PathBuf,
    display_name: String,
}

impl DocumentRef {
    pub fn from_path(path: PathBuf) -> Self {
        let display_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .unwrap_or("Documento PDF")
            .to_owned();
        Self { path, display_name }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn display_name(&self) -> &str {
        &self.display_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_a_safe_display_name() {
        let document = DocumentRef::from_path(PathBuf::from("/tmp/contrato.pdf"));
        assert_eq!(document.display_name(), "contrato.pdf");
    }

    #[test]
    fn visible_signature_is_clamped_inside_the_page() {
        let page = PageSize {
            width_points: 600.0,
            height_points: 800.0,
            rotation_degrees: 0,
        };
        let bottom_right = SignaturePlacement::from_normalized_point(2, page, 1.0, 1.0);
        assert_eq!(bottom_right.page_index, 2);
        assert_eq!(bottom_right.left, 380.0);
        assert_eq!(bottom_right.bottom, 0.0);
        assert_eq!(bottom_right.width, 220.0);
        assert_eq!(bottom_right.height, 72.0);

        let top_left = SignaturePlacement::from_normalized_point(0, page, -1.0, -1.0);
        assert_eq!(top_left.left, 0.0);
        assert_eq!(top_left.bottom, 728.0);
    }
}
