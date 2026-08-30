#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use anyhow::Result;
use nitum_pdf::{
    application::OpenDocument,
    infrastructure::{
        NativeAppearanceStore, NativeDocumentPicker, NativeHardwareTokenProvider,
        NativeIdentityStore, NativePadesSigning, NativePdfEngine, NativeTextClipboard,
        NativeUpdatePreferences,
        updater::{GithubUpdateService, NativeUpdateInstaller},
    },
    presentation::{self, PresentationServices},
};
use std::sync::Arc;

fn main() -> Result<()> {
    let initial_document = std::env::args_os().nth(1).map(std::path::PathBuf::from);
    let open_at_startup = initial_document.is_some();
    let open_document = OpenDocument::new(NativeDocumentPicker::new(initial_document));
    presentation::run(
        open_document,
        PresentationServices {
            pdf_engine: Arc::new(NativePdfEngine::new()?),
            pdf_signing: Arc::new(NativePadesSigning::new()?),
            identity_store: Arc::new(NativeIdentityStore::new()?),
            appearance_store: Arc::new(NativeAppearanceStore::new()?),
            text_clipboard: Arc::new(NativeTextClipboard),
            token_provider: Arc::new(NativeHardwareTokenProvider),
            update_service: Arc::new(GithubUpdateService::new()?),
            update_installer: Arc::new(NativeUpdateInstaller),
            update_preferences: Arc::new(NativeUpdatePreferences::new()?),
        },
        open_at_startup,
    )
}
