use i_slint_backend_testing::{
    AccessibleRole, ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions,
};
use nitum_pdf::{AppWindow, PageItem, Theme, VerificationCheck};
use slint::{
    ComponentHandle, Image, LogicalSize, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString,
    VecModel,
};
use std::rc::Rc;

fn luminance(pixels: &[u8]) -> f64 {
    pixels
        .as_chunks::<4>()
        .0
        .iter()
        .map(|pixel| {
            0.2126 * f64::from(pixel[0])
                + 0.7152 * f64::from(pixel[1])
                + 0.0722 * f64::from(pixel[2])
        })
        .sum::<f64>()
        / (pixels.len() / 4) as f64
}

fn save_if_requested(name: &str, snapshot: &slint::SharedPixelBuffer<slint::Rgba8Pixel>) {
    let Some(directory) = std::env::var_os("NITUM_VISUAL_OUTPUT") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    std::fs::create_dir_all(&directory).unwrap();
    image::save_buffer(
        directory.join(name),
        snapshot.as_bytes(),
        snapshot.width(),
        snapshot.height(),
        image::ColorType::Rgba8,
    )
    .unwrap();
}

fn assert_accessible_controls(ui: &AppWindow, context: &str) {
    for role in [
        AccessibleRole::Button,
        AccessibleRole::Checkbox,
        AccessibleRole::TextInput,
    ] {
        let controls = ElementQuery::from_root(ui)
            .match_accessible_role(role)
            .find_all();
        for control in controls {
            let label = control.accessible_label().unwrap_or_default();
            assert!(
                !label.trim().is_empty(),
                "an accessible {role:?} in {context} has no label: {}",
                control.id().unwrap_or_else(|| "<unknown>".into())
            );
        }
    }
}

fn assert_accessible_action(ui: &AppWindow, label: &str, context: &str) {
    assert!(
        ElementHandle::find_by_accessible_label(ui, label)
            .next()
            .is_some(),
        "the {context} view does not expose the {label:?} action"
    );
}

