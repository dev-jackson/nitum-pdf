//! The desktop has to recognise our window as belonging to our launcher.
//!
//! Four things carry the same identifier and all four must agree:
//!
//! - the desktop entry's file name, which is what Wayland matches an `app_id` to;
//! - `StartupWMClass`, which is what X11 matches a window's `WM_CLASS` to;
//! - `Icon`, which names the file installed under `hicolor`;
//! - the `slint::set_xdg_app_id` call the application makes at startup.
//!
//! When `StartupWMClass` said `nitum-pdf` while the window announced
//! `com.nitum.Pdf`, the shell treated the running window as a different,
//! unknown application: a second entry in the dock, no grouping with the
//! launcher, and the generic placeholder icon instead of ours.

const APP_ID: &str = "com.nitum.Pdf";

fn desktop_entry() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../data/com.nitum.Pdf.desktop"
    ))
    .expect("the desktop entry is installed from data/com.nitum.Pdf.desktop")
}

fn value_of(entry: &str, key: &str) -> String {
    entry
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}=")))
        .unwrap_or_else(|| panic!("the desktop entry has no {key} key"))
        .trim()
        .to_owned()
}

#[test]
fn the_desktop_entry_matches_the_application_id() {
    let entry = desktop_entry();
    assert_eq!(
        value_of(&entry, "StartupWMClass"),
        APP_ID,
        "X11 matches a window to its launcher through WM_CLASS, which winit sets \
         from the XDG application id"
    );
    assert_eq!(
        value_of(&entry, "Icon"),
        APP_ID,
        "the icon is installed as {APP_ID}.svg under hicolor"
    );
}

#[test]
fn the_application_announces_that_same_id() {
    let source = include_str!("../src/presentation.rs");
    assert!(
        source.contains(&format!("set_xdg_app_id(\"{APP_ID}\")")),
        "the application must announce {APP_ID} before its window is shown, or \
         Wayland cannot find com.nitum.Pdf.desktop"
    );
}

#[test]
fn the_package_installs_both_under_that_id() {
    let script = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../packaging/native/build-linux-deb.sh"
    ))
    .expect("the Debian build script is in packaging/native");
    assert!(
        script.contains(&format!("/usr/share/applications/{APP_ID}.desktop")),
        "Wayland looks the window up by the desktop file's name, so it has to be \
         installed as {APP_ID}.desktop"
    );
    assert!(
        script.contains(&format!(
            "/usr/share/icons/hicolor/scalable/apps/{APP_ID}.svg"
        )),
        "the Icon key names {APP_ID}, so the file has to be installed under that name"
    );
}
