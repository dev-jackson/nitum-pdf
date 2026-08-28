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

#[test]
fn setting_the_id_before_a_backend_exists_fails() {
    // This is the trap: without a backend there is no global context to write
    // into, so the call reports NoPlatform rather than doing anything.
    let result = slint::set_xdg_app_id(APPLICATION_ID);
    assert!(
        result.is_err(),
        "set_xdg_app_id is expected to fail with no backend selected; if this \
         ever starts succeeding, the ordering guard in announce_application_id \
         is no longer needed"
    );
}

#[test]
fn setting_the_id_after_selecting_a_backend_succeeds() {
    // Each integration test binary is its own process, so this one gets a fresh
    // context and can select a backend without the test above interfering.
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
