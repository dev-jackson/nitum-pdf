use crate::domain::{
    AppRelease, CertificationPermission, DocumentRef, HardwareToken, PadesLevel, PageBitmap,
    PageSize, SearchHit, SignatureAppearance, SignaturePlacement, SignatureReport, SigningIdentity,
};
use anyhow::Result;
use std::path::Path;

pub trait DocumentPicker {
    fn pick_pdf(&self) -> Option<DocumentRef>;
}

/// An already-open document. The password is consumed at open time and never
/// needs to cross the presentation boundary again.
pub trait OpenPdf: Send + Sync {
    fn page_count(&self) -> u32;
    fn page_size(&self, page_index: u32) -> Result<PageSize>;
    fn render_page(&self, page_index: u32, scale: f32) -> Result<PageBitmap>;
    fn page_text(&self, page_index: u32) -> Result<String>;
    fn text_in_rect(&self, page_index: u32, rect: crate::domain::PdfRect) -> Result<String>;
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>>;
}

/// Factory for read-only PDF sessions. Callers do not depend on PDFium.
pub trait PdfEngine: Send + Sync {
    fn open(&self, document: &Path, password: Option<&str>) -> Result<Box<dyn OpenPdf>>;
}

pub struct SignRequest<'a> {
    pub source: &'a Path,
    pub target: &'a Path,
    pub identity: &'a Path,
    pub secret: &'a str,
    pub level: PadesLevel,
    pub reason: Option<&'a str>,
    pub location: Option<&'a str>,
    pub placement: Option<SignaturePlacement>,
    pub appearance: Option<&'a Path>,
    pub certification: Option<CertificationPermission>,
}

pub struct HardwareSignRequest<'a> {
    pub source: &'a Path,
    pub target: &'a Path,
    pub token: &'a HardwareToken,
    pub pin: &'a str,
    pub level: PadesLevel,
    pub reason: Option<&'a str>,
    pub location: Option<&'a str>,
    pub placement: Option<SignaturePlacement>,
    pub appearance: Option<&'a Path>,
    pub certification: Option<CertificationPermission>,
}

/// PDF signing is independent from how private-key operations are provided.
/// A software identity and a PKCS#11 device must satisfy this same contract.
pub trait PdfSigning: Send + Sync {
    fn sign(&self, request: SignRequest<'_>) -> Result<Vec<SignatureReport>>;
    fn sign_hardware(&self, request: HardwareSignRequest<'_>) -> Result<Vec<SignatureReport>>;
    fn verify(&self, document: &Path) -> Result<Vec<SignatureReport>>;
}

pub trait IdentityStore: Send + Sync {
    fn import(&self, source: &Path) -> Result<SigningIdentity>;
    fn list(&self) -> Result<Vec<SigningIdentity>>;
}

pub trait AppearanceStore: Send + Sync {
    fn import(&self, source: &Path) -> Result<SignatureAppearance>;
    fn list(&self) -> Result<Vec<SignatureAppearance>>;
}

pub trait TextClipboard: Send + Sync {
    fn set_text(&self, text: String) -> Result<()>;
}

pub trait HardwareTokenProvider: Send + Sync {
    fn detected_modules(&self) -> Vec<std::path::PathBuf>;
    fn tokens(&self, module: &Path) -> Result<Vec<HardwareToken>>;
}

pub trait UpdateService: Send + Sync {
    fn latest(&self) -> Result<Option<AppRelease>>;
    fn download_verified(&self, release: &AppRelease) -> Result<std::path::PathBuf>;
}

/// Remembers what the person decided about updates, so the answer sticks.
pub trait UpdatePreferences: Send + Sync {
    /// Whether to look for updates at all.
    fn automatic(&self) -> bool;
    fn set_automatic(&self, automatic: bool) -> Result<()>;
    /// Whether this exact version has already been turned down.
    fn is_dismissed(&self, version: &str) -> bool;
    fn dismiss(&self, version: &str) -> Result<()>;
}

pub trait UpdateInstaller: Send + Sync {
    fn install(&self, package: &Path) -> Result<()>;
    fn relaunch(&self, document: Option<&Path>) -> Result<()>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Viewport {
    pub page_index: u32,
    pub zoom_percent: i32,
}

#[derive(Debug)]
pub struct ViewerState {
    viewport: Viewport,
    page_count: u32,
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            viewport: Viewport {
                page_index: 0,
                zoom_percent: 100,
            },
            page_count: 0,
        }
    }
}

impl ViewerState {
    pub fn opened(&mut self, page_count: u32) -> Viewport {
        self.page_count = page_count;
        self.viewport = Viewport {
            page_index: 0,
            zoom_percent: 100,
        };
        self.viewport
    }

    pub fn previous_page(&mut self) -> Option<Viewport> {
        if self.viewport.page_index == 0 {
            return None;
        }
        self.viewport.page_index -= 1;
        Some(self.viewport)
    }

    pub fn next_page(&mut self) -> Option<Viewport> {
        if self.viewport.page_index + 1 >= self.page_count {
            return None;
        }
        self.viewport.page_index += 1;
        Some(self.viewport)
    }

    pub fn zoom_by(&mut self, delta: i32) -> Viewport {
        self.viewport.zoom_percent = (self.viewport.zoom_percent + delta).clamp(10, 800);
        self.viewport
    }

    pub fn fit_width(&mut self) -> Viewport {
        self.viewport.zoom_percent = 100;
        self.viewport
    }

    pub fn go_to(&mut self, page_index: u32) -> Option<Viewport> {
        if page_index >= self.page_count {
            return None;
        }
        self.viewport.page_index = page_index;
        Some(self.viewport)
    }
}

pub struct OpenDocument<P> {
    picker: P,
}

impl<P: DocumentPicker> OpenDocument<P> {
    pub fn new(picker: P) -> Self {
        Self { picker }
    }
    pub fn execute(&self) -> Option<DocumentRef> {
        self.picker.pick_pdf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct FixedPicker;
    impl DocumentPicker for FixedPicker {
        fn pick_pdf(&self) -> Option<DocumentRef> {
            Some(DocumentRef::from_path(PathBuf::from("sample.pdf")))
        }
    }

    #[test]
    fn open_document_depends_on_a_port() {
        assert_eq!(
            OpenDocument::new(FixedPicker)
                .execute()
                .unwrap()
                .display_name(),
            "sample.pdf"
        );
    }

    #[test]
    fn viewer_navigation_never_leaves_document_bounds() {
        let mut state = ViewerState::default();
        state.opened(2);
        assert_eq!(state.previous_page(), None);
        assert_eq!(state.next_page().unwrap().page_index, 1);
        assert_eq!(state.next_page(), None);
        assert_eq!(state.previous_page().unwrap().page_index, 0);
    }

    #[test]
    fn viewer_zoom_is_bounded_and_fit_is_predictable() {
        let mut state = ViewerState::default();
        state.opened(1);
        assert_eq!(state.zoom_by(-1000).zoom_percent, 10);
        assert_eq!(state.zoom_by(2000).zoom_percent, 800);
        assert_eq!(state.fit_width().zoom_percent, 100);
    }
}
