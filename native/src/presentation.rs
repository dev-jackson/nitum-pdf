use crate::{
    AppWindow, PageItem, VerificationCheck,
    application::{
        AppearanceStore, DocumentPicker, HardwareSignRequest, HardwareTokenProvider, IdentityStore,
        OpenDocument, OpenPdf, PdfEngine, PdfSigning, SignRequest, TextClipboard, UpdateInstaller,
        UpdateService, ViewerState,
    },
    domain::{
        AppRelease, CertificationPermission, HardwareToken, PadesLevel, PageBitmap,
        PdfPasswordRequired, SignatureAppearance, SignaturePlacement, SigningIdentity,
    },
};
use anyhow::Result;
use slint::{
    ComponentHandle, Image, Model, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel, Weak,
};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

type ActiveDocument = Arc<Mutex<Option<Box<dyn OpenPdf>>>>;
type PendingDocument = Arc<Mutex<Option<PathBuf>>>;
type TokenChoices = Arc<Mutex<Vec<HardwareToken>>>;
type AvailableRelease = Arc<Mutex<Option<AppRelease>>>;
type AppearanceChoices = Arc<Mutex<Vec<SignatureAppearance>>>;
type IdentityChoices = Arc<Mutex<Vec<SigningIdentity>>>;

pub struct PresentationServices {
    pub pdf_engine: Arc<dyn PdfEngine>,
    pub pdf_signing: Arc<dyn PdfSigning>,
    pub identity_store: Arc<dyn IdentityStore>,
    pub appearance_store: Arc<dyn AppearanceStore>,
    pub text_clipboard: Arc<dyn TextClipboard>,
    pub token_provider: Arc<dyn HardwareTokenProvider>,
    pub update_service: Arc<dyn UpdateService>,
    pub update_installer: Arc<dyn UpdateInstaller>,
}

fn spawn_update_check(
    service: Arc<dyn UpdateService>,
    available: AvailableRelease,
    weak: Weak<AppWindow>,
    interactive: bool,
) {
    if interactive && let Some(ui) = weak.upgrade() {
        ui.set_active_dialog(4);
        ui.set_update_checking(true);
        ui.set_update_status("Consultando releases oficiales…".into());
    }
    std::thread::spawn(move || {
        let result = service.latest();
        let _ = slint::invoke_from_event_loop(move || {
            let Some(ui) = weak.upgrade() else { return };
            ui.set_update_checking(false);
            match result {
                Ok(Some(release)) => {
                    ui.set_update_version(release.version.as_str().into());
                    ui.set_update_available(true);
                    ui.set_update_status(
                        format!("La versión {} está lista para descargar.", release.version).into(),
                    );
                    if let Ok(mut value) = available.lock() {
                        *value = Some(release);
                    }
                }
                Ok(None) => {
                    ui.set_update_available(false);
                    ui.set_update_status("Ya tienes la versión más reciente.".into());
                    if let Ok(mut value) = available.lock() {
                        *value = None;
                    }
                }
                Err(error) if interactive => {
                    ui.set_update_available(false);
                    ui.set_update_status(
                        format!("No pudimos comprobar actualizaciones: {error}").into(),
                    );
                }
                Err(_) => {
                    // Automatic checks stay silent when the device is offline.
                }
            }
        });
    });
}

#[derive(Clone)]
struct OpenContext {
    engine: Arc<dyn PdfEngine>,
    active_document: ActiveDocument,
    pending_document: PendingDocument,
    viewer_state: Arc<Mutex<ViewerState>>,
    generation: Arc<AtomicU64>,
    weak: Weak<AppWindow>,
}

fn spawn_token_scan(
    provider: Arc<dyn HardwareTokenProvider>,
    modules: Vec<PathBuf>,
    choices: TokenChoices,
    weak: Weak<AppWindow>,
) {
    if let Some(ui) = weak.upgrade() {
        ui.set_active_dialog(6);
        ui.set_token_status("Buscando tarjetas conectadas…".into());
        ui.set_token_options(ModelRc::default());
    }
    std::thread::spawn(move || {
        let mut found = Vec::new();
        let mut last_error = None;
        for module in modules {
            match provider.tokens(&module) {
                Ok(mut tokens) => found.append(&mut tokens),
                Err(error) => last_error = Some(error.to_string()),
            }
        }
        let _ = slint::invoke_from_event_loop(move || {
            let Some(ui) = weak.upgrade() else { return };
            if found.is_empty() {
                ui.set_token_status(
                    last_error
                        .unwrap_or_else(|| "No encontramos una tarjeta. Conéctala o elige el módulo PKCS#11 de su fabricante.".to_owned())
                        .into(),
                );
                return;
            }
            let labels = found
                .iter()
                .map(|token| format!("{} · {}", token.label, token.serial).into())
                .collect::<Vec<_>>();
            ui.set_token_status(format!("Encontramos {} dispositivo(s).", found.len()).into());
            ui.set_token_options(ModelRc::new(Rc::new(VecModel::from(labels))));
            if let Ok(mut value) = choices.lock() {
                *value = found;
            }
        });
    });
}

/// Verdicts a verification row can carry, mirrored in `VerificationCheck.tone`.
const TONE_NEUTRAL: i32 = 0;
const TONE_OK: i32 = 1;
const TONE_WARNING: i32 = 2;
const TONE_FAILED: i32 = 3;

/// "1 firma" and "2 firmas" rather than the "firma(s)" the interface used to show.
fn describe_count(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("1 {singular}")
    } else {
        format!("{count} {plural}")
    }
}

fn page_image(bitmap: PageBitmap) -> (Image, f32) {
    let aspect = bitmap.height as f32 / bitmap.width.max(1) as f32;
    let pixels = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        &bitmap.rgba,
        bitmap.width,
        bitmap.height,
    );
    (Image::from_rgba8(pixels), aspect)
}

fn evict_distant_pages(ui: &AppWindow, center: usize) {
    let model = ui.get_pages();
    for index in 0..model.row_count() {
        if index.abs_diff(center) <= 1 {
            continue;
        }
        let Some(page) = model.row_data(index) else {
            continue;
        };
        if page.loaded {
            model.set_row_data(
                index,
                PageItem {
                    image: Image::default(),
                    loaded: false,
                    ..page
                },
            );
        }
    }
}

