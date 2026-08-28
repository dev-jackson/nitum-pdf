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
    /// Places the signature box **centred** on the given point, expressed as a
    /// fraction of the page with the origin at its top-left corner.
    ///
    /// The point is the centre and not a corner because the person picks a spot
    /// by pointing at it: anchoring the corner there made the signature land
    /// down and to the right of the place they aimed at, by half its own size.
    /// The box is then clamped so it always stays inside the page.
    pub fn from_normalized_point(page_index: u32, page: PageSize, x: f32, y_from_top: f32) -> Self {
        let width = SIGNATURE_WIDTH_POINTS.min(page.width_points.max(1.0));
        let height = SIGNATURE_HEIGHT_POINTS.min(page.height_points.max(1.0));
        let left = (x.clamp(0.0, 1.0) * page.width_points - width / 2.0)
            .clamp(0.0, (page.width_points - width).max(0.0));
        let bottom =
            (page.height_points - y_from_top.clamp(0.0, 1.0) * page.height_points - height / 2.0)
                .clamp(0.0, (page.height_points - height).max(0.0));
        Self {
            page_index,
            left,
            bottom,
            width,
            height,
        }
    }

    /// Places the signature inside the rectangle the person dragged over the
    /// page, given as two opposite corners in page fractions with the origin at
    /// the top-left corner.
    ///
    /// This is how Acrobat asks for a signature area: you drag a box and the
    /// signature fills it. A drag smaller than `SIGNATURE_MIN_*_POINTS` is
    /// treated as a click rather than a box, because a stray two-pixel drag
    /// should not produce a signature too small to read.
    pub fn from_normalized_rect(
        page_index: u32,
        page: PageSize,
        start: (f32, f32),
        end: (f32, f32),
    ) -> Self {
        let left_fraction = start.0.min(end.0).clamp(0.0, 1.0);
        let right_fraction = start.0.max(end.0).clamp(0.0, 1.0);
        let top_fraction = start.1.min(end.1).clamp(0.0, 1.0);
        let bottom_fraction = start.1.max(end.1).clamp(0.0, 1.0);

        let width = (right_fraction - left_fraction) * page.width_points;
        let height = (bottom_fraction - top_fraction) * page.height_points;
        if width < SIGNATURE_MIN_WIDTH_POINTS || height < SIGNATURE_MIN_HEIGHT_POINTS {
            let centre_x = (left_fraction + right_fraction) / 2.0;
            let centre_y = (top_fraction + bottom_fraction) / 2.0;
            return Self::from_normalized_point(page_index, page, centre_x, centre_y);
        }

        Self {
            page_index,
            left: left_fraction * page.width_points,
            bottom: page.height_points - bottom_fraction * page.height_points,
            width: width.min(page.width_points),
            height: height.min(page.height_points),
        }
    }
}

/// Size of a visible signature, in PDF points. The preview drawn over the page
/// and the box written into the document both read these, so what you see when
/// you place it is the size you get.
pub const SIGNATURE_WIDTH_POINTS: f32 = 220.0;
pub const SIGNATURE_HEIGHT_POINTS: f32 = 72.0;
/// Margin from the page corner when nobody picks a spot.
pub const SIGNATURE_DEFAULT_MARGIN_POINTS: f32 = 36.0;
/// Below this a drag is a click, not a box: a signature that small is unreadable.
pub const SIGNATURE_MIN_WIDTH_POINTS: f32 = 60.0;
pub const SIGNATURE_MIN_HEIGHT_POINTS: f32 = 24.0;

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
        // Aiming at the far corner keeps the whole box on the page.
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

    #[test]
    fn a_dragged_rectangle_becomes_the_signature_area() {
        let page = PageSize {
            width_points: 600.0,
            height_points: 800.0,
            rotation_degrees: 0,
        };
        // Dragging from a quarter in to three quarters across, and from a tenth
        // down to a third down, fills exactly that box.
        let dragged = SignaturePlacement::from_normalized_rect(1, page, (0.25, 0.10), (0.75, 0.30));
        // Fractions of a page are floats, so these compare within a rounding error.
        let close = |value: f32, expected: f32| (value - expected).abs() < 0.01;
        assert_eq!(dragged.page_index, 1);
        assert!(close(dragged.left, 150.0), "left was {}", dragged.left);
        assert!(close(dragged.width, 300.0), "width was {}", dragged.width);
        assert!(
            close(dragged.height, 160.0),
            "height was {}",
            dragged.height
        );
        // 0.30 of the way down is 240 points from the top, so 560 up from the bottom.
        assert!(
            close(dragged.bottom, 560.0),
            "bottom was {}",
            dragged.bottom
        );

        // Dragging backwards describes the same box.
        let backwards =
            SignaturePlacement::from_normalized_rect(1, page, (0.75, 0.30), (0.25, 0.10));
        assert_eq!(backwards.left, dragged.left);
        assert_eq!(backwards.bottom, dragged.bottom);
        assert_eq!(backwards.width, dragged.width);
        assert_eq!(backwards.height, dragged.height);
    }

    #[test]
    fn a_drag_too_small_to_read_is_treated_as_a_click() {
        let page = PageSize {
            width_points: 600.0,
            height_points: 800.0,
            rotation_degrees: 0,
        };
        // A two-pixel wobble while clicking must not produce a sliver.
        let wobble = SignaturePlacement::from_normalized_rect(0, page, (0.5, 0.5), (0.503, 0.502));
        assert_eq!(wobble.width, SIGNATURE_WIDTH_POINTS);
        assert_eq!(wobble.height, SIGNATURE_HEIGHT_POINTS);
        assert!((wobble.left + wobble.width / 2.0 - 300.9).abs() < 0.01);
    }

    #[test]
    fn visible_signature_is_centred_on_the_point_you_choose() {
        let page = PageSize {
            width_points: 600.0,
            height_points: 800.0,
            rotation_degrees: 0,
        };
        // The middle of the page puts the middle of the box there: the centre
        // sits at (300, 400) whichever way the axes run.
        let middle = SignaturePlacement::from_normalized_point(0, page, 0.5, 0.5);
        assert_eq!(middle.left + middle.width / 2.0, 300.0);
        assert_eq!(middle.bottom + middle.height / 2.0, 400.0);

        // A quarter down from the top is 200 points from the top, so the centre
        // sits 600 points up from the bottom.
        let upper = SignaturePlacement::from_normalized_point(0, page, 0.25, 0.25);
        assert_eq!(upper.left + upper.width / 2.0, 150.0);
        assert_eq!(upper.bottom + upper.height / 2.0, 600.0);
    }
}