#[test]
fn light_dark_and_signature_center_render_headlessly() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        threading: false,
        renderer_name: Some("software".into()),
    })))
    .unwrap();

    let ui = AppWindow::new().unwrap();
    ui.window().set_size(LogicalSize::new(1180.0, 820.0));
    ui.show().unwrap();

    assert_accessible_controls(&ui, "empty shell");
    assert_accessible_action(&ui, "Abrir un PDF", "empty shell");

    let light = ui.window().take_snapshot().unwrap();
    ui.global::<Theme>().set_dark(true);
    let dark = ui.window().take_snapshot().unwrap();
    assert!(luminance(light.as_bytes()) > luminance(dark.as_bytes()) + 35.0);

    ui.set_active_dialog(1);
    assert_accessible_controls(&ui, "signature center");
    assert_accessible_action(&ui, "Cerrar", "signature center");
    let signature_center = ui.window().take_snapshot().unwrap();
    assert_ne!(dark.as_bytes(), signature_center.as_bytes());
    assert_eq!(
        (signature_center.width(), signature_center.height()),
        (1180, 820)
    );

    save_if_requested("shell-light.png", &light);
    save_if_requested("shell-dark.png", &dark);
    save_if_requested("signature-center-dark.png", &signature_center);

    let mut pixels = SharedPixelBuffer::<Rgba8Pixel>::new(600, 800);
    for (index, pixel) in pixels
        .make_mut_bytes()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .enumerate()
    {
        let y = index / 600;
        let line = y > 90 && y % 42 < 3;
        pixel.copy_from_slice(if line {
            &[210, 215, 225, 255]
        } else {
            &[255, 255, 255, 255]
        });
    }
    let page_image = Image::from_rgba8(pixels);
    ui.set_document_open(true);
    ui.set_document_title("Contrato de ejemplo.pdf".into());
    ui.set_page_count(2);
    ui.set_current_page(1);
    ui.set_pages(ModelRc::new(Rc::new(VecModel::from(vec![
        PageItem {
            number: 1,
            image: page_image.clone(),
            aspect: 1.333,
            loaded: true,
            error: "".into(),
        },
        PageItem {
            number: 2,
            image: page_image,
            aspect: 1.333,
            loaded: true,
            error: "".into(),
        },
    ]))));
    ui.set_active_dialog(0);
    let document = ui.window().take_snapshot().unwrap();
    save_if_requested("continuous-document-dark.png", &document);

    ui.set_identity_name("Identidad de prueba".into());
    ui.set_identity_path("identity.p12".into());
    ui.set_active_dialog(2);
    assert_accessible_controls(&ui, "signing flow");
    // Every step keeps two ways out: back to the previous step, and close.
    assert_accessible_action(&ui, "Atrás", "signing flow");
    assert_accessible_action(&ui, "Cerrar", "signing flow");
    let signing = ui.window().take_snapshot().unwrap();
    assert_ne!(document.as_bytes(), signing.as_bytes());
    save_if_requested("signing-flow-dark.png", &signing);

    ui.set_appearance_options(ModelRc::new(Rc::new(VecModel::from(vec![
        "Firma personal".into(),
        "Iniciales contrato".into(),
    ]))));
    ui.set_active_dialog(7);
    assert_accessible_controls(&ui, "appearance library");
    assert_accessible_action(&ui, "Cerrar", "appearance library");
    let _transition_frame = ui.window().take_snapshot().unwrap();
    let appearance_library = ui.window().take_snapshot().unwrap();
    assert_ne!(signing.as_bytes(), appearance_library.as_bytes());
    save_if_requested("appearance-library-dark.png", &appearance_library);

    ui.set_identity_options(ModelRc::new(Rc::new(VecModel::from(vec![
        "Identidad corporativa".into(),
        "Firma personal".into(),
    ]))));
    ui.set_active_dialog(8);
    assert_accessible_controls(&ui, "identity library");
    assert_accessible_action(&ui, "Cerrar", "identity library");
    for _ in 0..3 {
        let _transition_frame = ui.window().take_snapshot().unwrap();
    }
    let identity_library = ui.window().take_snapshot().unwrap();
    assert_ne!(appearance_library.as_bytes(), identity_library.as_bytes());
    save_if_requested("identity-library-dark.png", &identity_library);

    ui.set_active_dialog(0);
    // Moving an existing placement: the preview shows the box at its current
    // spot, at the real size it will have in the PDF.
    ui.set_signature_box_width(0.36);
    ui.set_signature_box_height(0.07);
    ui.set_signature_page(0);
    ui.set_signature_x(0.42);
    ui.set_signature_y(0.38);
    ui.set_signature_position_set(true);
    ui.set_placement_mode(true);
    ui.set_viewer_status("".into());
    let placement = ui.window().take_snapshot().unwrap();
    assert_ne!(document.as_bytes(), placement.as_bytes());
    save_if_requested("signature-placement-dark.png", &placement);

    ui.set_placement_mode(false);
    ui.set_viewer_status("".into());
    ui.set_active_dialog(2);
    ui.window().set_size(LogicalSize::new(720.0, 560.0));
    let compact_signing = ui.window().take_snapshot().unwrap();
    assert_eq!(
        (compact_signing.width(), compact_signing.height()),
        (720, 560)
    );
    assert_ne!(compact_signing.as_bytes(), signing.as_bytes());
    save_if_requested("signing-flow-compact-dark.png", &compact_signing);

    ui.set_active_dialog(0);
    let _transition_frame = ui.window().take_snapshot().unwrap();
    let compact_document = ui.window().take_snapshot().unwrap();
    assert_eq!(
        (compact_document.width(), compact_document.height()),
        (720, 560)
    );
    save_if_requested("document-compact-dark.png", &compact_document);

    ui.set_selection_mode(true);
    let _transition_frame = ui.window().take_snapshot().unwrap();
    let selection_mode = ui.window().take_snapshot().unwrap();
    assert_ne!(compact_document.as_bytes(), selection_mode.as_bytes());
    save_if_requested("text-selection-compact-dark.png", &selection_mode);
    ui.set_selection_mode(false);

    ui.set_search_open(true);
    let compact_search = ui.window().take_snapshot().unwrap();
    assert_ne!(compact_document.as_bytes(), compact_search.as_bytes());
    save_if_requested("search-compact-dark.png", &compact_search);
    ui.set_search_open(false);

    ui.set_active_dialog(7);
    let _transition_frame = ui.window().take_snapshot().unwrap();
    let compact_appearances = ui.window().take_snapshot().unwrap();
    assert_eq!(
        (compact_appearances.width(), compact_appearances.height()),
        (720, 560)
    );
    save_if_requested("appearance-library-compact-dark.png", &compact_appearances);

    ui.set_active_dialog(8);
    for _ in 0..3 {
        let _transition_frame = ui.window().take_snapshot().unwrap();
    }
    let compact_identities = ui.window().take_snapshot().unwrap();
    assert_eq!(
        (compact_identities.width(), compact_identities.height()),
        (720, 560)
    );
    save_if_requested("identity-library-compact-dark.png", &compact_identities);

    ui.window().set_size(LogicalSize::new(1180.0, 820.0));
    ui.set_current_version("0.6.0".into());
    ui.set_update_version("0.7.0".into());
    ui.set_update_available(true);
    ui.set_update_status("La versión 0.7.0 está lista para descargar.".into());
    ui.set_active_dialog(4);
    assert_accessible_controls(&ui, "updater");
    assert_accessible_action(&ui, "Ahora no", "updater");
    let update_available = ui.window().take_snapshot().unwrap();
    assert_ne!(update_available.as_bytes(), signing.as_bytes());
    save_if_requested("update-available-dark.png", &update_available);

    ui.set_update_available(false);
    ui.set_update_status("Ya tienes la versión más reciente.".into());
    let up_to_date = ui.window().take_snapshot().unwrap();
    assert_ne!(update_available.as_bytes(), up_to_date.as_bytes());
    save_if_requested("up-to-date-dark.png", &up_to_date);

    ui.global::<Theme>().set_dark(false);
    ui.set_active_dialog(2);
    let _transition_frame = ui.window().take_snapshot().unwrap();
    let signing_light = ui.window().take_snapshot().unwrap();
    assert!(luminance(signing_light.as_bytes()) > luminance(signing.as_bytes()) + 20.0);
    save_if_requested("signing-flow-light.png", &signing_light);
    ui.set_certification_permission(2);
    ui.set_active_dialog(1);
    ui.set_active_dialog(2);
    let _transition_frame = ui.window().take_snapshot().unwrap();
    let certification_light = ui.window().take_snapshot().unwrap();
    assert!(
        certification_light
            .as_bytes()
            .iter()
            .any(|value| *value != 0)
    );
    save_if_requested("certification-flow-light.png", &certification_light);
    ui.set_certification_permission(-1);

    ui.set_active_dialog(7);
    let _transition_frame = ui.window().take_snapshot().unwrap();
    let appearances_light = ui.window().take_snapshot().unwrap();
    assert!(luminance(appearances_light.as_bytes()) > luminance(appearance_library.as_bytes()));
    save_if_requested("appearance-library-light.png", &appearances_light);

    ui.set_active_dialog(8);
    let _transition_frame = ui.window().take_snapshot().unwrap();
    let identities_light = ui.window().take_snapshot().unwrap();
    assert!(luminance(identities_light.as_bytes()) > luminance(identity_library.as_bytes()));
    save_if_requested("identity-library-light.png", &identities_light);

    // The password, verification and token dialogs had no capture at all, which
    // is how their defects survived every review: DESIGN.md rule 9 asks for a
    // reproducible capture of each important flow.
    ui.set_unlock_status("La contraseña no es correcta.".into());
    ui.set_active_dialog(3);
    let _transition_frame = ui.window().take_snapshot().unwrap();
    let unlock_light = ui.window().take_snapshot().unwrap();
    assert_accessible_controls(&ui, "unlock");
    assert_accessible_action(&ui, "Abrir PDF", "unlock");
    save_if_requested("unlock-light.png", &unlock_light);
    ui.set_unlock_status("".into());

    // Intact and covered, but from a certificate nobody vouches for: the case
    // the old single green sentence hid.
    ui.set_verification_success(true);
    ui.set_verification_checks(ModelRc::new(Rc::new(VecModel::from(vec![
        VerificationCheck {
            tone: 1,
            label: "El documento no se ha alterado".into(),
            detail: "El contenido coincide exactamente con lo que se firmó.".into(),
        },
        VerificationCheck {
            tone: 1,
            label: "La firma cubre todo el documento".into(),
            detail: "No hay páginas ni cambios fuera de lo firmado.".into(),
        },
        VerificationCheck {
            tone: 2,
            label: "No podemos confirmar quién emitió el certificado".into(),
            detail:
                "La firma es válida, pero nadie en este equipo avala la identidad del firmante."
                    .into(),
        },
        VerificationCheck {
            tone: 0,
            label: "Firmado por Identidad de prueba".into(),
            detail: "1 firma, 1 sello de tiempo. Sin certificación DocMDP.".into(),
        },
    ]))));
    ui.set_active_dialog(5);
    let _transition_frame = ui.window().take_snapshot().unwrap();
    let verification_light = ui.window().take_snapshot().unwrap();
    assert_accessible_controls(&ui, "verification");
    assert_accessible_action(&ui, "Cerrar", "verification");
    save_if_requested("verification-light.png", &verification_light);

    ui.global::<Theme>().set_dark(true);
    ui.set_verification_success(false);
    ui.set_verification_checks(ModelRc::default());
    ui.set_verification_status("Este PDF no contiene ninguna firma digital.".into());
    let verification_empty = ui.window().take_snapshot().unwrap();
    save_if_requested("verification-empty-dark.png", &verification_empty);

    ui.set_token_status("Encontramos 2 dispositivos.".into());
    ui.set_token_options(ModelRc::new(Rc::new(VecModel::from(vec![
        SharedString::from("YubiKey 5 NFC · 31245678"),
        SharedString::from("DNIe 4.0 · 00998877"),
    ]))));
    ui.set_active_dialog(6);
    let _transition_frame = ui.window().take_snapshot().unwrap();
    let tokens_dark = ui.window().take_snapshot().unwrap();
    assert_accessible_controls(&ui, "tokens");
    save_if_requested("tokens-dark.png", &tokens_dark);
}
