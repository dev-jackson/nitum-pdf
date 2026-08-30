//! Dragging over the page must set the signature's position **and** its size.
//!
//! Acrobat and FirmaEC both ask you to draw a rectangle, and that is what the
//! interface promises: "Pulsa y arrastra sobre el documento para dibujar el área
//! de la firma". This drives a real pointer press, move and release through the
//! window and checks the area that comes out, so "it only ever gives me a fixed
//! box" is a claim the test suite can settle.

use i_slint_backend_testing::{ElementHandle, ElementQuery};
use nitum_pdf::{AppWindow, PageItem};
use slint::platform::PointerEventButton;
use slint::platform::WindowEvent;
use slint::{ComponentHandle, LogicalPosition, ModelRc, VecModel};
use std::rc::Rc;

/// A window showing one page, already in placement mode.
fn window_in_placement_mode() -> AppWindow {
    // The tests share one process, and the platform may only be set once.
    static BACKEND: std::sync::Once = std::sync::Once::new();
    BACKEND.call_once(i_slint_backend_testing::init_no_event_loop);
    let ui = AppWindow::new().expect("window");
    ui.window().set_size(slint::LogicalSize::new(1180.0, 820.0));

    ui.set_document_open(true);
    ui.set_page_count(1);
    ui.set_current_page(1);
    ui.set_page_aspect(1.414);
    ui.set_pages(ModelRc::new(Rc::new(VecModel::from(vec![PageItem {
        number: 1,
        image: slint::Image::default(),
        aspect: 1.414,
        loaded: false,
        error: "".into(),
    }]))));
    ui.set_placement_mode(true);
    // The page list only lays its rows out once a frame has been produced, and
    // hit-testing needs that layout to exist.
    let _ = ui.window().take_snapshot();
    ui
}

/// The overlay that covers the page while an area is being drawn.
fn placement_surface(ui: &AppWindow) -> ElementHandle {
    ElementQuery::from_root(ui)
        .match_id("AppWindow::place")
        .find_first()
        .expect("the placement surface is present while placing a signature")
}

/// Presses, moves and releases inside the visible part of the window.
///
/// The page is taller than the window, so its centre is off-screen; a helper
/// that always starts from an element's centre would press outside the window
/// and deliver nothing. These are window coordinates, which is where a person's
/// pointer actually is.
fn drag(ui: &AppWindow, from: (f32, f32), to: (f32, f32)) {
    let window = ui.window();
    window.dispatch_event(WindowEvent::PointerPressed {
        position: LogicalPosition::new(from.0, from.1),
        button: PointerEventButton::Left,
    });
    // A couple of intermediate moves, the way a real drag arrives.
    for step in 1..=4 {
        let fraction = step as f32 / 4.0;
        window.dispatch_event(WindowEvent::PointerMoved {
            position: LogicalPosition::new(
                from.0 + (to.0 - from.0) * fraction,
                from.1 + (to.1 - from.1) * fraction,
            ),
        });
    }
    window.dispatch_event(WindowEvent::PointerReleased {
        position: LogicalPosition::new(to.0, to.1),
        button: PointerEventButton::Left,
    });
}

/// One test, because the Slint backend is per-process and per-thread: parallel
/// tests each try to create their own and only the first succeeds.
#[test]
fn dragging_draws_the_signature_area() {
    // A drag records position and size, and the size is whatever was drawn.
    let ui = window_in_placement_mode();
    let surface = placement_surface(&ui);
    let origin = surface.absolute_position();
    let size = surface.size();

    let from = (origin.x + 100.0, origin.y + 100.0);
    let to = (origin.x + 500.0, origin.y + 260.0);
    drag(&ui, from, to);

    assert!(
        ui.get_signature_area_set(),
        "a drag has to record an area, not just a point"
    );
    assert!(ui.get_signature_position_set());
    assert!(
        !ui.get_placement_mode(),
        "releasing ends placement and returns to the details"
    );
    assert_eq!(
        ui.get_active_dialog(),
        2,
        "releasing the drag returns to the signing details, so the flow reads \
         draw then confirm"
    );

    let width = (ui.get_signature_x2() - ui.get_signature_x()).abs();
    let height = (ui.get_signature_y2() - ui.get_signature_y()).abs();
    let expected_width = 400.0 / size.width;
    let expected_height = 160.0 / size.height;
    assert!(
        (width - expected_width).abs() < 0.01,
        "the drawn width follows the pointer: expected {expected_width}, got {width}"
    );
    assert!(
        (height - expected_height).abs() < 0.01,
        "the drawn height follows the pointer: expected {expected_height}, got {height}"
    );

    // A different drag gives a different box. Two drags that produced the same
    // rectangle would be the "it only ever gives me one fixed box" complaint.
    ui.set_signature_area_set(false);
    ui.set_signature_position_set(false);
    ui.set_active_dialog(0);
    ui.set_placement_mode(true);
    let _ = ui.window().take_snapshot();

    drag(
        &ui,
        (origin.x + 60.0, origin.y + 60.0),
        (origin.x + 860.0, origin.y + 300.0),
    );
    let wider = (ui.get_signature_x2() - ui.get_signature_x()).abs();
    assert!(
        wider > width * 1.8,
        "a drag twice as wide gives a much wider area: {width} then {wider}"
    );
}
