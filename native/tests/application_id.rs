//! Proves the application id is actually applied, not merely requested.
//!
//! `slint::set_xdg_app_id` writes into the global Slint context and returns
//! `PlatformError::NoPlatform` while no backend has been selected. Calling it
//! before the backend is up therefore does nothing, and dropping the result
//! makes that failure invisible: the window then reaches the compositor with no
//! application id, the shell cannot match it to `com.nitum.Pdf.desktop`, and it
//! shows a second, generic entry instead of ours.
//!
//! These tests pin the ordering that makes the call succeed, and the one that
//! makes it fail, so the bug cannot come back unnoticed.

use nitum_pdf::presentation::APPLICATION_ID;

/// Both halves live in one test because the order is the whole point, and
/// because selecting a backend is a one-way change to the process state that a
/// second test could otherwise observe out of order.
#[test]
fn the_id_only_applies_once_a_backend_has_been_selected() {
    // The trap: with no backend there is no context to write into, so the call
    // reports NoPlatform instead of doing anything. This is what the old code
    // did on every launch, and it dropped the error.
    assert!(
        slint::set_xdg_app_id(APPLICATION_ID).is_err(),
        "set_xdg_app_id is expected to fail while no backend is selected; if it \
         ever starts succeeding, the ordering guard in announce_application_id \
         is no longer load-bearing and this test should be revisited"
    );

    i_slint_backend_testing::init_no_event_loop();

    slint::set_xdg_app_id(APPLICATION_ID)
        .expect("with a backend selected the application id must actually be applied");
}

#[test]
fn the_id_is_the_one_the_desktop_entry_declares() {
    let entry = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../data/com.nitum.Pdf.desktop"
    ))
    .expect("the desktop entry ships in data/");
    for key in ["Icon", "StartupWMClass"] {
        let value = entry
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{key}=")))
            .unwrap_or_else(|| panic!("the desktop entry has no {key} key"))
            .trim();
        assert_eq!(
            value, APPLICATION_ID,
            "{key} has to name the same id the window announces"
        );
    }
}