fn invalidate_other_page_renders(ui: &AppWindow, keep: usize) {
    let model = ui.get_pages();
    for index in 0..model.row_count() {
        if index == keep {
            continue;
        }
        let Some(page) = model.row_data(index) else {
            continue;
        };
        if page.loaded {
            model.set_row_data(
                index,
                PageItem {
                    image: Image::default(),
                    loaded: false,
                    ..page
                },
            );
        }
    }
}

/// Width, in logical pixels, that a page is rasterised at. The window owns the
/// value because zoom 100 % means "as wide as the window allows"; if the window
/// is gone the fallback only has to be sane, never exact.
fn render_base_width(weak: &Weak<AppWindow>) -> f32 {
    weak.upgrade()
        .map(|ui| ui.get_page_base_width())
        .filter(|width| *width >= 320.0)
        .unwrap_or(760.0)
}

fn spawn_render(
    document: ActiveDocument,
    generation: Arc<AtomicU64>,
    weak: Weak<AppWindow>,
    page_index: u32,
    zoom_percent: i32,
    navigate: bool,
) {
    let token = generation.load(Ordering::SeqCst);
    let base_width = render_base_width(&weak);
    if navigate && let Some(ui) = weak.upgrade() {
        ui.set_document_loading(true);
        ui.set_document_error("".into());
    }
    std::thread::spawn(move || {
        let rendered = (|| {
            let guard = document
                .lock()
                .map_err(|_| anyhow::anyhow!("estado del documento bloqueado"))?;
            let pdf = guard
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("No hay un PDF abierto."))?;
            if page_index >= pdf.page_count() {
                anyhow::bail!("La página solicitada no existe.");
            }
            let size = pdf.page_size(page_index)?;
            let display_width = (base_width * zoom_percent as f32 / 100.0).min(4096.0);
            let scale = (display_width / size.width_points.max(1.0)).clamp(0.1, 8.0);
            pdf.render_page(page_index, scale)
        })();
        let _ = slint::invoke_from_event_loop(move || {
            if generation.load(Ordering::SeqCst) != token {
                return;
            }
            let Some(ui) = weak.upgrade() else { return };
            if navigate {
                ui.set_document_loading(false);
            }
            match rendered {
                Ok(bitmap) => {
                    let (image, aspect) = page_image(bitmap);
                    ui.set_page_image(image.clone());
                    ui.set_page_aspect(aspect);
                    let model = ui.get_pages();
                    model.set_row_data(
                        page_index as usize,
                        PageItem {
                            number: page_index.min(i32::MAX as u32) as i32 + 1,
                            image,
                            aspect,
                            loaded: true,
                            error: "".into(),
                        },
                    );
                    ui.set_current_page(page_index.min(i32::MAX as u32) as i32 + 1);
                    ui.set_zoom_percent(zoom_percent);
                    if navigate {
                        ui.set_scroll_page(page_index.min(i32::MAX as u32) as i32);
                    }
                }
                Err(error) => {
                    let message = error.to_string();
                    let model = ui.get_pages();
                    if let Some(mut page) = model.row_data(page_index as usize) {
                        page.error = message.as_str().into();
                        model.set_row_data(page_index as usize, page);
                    } else {
                        ui.set_document_error(message.into());
                    }
                }
            }
        });
    });
}

fn spawn_open(context: OpenContext, path: PathBuf, password: Option<zeroize::Zeroizing<String>>) {
    let token = context.generation.fetch_add(1, Ordering::SeqCst) + 1;
    let base_width = render_base_width(&context.weak);
    if let Some(ui) = context.weak.upgrade() {
        ui.set_document_loading(true);
        ui.set_document_error("".into());
        ui.set_unlock_busy(password.is_some());
        ui.set_unlock_status("".into());
    }
    std::thread::spawn(move || {
        let loaded = context
            .engine
            .open(&path, password.as_deref().map(String::as_str))
            .and_then(|pdf| {
                let page_count = pdf.page_count();
                if page_count == 0 {
                    anyhow::bail!("El PDF no contiene páginas.");
                }
                let aspects = (0..page_count)
                    .map(|index| {
                        let size = pdf.page_size(index)?;
                        Ok::<_, anyhow::Error>(size.height_points / size.width_points.max(1.0))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let size = pdf.page_size(0)?;
                let scale = (base_width / size.width_points.max(1.0)).clamp(0.1, 8.0);
                let bitmap = pdf.render_page(0, scale)?;
                if context.generation.load(Ordering::SeqCst) == token {
                    *context
                        .active_document
                        .lock()
                        .map_err(|_| anyhow::anyhow!("estado del documento bloqueado"))? =
                        Some(pdf);
                    context
                        .viewer_state
                        .lock()
                        .map_err(|_| anyhow::anyhow!("estado del visor bloqueado"))?
                        .opened(page_count);
                }
                Ok((page_count, bitmap, aspects))
            });
        let _ = slint::invoke_from_event_loop(move || {
            if context.generation.load(Ordering::SeqCst) != token {
                return;
            }
            let Some(ui) = context.weak.upgrade() else {
                return;
            };
            ui.set_document_loading(false);
            ui.set_unlock_busy(false);
            match loaded {
                Ok((page_count, bitmap, aspects)) => {
                    if let Ok(mut pending) = context.pending_document.lock() {
                        *pending = None;
                    }
                    let (image, aspect) = page_image(bitmap);
                    ui.set_page_image(image.clone());
                    ui.set_page_aspect(aspect);
                    ui.set_page_count(page_count.min(i32::MAX as u32) as i32);
                    ui.set_scroll_page(0);
                    let pages = aspects
                        .into_iter()
                        .enumerate()
                        .map(|(index, page_aspect)| PageItem {
                            number: index.min(i32::MAX as usize) as i32 + 1,
                            image: if index == 0 {
                                image.clone()
                            } else {
                                Image::default()
                            },
                            aspect: page_aspect,
                            loaded: index == 0,
                            error: "".into(),
                        })
                        .collect::<Vec<_>>();
                    ui.set_pages(ModelRc::new(Rc::new(VecModel::from(pages))));
                    ui.set_active_dialog(0);
                }
                Err(error) if error.downcast_ref::<PdfPasswordRequired>().is_some() => {
                    ui.set_active_dialog(3);
                    ui.set_unlock_status(
                        if password.is_some() {
                            "La contraseña no es correcta. Inténtalo de nuevo."
                        } else {
                            ""
                        }
                        .into(),
                    );
                }
                Err(error) => ui.set_document_error(error.to_string().into()),
            }
        });
    });
}

pub fn run<P: DocumentPicker + 'static>(
    open_document: OpenDocument<P>,
    services: PresentationServices,
    open_at_startup: bool,
) -> Result<()> {
    let PresentationServices {
        pdf_engine,
        pdf_signing,
        identity_store,
        appearance_store,
        text_clipboard,
        token_provider,
        update_service,
        update_installer,
    } = services;
    // Wayland and X11 match a window to its desktop entry through the XDG application id,
    // so without this the shell has no way to find com.nitum.Pdf.desktop and shows no icon.
    // It only applies to those platforms; elsewhere the call is inert.
    let _ = slint::set_xdg_app_id("com.nitum.Pdf");
    let ui = AppWindow::new()?;
    let active_document: ActiveDocument = Arc::new(Mutex::new(None));
    let pending_document: PendingDocument = Arc::new(Mutex::new(None));
    let generation = Arc::new(AtomicU64::new(0));
    let viewer_state = Arc::new(Mutex::new(ViewerState::default()));
    let available_release: AvailableRelease = Arc::new(Mutex::new(None));
    let token_choices: TokenChoices = Arc::new(Mutex::new(Vec::new()));
    let selected_token: Arc<Mutex<Option<HardwareToken>>> = Arc::new(Mutex::new(None));
    ui.set_current_version(env!("CARGO_PKG_VERSION").into());

    let weak = ui.as_weak();
    let document_state = Arc::clone(&active_document);
    let render_generation = Arc::clone(&generation);
    let viewer_for_open = Arc::clone(&viewer_state);
    let pending_for_open = Arc::clone(&pending_document);
    let engine_for_open = Arc::clone(&pdf_engine);
    ui.on_open_document(move || {
        let Some(document) = open_document.execute() else {
            return;
        };
        if let Some(ui) = weak.upgrade() {
            ui.set_document_title(document.display_name().into());
            ui.set_document_path(document.path().to_string_lossy().into_owned().into());
            ui.set_document_open(true);
            ui.set_document_loading(true);
            ui.set_document_error("".into());
            ui.set_page_count(0);
            ui.set_current_page(1);
            ui.set_zoom_percent(100);
            ui.set_search_status("".into());
        }
        let path = document.path().to_owned();
        if let Ok(mut pending) = pending_for_open.lock() {
            *pending = Some(path.clone());
        }
        spawn_open(
            OpenContext {
                engine: Arc::clone(&engine_for_open),
                active_document: Arc::clone(&document_state),
                pending_document: Arc::clone(&pending_for_open),
                viewer_state: Arc::clone(&viewer_for_open),
                generation: Arc::clone(&render_generation),
                weak: weak.clone(),
            },
            path,
            None,
        );
    });

    let engine = Arc::clone(&pdf_engine);
    let state = Arc::clone(&active_document);
    let pending = Arc::clone(&pending_document);
    let viewer = Arc::clone(&viewer_state);
    let render_generation = Arc::clone(&generation);
    let weak = ui.as_weak();
    ui.on_unlock_document(move |password| {
        let path = pending.lock().ok().and_then(|value| value.clone());
        let Some(path) = path else { return };
        spawn_open(
            OpenContext {
                engine: Arc::clone(&engine),
                active_document: Arc::clone(&state),
                pending_document: Arc::clone(&pending),
                viewer_state: Arc::clone(&viewer),
                generation: Arc::clone(&render_generation),
                weak: weak.clone(),
            },
            path,
            Some(zeroize::Zeroizing::new(password.to_string())),
        );
    });

    let pending = Arc::clone(&pending_document);
    let state = Arc::clone(&active_document);
    let generation_for_cancel = Arc::clone(&generation);
    let weak = ui.as_weak();
    ui.on_cancel_unlock(move || {
        generation_for_cancel.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut value) = pending.lock() {
            *value = None;
        }
        if let Ok(mut value) = state.lock() {
            *value = None;
        }
        if let Some(ui) = weak.upgrade() {
            ui.set_active_dialog(0);
            ui.set_document_open(false);
            ui.set_document_loading(false);
            ui.set_document_error("".into());
            ui.set_unlock_status("".into());
        }
    });

    let state = Arc::clone(&active_document);
    let render_generation = Arc::clone(&generation);
    let weak = ui.as_weak();
    ui.on_request_page(move |index| {
        let Ok(page_index) = u32::try_from(index) else {
            return;
        };
        let Some(ui) = weak.upgrade() else { return };
        if ui
            .get_pages()
            .row_data(page_index as usize)
            .is_some_and(|page| page.loaded)
        {
            return;
        }
        spawn_render(
            Arc::clone(&state),
            Arc::clone(&render_generation),
            weak.clone(),
            page_index,
            ui.get_zoom_percent(),
            false,
        );
    });

    let viewer = Arc::clone(&viewer_state);
    let document_for_visible = Arc::clone(&active_document);
    let generation_for_visible = Arc::clone(&generation);
    let weak = ui.as_weak();
    ui.on_visible_page(move |index| {
        let Ok(page_index) = u32::try_from(index) else {
            return;
        };
        if viewer
            .lock()
            .ok()
            .and_then(|mut state| state.go_to(page_index))
            .is_some()
            && let Some(ui) = weak.upgrade()
        {
            ui.set_current_page(index + 1);
            evict_distant_pages(&ui, page_index as usize);
            if ui
                .get_pages()
                .row_data(page_index as usize)
                .is_some_and(|page| !page.loaded)
            {
                spawn_render(
                    Arc::clone(&document_for_visible),
                    Arc::clone(&generation_for_visible),
                    weak.clone(),
                    page_index,
                    ui.get_zoom_percent(),
                    false,
                );
            }
        }
    });

    let state = Arc::clone(&active_document);
    let render_generation = Arc::clone(&generation);
    let viewer = Arc::clone(&viewer_state);
    let weak = ui.as_weak();
    ui.on_previous_page(move || {
        let viewport = viewer
            .lock()
            .ok()
            .and_then(|mut state| state.previous_page());
        if let Some(viewport) = viewport {
            spawn_render(
                Arc::clone(&state),
                Arc::clone(&render_generation),
                weak.clone(),
                viewport.page_index,
                viewport.zoom_percent,
                true,
            );
        }
    });

    let state = Arc::clone(&active_document);
    let render_generation = Arc::clone(&generation);
    let viewer = Arc::clone(&viewer_state);
    let weak = ui.as_weak();
    ui.on_next_page(move || {
        let viewport = viewer.lock().ok().and_then(|mut state| state.next_page());
        if let Some(viewport) = viewport {
            spawn_render(
                Arc::clone(&state),
                Arc::clone(&render_generation),
                weak.clone(),
                viewport.page_index,
                viewport.zoom_percent,
                true,
            );
        }
    });

    let state = Arc::clone(&active_document);
    let render_generation = Arc::clone(&generation);
    let viewer = Arc::clone(&viewer_state);
    let weak = ui.as_weak();
    ui.on_zoom_out(move || {
        let Some(viewport) = viewer.lock().ok().map(|mut state| state.zoom_by(-25)) else {
            return;
        };
        if let Some(ui) = weak.upgrade() {
            invalidate_other_page_renders(&ui, viewport.page_index as usize);
        }
        spawn_render(
            Arc::clone(&state),
            Arc::clone(&render_generation),
            weak.clone(),
            viewport.page_index,
            viewport.zoom_percent,
            true,
        );
    });

    let state = Arc::clone(&active_document);
    let render_generation = Arc::clone(&generation);
    let viewer = Arc::clone(&viewer_state);
    let weak = ui.as_weak();
    ui.on_zoom_in(move || {
        let Some(viewport) = viewer.lock().ok().map(|mut state| state.zoom_by(25)) else {
            return;
        };
        if let Some(ui) = weak.upgrade() {
            invalidate_other_page_renders(&ui, viewport.page_index as usize);
        }
        spawn_render(
            Arc::clone(&state),
            Arc::clone(&render_generation),
            weak.clone(),
            viewport.page_index,
            viewport.zoom_percent,
            true,
        );
    });

    let state = Arc::clone(&active_document);
    let render_generation = Arc::clone(&generation);
    let viewer = Arc::clone(&viewer_state);
    let weak = ui.as_weak();
    ui.on_fit_width(move || {
        let Some(viewport) = viewer.lock().ok().map(|mut state| state.fit_width()) else {
            return;
        };
        if let Some(ui) = weak.upgrade() {
            invalidate_other_page_renders(&ui, viewport.page_index as usize);
        }
        spawn_render(
            Arc::clone(&state),
            Arc::clone(&render_generation),
            weak.clone(),
            viewport.page_index,
            viewport.zoom_percent,
            true,
        );
    });

    let state = Arc::clone(&active_document);
    let search_generation = Arc::clone(&generation);
    let viewer_for_search = Arc::clone(&viewer_state);
    let weak = ui.as_weak();
    ui.on_search_document(move |query| {
        let query = query.to_string();
        let token = search_generation.fetch_add(1, Ordering::SeqCst) + 1;
        let state = Arc::clone(&state);
        let generation = Arc::clone(&search_generation);
        let viewer = Arc::clone(&viewer_for_search);
        let weak = weak.clone();
        let base_width = render_base_width(&weak);
        if let Some(ui) = weak.upgrade() {
            ui.set_search_status("Buscando…".into())
        }
        std::thread::spawn(move || {
            let result = (|| {
                let guard = state
                    .lock()
                    .map_err(|_| anyhow::anyhow!("estado bloqueado"))?;
                let pdf = guard
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("No hay un PDF abierto."))?;
                let hits = pdf.search(&query, 500)?;
                let first_page = hits.first().map(|hit| hit.page_index);
                let bitmap = if let Some(page) = first_page {
                    let size = pdf.page_size(page)?;
                    Some(pdf.render_page(
                        page,
                        (base_width / size.width_points.max(1.0)).clamp(0.1, 8.0),
                    )?)
                } else {
                    None
                };
                Ok::<_, anyhow::Error>((hits.len(), first_page, bitmap))
            })();
            let _ = slint::invoke_from_event_loop(move || {
                if generation.load(Ordering::SeqCst) != token {
                    return;
                }
                let Some(ui) = weak.upgrade() else { return };
                match result {
                    Ok((count, first_page, bitmap)) => {
                        let status = match count {
                            0 => "Sin resultados".to_owned(),
                            1 => "1 resultado".to_owned(),
                            _ => format!("{count} resultados"),
                        };
                        ui.set_search_status(status.into());
                        if let (Some(page), Some(bitmap)) = (first_page, bitmap) {
                            if let Ok(mut state) = viewer.lock() {
                                state.go_to(page);
                                state.fit_width();
                            }
                            let (image, aspect) = page_image(bitmap);
                            ui.set_page_image(image.clone());
                            ui.set_page_aspect(aspect);
                            let model = ui.get_pages();
                            model.set_row_data(
                                page as usize,
                                PageItem {
                                    number: page.min(i32::MAX as u32) as i32 + 1,
                                    image,
                                    aspect,
                                    loaded: true,
                                    error: "".into(),
                                },
                            );
                            ui.set_current_page(page as i32 + 1);
                            ui.set_scroll_page(page.min(i32::MAX as u32) as i32);
                            ui.set_zoom_percent(100);
                        }
                    }
                    Err(error) => ui.set_search_status(error.to_string().into()),
                }
            });
        });
    });

    let state = Arc::clone(&active_document);
    let page_clipboard = Arc::clone(&text_clipboard);
    let weak = ui.as_weak();
    ui.on_copy_current_page(move || {
        let Some(ui) = weak.upgrade() else { return };
        let Ok(page_index) = u32::try_from(ui.get_current_page().saturating_sub(1)) else {
            return;
        };
        ui.set_viewer_status("Preparando texto…".into());
        let state = Arc::clone(&state);
        let weak = weak.clone();
        let clipboard = Arc::clone(&page_clipboard);
        std::thread::spawn(move || {
            let result = (|| {
                let guard = state
                    .lock()
                    .map_err(|_| anyhow::anyhow!("estado del documento bloqueado"))?;
                let document = guard
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("No hay un PDF abierto."))?;
                let text = document.page_text(page_index)?;
                if text.trim().is_empty() {
                    anyhow::bail!("Esta página no contiene texto copiable.");
                }
                Ok::<_, anyhow::Error>(text)
            })();
            let _ = slint::invoke_from_event_loop(move || {
                let Some(ui) = weak.upgrade() else { return };
                match result {
                    Ok(text) => match clipboard.set_text(text) {
                        Ok(()) => ui.set_viewer_status("Texto de la página copiado".into()),
                        Err(error) => ui.set_viewer_status(
                            format!("No se pudo usar el portapapeles: {error}").into(),
                        ),
                    },
                    Err(error) => ui.set_viewer_status(error.to_string().into()),
                }
            });
        });
    });

    let state = Arc::clone(&active_document);
    let weak = ui.as_weak();
    ui.on_copy_selection(move |page_index, x1, y1, x2, y2| {
        let Some(ui) = weak.upgrade() else { return };
        let Ok(page_index) = u32::try_from(page_index) else {
            return;
        };
        ui.set_viewer_status("Copiando selección…".into());
        let state = Arc::clone(&state);
        let weak = weak.clone();
        let clipboard = Arc::clone(&text_clipboard);
        std::thread::spawn(move || {
            let result = (|| {
                let guard = state
                    .lock()
                    .map_err(|_| anyhow::anyhow!("estado del documento bloqueado"))?;
                let document = guard
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("No hay un PDF abierto."))?;
                let size = document.page_size(page_index)?;
                let left = x1.min(x2).clamp(0.0, 1.0) * size.width_points;
                let right = x1.max(x2).clamp(0.0, 1.0) * size.width_points;
                let top_from_screen = y1.min(y2).clamp(0.0, 1.0);
                let bottom_from_screen = y1.max(y2).clamp(0.0, 1.0);
                let text = document.text_in_rect(
                    page_index,
                    crate::domain::PdfRect {
                        left,
                        right,
                        bottom: size.height_points * (1.0 - bottom_from_screen),
                        top: size.height_points * (1.0 - top_from_screen),
                    },
                )?;
                if text.trim().is_empty() {
                    anyhow::bail!("La zona seleccionada no contiene texto copiable.");
                }
                clipboard.set_text(text)
            })();
            let _ = slint::invoke_from_event_loop(move || {
                let Some(ui) = weak.upgrade() else { return };
                ui.set_viewer_status(
                    match result {
                        Ok(()) => "Selección copiada".to_owned(),
                        Err(error) => error.to_string(),
                    }
                    .into(),
                );
            });
        });
    });

    let initial_identities = identity_store.list().unwrap_or_default();
    let identity_choices: IdentityChoices = Arc::new(Mutex::new(initial_identities.clone()));
    ui.set_identity_options(ModelRc::new(Rc::new(VecModel::from(
        initial_identities
            .iter()
            .map(|identity| identity.label.as_str().into())
            .collect::<Vec<slint::SharedString>>(),
    ))));
    if let Some(identity) = initial_identities.first() {
        ui.set_identity_name(identity.label.as_str().into());
        ui.set_identity_path(identity.path.to_string_lossy().into_owned().into());
    }

    let weak = ui.as_weak();
    let identities = Arc::clone(&identity_store);
    let choices = Arc::clone(&identity_choices);
    ui.on_show_identity_library(move || {
        let Some(ui) = weak.upgrade() else { return };
        match identities.list() {
            Ok(items) => {
                ui.set_identity_options(ModelRc::new(Rc::new(VecModel::from(
                    items
                        .iter()
                        .map(|identity| identity.label.as_str().into())
                        .collect::<Vec<slint::SharedString>>(),
                ))));
                if let Ok(mut stored) = choices.lock() {
                    *stored = items;
                }
                ui.set_active_dialog(8);
            }
            Err(error) => {
                ui.set_signing_success(false);
                ui.set_signing_status(error.to_string().into());
            }
        }
    });

    let weak = ui.as_weak();
    let choices = Arc::clone(&identity_choices);
    let selected = Arc::clone(&selected_token);
    ui.on_choose_identity(move |index| {
        let identity = usize::try_from(index)
            .ok()
            .and_then(|index| choices.lock().ok()?.get(index).cloned());
        let Some(identity) = identity else { return };
        if let Ok(mut token) = selected.lock() {
            *token = None;
        }
        if let Some(ui) = weak.upgrade() {
            ui.set_identity_kind(0);
            ui.set_identity_name(identity.label.into());
            ui.set_identity_path(identity.path.to_string_lossy().into_owned().into());
            ui.set_signing_success(true);
            ui.set_signing_status("Identidad elegida. Introduce su contraseña para firmar.".into());
            ui.set_active_dialog(2);
        }
    });
    let initial_appearances = appearance_store.list().unwrap_or_default();
    let appearance_choices: AppearanceChoices = Arc::new(Mutex::new(initial_appearances.clone()));
    ui.set_appearance_options(ModelRc::new(Rc::new(VecModel::from(
        initial_appearances
            .iter()
            .map(|appearance| appearance.label.as_str().into())
            .collect::<Vec<slint::SharedString>>(),
    ))));
    if let Some(appearance) = initial_appearances.first() {
        ui.set_appearance_name(appearance.label.as_str().into());
        ui.set_appearance_path(appearance.path.to_string_lossy().into_owned().into());
    }

    let weak = ui.as_weak();
    let appearances = Arc::clone(&appearance_store);
    let choices = Arc::clone(&appearance_choices);
    ui.on_show_appearance_library(move || {
        let Some(ui) = weak.upgrade() else { return };
        match appearances.list() {
            Ok(items) => {
                ui.set_appearance_options(ModelRc::new(Rc::new(VecModel::from(
                    items
                        .iter()
                        .map(|appearance| appearance.label.as_str().into())
                        .collect::<Vec<slint::SharedString>>(),
                ))));
                if let Ok(mut stored) = choices.lock() {
                    *stored = items;
                }
                ui.set_active_dialog(7);
            }
            Err(error) => {
                ui.set_signing_success(false);
                ui.set_signing_status(error.to_string().into());
            }
        }
    });

    let weak = ui.as_weak();
    let choices = Arc::clone(&appearance_choices);
    ui.on_choose_appearance(move |index| {
        let appearance = usize::try_from(index)
            .ok()
            .and_then(|index| choices.lock().ok()?.get(index).cloned());
        let Some(appearance) = appearance else { return };
        if let Some(ui) = weak.upgrade() {
            ui.set_appearance_name(appearance.label.into());
            ui.set_appearance_path(appearance.path.to_string_lossy().into_owned().into());
            ui.set_signing_success(true);
            ui.set_signing_status("Firma visual elegida para este documento.".into());
            ui.set_active_dialog(2);
        }
    });

    let weak = ui.as_weak();
    let identities = Arc::clone(&identity_store);
    let choices = Arc::clone(&identity_choices);
    let selected_for_file = Arc::clone(&selected_token);
    ui.on_select_identity(move || {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Identidad digital PKCS#12", &["p12", "pfx"])
            .pick_file()
        else {
            return;
        };
        if let Some(ui) = weak.upgrade() {
            match identities.import(&path) {
                Ok(identity) => {
                    if let Ok(mut token) = selected_for_file.lock() {
                        *token = None;
                    }
                    ui.set_identity_kind(0);
                    ui.set_identity_name(identity.label.as_str().into());
                    ui.set_identity_path(identity.path.to_string_lossy().into_owned().into());
                    if let Ok(items) = identities.list() {
                        ui.set_identity_options(ModelRc::new(Rc::new(VecModel::from(
                            items
                                .iter()
                                .map(|item| item.label.as_str().into())
                                .collect::<Vec<slint::SharedString>>(),
                        ))));
                        if let Ok(mut stored) = choices.lock() {
                            *stored = items;
                        }
                    }
                    ui.set_signing_status("Identidad guardada para reutilizarla.".into());
                    ui.set_signing_success(true);
                    if ui.get_active_dialog() == 8 {
                        ui.set_active_dialog(2);
                    }
                }
                Err(error) => {
                    ui.set_signing_status(error.to_string().into());
                    ui.set_signing_success(false);
                }
            }
        }
    });

    let provider = Arc::clone(&token_provider);
    let choices = Arc::clone(&token_choices);
    let weak = ui.as_weak();
    ui.on_discover_tokens(move || {
        let modules = provider.detected_modules();
        spawn_token_scan(
            Arc::clone(&provider),
            modules,
            Arc::clone(&choices),
            weak.clone(),
        );
    });

    let provider = Arc::clone(&token_provider);
    let choices = Arc::clone(&token_choices);
    let weak = ui.as_weak();
    ui.on_select_token_module(move || {
        let Some(module) = rfd::FileDialog::new()
            .add_filter("Módulo PKCS#11", &["so", "dylib", "dll"])
            .pick_file()
        else {
            return;
        };
        spawn_token_scan(
            Arc::clone(&provider),
            vec![module],
            Arc::clone(&choices),
            weak.clone(),
        );
    });

    let choices = Arc::clone(&token_choices);
    let selected = Arc::clone(&selected_token);
    let weak = ui.as_weak();
    ui.on_choose_token(move |index| {
        let token = usize::try_from(index)
            .ok()
            .and_then(|index| choices.lock().ok()?.get(index).cloned());
        let Some(token) = token else { return };
        if let Ok(mut value) = selected.lock() {
            *value = Some(token.clone());
        }
        if let Some(ui) = weak.upgrade() {
            ui.set_identity_kind(1);
            ui.set_token_label(token.label.into());
            ui.set_active_dialog(2);
            ui.set_signing_status("Tarjeta lista. El PIN se usará una sola vez.".into());
            ui.set_signing_success(true);
        }
    });

    let weak = ui.as_weak();
    let appearances = Arc::clone(&appearance_store);
    let choices = Arc::clone(&appearance_choices);
    ui.on_select_appearance(move || {
        let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "Firma visual",
                &["png", "jpg", "jpeg", "gif", "bmp", "tif", "tiff", "webp"],
            )
            .pick_file()
        else {
            return;
        };
        if let Some(ui) = weak.upgrade() {
            match appearances.import(&path) {
                Ok(appearance) => {
                    ui.set_appearance_name(appearance.label.as_str().into());
                    ui.set_appearance_path(appearance.path.to_string_lossy().into_owned().into());
                    if let Ok(items) = appearances.list() {
                        ui.set_appearance_options(ModelRc::new(Rc::new(VecModel::from(
                            items
                                .iter()
                                .map(|item| item.label.as_str().into())
                                .collect::<Vec<slint::SharedString>>(),
                        ))));
                        if let Ok(mut stored) = choices.lock() {
                            *stored = items;
                        }
                    }
                    ui.set_signing_success(true);
                    ui.set_signing_status("Firma visual guardada para reutilizarla.".into());
                    if matches!(ui.get_active_dialog(), 1 | 7) {
                        ui.set_active_dialog(2);
                    }
                }
                Err(error) => {
                    ui.set_signing_success(false);
                    ui.set_signing_status(error.to_string().into());
                }
            }
        }
    });

    let weak = ui.as_weak();
    let signing_service = Arc::clone(&pdf_signing);
    let hardware_token = Arc::clone(&selected_token);
    let document_for_signing = Arc::clone(&active_document);
    ui.on_sign_with_identity(
        move |secret, level, certification, reason, location, visible| {
            let Some(ui) = weak.upgrade() else { return };
            let source = std::path::PathBuf::from(ui.get_document_path().to_string());
            let identity = std::path::PathBuf::from(ui.get_identity_path().to_string());
            let appearance = PathBuf::from(ui.get_appearance_path().to_string());
            let identity_kind = ui.get_identity_kind();
            let page_index = ui.get_current_page().saturating_sub(1) as u32;
            let requested_position = ui.get_signature_position_set().then(|| {
                (
                    ui.get_signature_page().max(0) as u32,
                    ui.get_signature_x().clamp(0.0, 1.0),
                    ui.get_signature_y().clamp(0.0, 1.0),
                )
            });
            if source.as_os_str().is_empty()
                || (identity_kind == 0 && identity.as_os_str().is_empty())
            {
                return;
            }
            let token = hardware_token.lock().ok().and_then(|value| value.clone());
            if identity_kind == 1 && token.is_none() {
                ui.set_signing_success(false);
                ui.set_signing_status("Vuelve a seleccionar la tarjeta o el token.".into());
                return;
            }
            let suggested = format!(
                "{}-{}.pdf",
                source
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("documento"),
                if certification == -1 {
                    "firmado"
                } else {
                    "certificado"
                }
            );
            let mut dialog = rfd::FileDialog::new()
                .add_filter("Documento PDF", &["pdf"])
                .set_file_name(&suggested);
            if let Some(parent) = source.parent() {
                dialog = dialog.set_directory(parent)
            }
            let Some(target) = dialog.save_file() else {
                return;
            };

            ui.set_signing_busy(true);
            ui.set_signing_success(false);
            ui.set_signing_status("Preparando y comprobando la firma…".into());
            let signing = Arc::clone(&signing_service);
            let weak = weak.clone();
            let secret = zeroize::Zeroizing::new(secret.to_string());
            let reason = reason.trim().to_owned();
            let location = location.trim().to_owned();
            let level = match level {
                1 => PadesLevel::BaselineT,
                2 => PadesLevel::BaselineLt,
                3 => PadesLevel::BaselineLta,
                _ => PadesLevel::BaselineB,
            };
            let certification = match certification {
                1 => Some(CertificationPermission::NoChanges),
                2 => Some(CertificationPermission::FormFilling),
                3 => Some(CertificationPermission::FormFillingAndAnnotations),
                _ => None,
            };
            let placement = if visible {
                let selected_page = requested_position.map_or(page_index, |value| value.0);
                let dimensions = document_for_signing
                    .lock()
                    .ok()
                    .and_then(|document| document.as_ref()?.page_size(selected_page).ok());
                let Some(dimensions) = dimensions else {
                    ui.set_signing_busy(false);
                    ui.set_signing_status("No se pudo calcular la posición de la firma.".into());
                    return;
                };
                Some(requested_position.map_or(
                    SignaturePlacement {
                        page_index: selected_page,
                        left: 36.0_f32.min((dimensions.width_points - 220.0).max(0.0)),
                        bottom: 36.0_f32.min((dimensions.height_points - 72.0).max(0.0)),
                        width: 220.0_f32.min(dimensions.width_points.max(1.0)),
                        height: 72.0_f32.min(dimensions.height_points.max(1.0)),
                    },
                    |(_, x, y)| {
                        SignaturePlacement::from_normalized_point(selected_page, dimensions, x, y)
                    },
                ))
            } else {
                None
            };
            std::thread::spawn(move || {
                let appearance =
                    (!appearance.as_os_str().is_empty()).then_some(appearance.as_path());
                let reason = (!reason.is_empty()).then_some(reason.as_str());
                let location = (!location.is_empty()).then_some(location.as_str());
                let result = if let Some(token) = token.as_ref() {
                    signing.sign_hardware(HardwareSignRequest {
                        source: &source,
                        target: &target,
                        token,
                        pin: secret.as_str(),
                        level,
                        reason,
                        location,
                        placement,
                        appearance,
                        certification,
                    })
                } else {
                    signing.sign(SignRequest {
                        source: &source,
                        target: &target,
                        identity: &identity,
                        secret: secret.as_str(),
                        level,
                        reason,
                        location,
                        placement,
                        appearance,
                        certification,
                    })
                };
                let _ = slint::invoke_from_event_loop(move || {
                    let Some(ui) = weak.upgrade() else { return };
                    ui.set_signing_busy(false);
                    match result {
                        Ok(reports) => {
                            let signer = reports
                                .iter()
                                .rev()
                                .find(|report| {
                                    report.kind == crate::domain::SignatureKind::Approval
                                })
                                .map(|report| report.signer_name.as_str())
                                .unwrap_or("Firmante");
                            let name = target
                                .file_name()
                                .and_then(|value| value.to_str())
                                .unwrap_or("PDF firmado");
                            ui.set_signing_success(true);
                            ui.set_signing_status(
                                format!(
                                    "{} válida de {signer}. Guardado como {name}.",
                                    if certification.is_some() {
                                        "Certificación DocMDP"
                                    } else {
                                        "Firma"
                                    }
                                )
                                .into(),
                            );
                        }
                        Err(error) => {
                            ui.set_signing_success(false);
                            ui.set_signing_status(error.to_string().into());
                        }
                    }
                });
            });
        },
    );

    let weak = ui.as_weak();
    ui.on_show_signature_center(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_active_dialog(1)
        }
    });
    let weak = ui.as_weak();
    ui.on_close_dialog(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_active_dialog(0)
        }
    });
    let weak = ui.as_weak();
    ui.on_choose_signing(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_active_dialog(2)
        }
    });

    let signing = Arc::clone(&pdf_signing);
    let weak = ui.as_weak();
    ui.on_verify_signatures(move || {
        let Some(ui) = weak.upgrade() else { return };
        let path = PathBuf::from(ui.get_document_path().to_string());
        if path.as_os_str().is_empty() {
            return;
        }
        ui.set_active_dialog(5);
        ui.set_verification_busy(true);
        ui.set_verification_success(false);
        ui.set_verification_status("Comprobando todas las firmas…".into());
        let signing = Arc::clone(&signing);
        let weak = weak.clone();
        std::thread::spawn(move || {
            let result = signing.verify(&path);
            let _ = slint::invoke_from_event_loop(move || {
                let Some(ui) = weak.upgrade() else { return };
                ui.set_verification_busy(false);
                match result {
                    Ok(reports) if reports.is_empty() => {
                        ui.set_verification_success(false);
                        ui.set_verification_status("Este PDF no contiene firmas digitales.".into());
                    }
                    Ok(reports) => {
                        let count = reports
                            .iter()
                            .filter(|report| {
                                report.kind == crate::domain::SignatureKind::Approval
                            })
                            .count();
                        let timestamps = reports
                            .iter()
                            .filter(|report| {
                                report.kind == crate::domain::SignatureKind::DocumentTimestamp
                            })
                            .count();
                        let intact = reports.iter().all(|report| report.cryptographically_intact);
                        let trusted = reports.iter().all(|report| report.chain_trusted);
                        let covered = reports.iter().all(|report| report.covers_whole_document);
                        let names = reports
                            .iter()
                            .filter(|report| {
                                report.kind == crate::domain::SignatureKind::Approval
                            })
                            .map(|report| report.signer_name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        let levels = reports
                            .iter()
                            .filter(|report| {
                                report.kind == crate::domain::SignatureKind::Approval
                            })
                            .filter_map(|report| report.pades_level)
                            .map(|level| match level {
                                PadesLevel::BaselineB => "B-B",
                                PadesLevel::BaselineT => "B-T",
                                PadesLevel::BaselineLt => "B-LT",
                                PadesLevel::BaselineLta => "B-LTA",
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        let certifications = reports
                            .iter()
                            .filter_map(|report| report.certification)
                            .map(|permission| match permission {
                                CertificationPermission::NoChanges => "sin cambios",
                                CertificationPermission::FormFilling => "formularios y firmas",
                                CertificationPermission::FormFillingAndAnnotations => {
                                    "formularios, firmas y anotaciones"
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        let certification_summary = if certifications.is_empty() {
                            "Sin certificación DocMDP".to_owned()
                        } else {
                            format!("Certificación DocMDP: {certifications}")
                        };
                        // One row per question a reader actually has, each
                        // with its own verdict. Folding these into a single
                        // sentence hid the important case: a document can be
                        // perfectly intact and still be signed by a certificate
                        // nobody has vouched for.
                        let mut checks: Vec<VerificationCheck> = Vec::new();
                        checks.push(VerificationCheck {
                            tone: if intact { TONE_OK } else { TONE_FAILED },
                            label: if intact {
                                "El documento no se ha alterado".into()
                            } else {
                                "El documento se modificó después de firmarse".into()
                            },
                            detail: if intact {
                                "El contenido coincide exactamente con lo que se firmó.".into()
                            } else {
                                "La comprobación criptográfica no cuadra: no te fíes de este PDF."
                                    .into()
                            },
                        });
                        checks.push(VerificationCheck {
                            tone: if covered { TONE_OK } else { TONE_WARNING },
                            label: if covered {
                                "La firma cubre todo el documento".into()
                            } else {
                                "La firma sólo cubre una parte".into()
                            },
                            detail: if covered {
                                "No hay páginas ni cambios fuera de lo firmado.".into()
                            } else {
                                "Se añadió contenido después de firmar; esa parte no está respaldada."
                                    .into()
                            },
                        });
                        checks.push(VerificationCheck {
                            tone: if trusted { TONE_OK } else { TONE_WARNING },
                            label: if trusted {
                                "El certificado es de confianza".into()
                            } else {
                                "No podemos confirmar quién emitió el certificado".into()
                            },
                            detail: if trusted {
                                "La cadena llega hasta una autoridad reconocida por este equipo."
                                    .into()
                            } else {
                                "La firma es válida, pero nadie en este equipo avala la identidad del firmante."
                                    .into()
                            },
                        });
                        checks.push(VerificationCheck {
                            tone: TONE_NEUTRAL,
                            label: if names.is_empty() {
                                "Sin firmante identificado".into()
                            } else {
                                format!("Firmado por {names}").into()
                            },
                            detail: format!(
                                "{}{}. {certification_summary}.",
                                describe_count(count, "firma", "firmas"),
                                if timestamps == 0 {
                                    String::new()
                                } else {
                                    format!(
                                        ", {}",
                                        describe_count(
                                            timestamps,
                                            "sello de tiempo",
                                            "sellos de tiempo"
                                        )
                                    )
                                },
                            )
                            .into(),
                        });
                        if !levels.is_empty() {
                            checks.push(VerificationCheck {
                                tone: TONE_NEUTRAL,
                                label: "Nivel de la firma".into(),
                                detail: levels.as_str().into(),
                            });
                        }

                        ui.set_verification_success(intact);
                        ui.set_verification_status("".into());
                        ui.set_verification_checks(ModelRc::new(Rc::new(VecModel::from(checks))));
                    }
                    Err(error) => {
                        ui.set_verification_success(false);
                        ui.set_verification_checks(ModelRc::default());
                        ui.set_verification_status(error.to_string().into());
                    }
                }
            });
        });
    });

    let checker = Arc::clone(&update_service);
    let releases_for_check = Arc::clone(&available_release);
    let weak = ui.as_weak();
    ui.on_check_updates(move || {
        spawn_update_check(
            Arc::clone(&checker),
            Arc::clone(&releases_for_check),
            weak.clone(),
            true,
        );
    });

    let release_for_install = Arc::clone(&available_release);
    let updater = Arc::clone(&update_service);
    let installer = Arc::clone(&update_installer);
    let weak = ui.as_weak();
    ui.on_install_update(move || {
        let release = release_for_install
            .lock()
            .ok()
            .and_then(|value| value.clone());
        let Some(release) = release else { return };
        let document = weak.upgrade().and_then(|ui| {
            ui.set_update_busy(true);
            ui.set_update_status("Descargando y verificando…".into());
            let value = ui.get_document_path().to_string();
            (!value.is_empty()).then(|| PathBuf::from(value))
        });
        let updater = Arc::clone(&updater);
        let installer = Arc::clone(&installer);
        let weak = weak.clone();
        std::thread::spawn(move || {
            let result = (|| {
                let package = updater.download_verified(&release)?;
                let status_weak = weak.clone();
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = status_weak.upgrade() {
                        ui.set_update_status("Esperando la autorización del sistema…".into());
                    }
                })?;
                installer.install(&package)?;
                installer.relaunch(document.as_deref())?;
                Ok::<_, anyhow::Error>(())
            })();
            let _ = slint::invoke_from_event_loop(move || match result {
                Ok(()) => {
                    let _ = slint::quit_event_loop();
                }
                Err(error) => {
                    if let Some(ui) = weak.upgrade() {
                        ui.set_update_busy(false);
                        ui.set_update_status(error.to_string().into());
                    }
                }
            });
        });
    });

    spawn_update_check(
        Arc::clone(&update_service),
        Arc::clone(&available_release),
        ui.as_weak(),
        false,
    );

    if open_at_startup {
        ui.invoke_open_document()
    }
    ui.run()?;
    Ok(())
}
