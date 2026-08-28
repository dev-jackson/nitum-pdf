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
    assert_eq!(
        nitum_pdf::presentation::APPLICATION_ID,
        APP_ID,
        "the id the window announces and the id the desktop entry declares are \
         the same string"
    );

    // Ordering matters and is easy to get wrong silently: see
    // tests/application_id.rs, which proves the call does nothing at all when a
    // backend has not been selected first.
    let source = include_str!("../src/presentation.rs");
    let selects = source
        .find("BackendSelector::new().select()")
        .expect("a backend has to be selected before the id can be announced");
    let announces = source
        .find("set_xdg_app_id(APPLICATION_ID)")
        .expect("the application has to announce its id");
    assert!(
        selects < announces,
        "the backend must be selected before set_xdg_app_id, or the call fails \
         with NoPlatform and the window reaches the compositor unidentified"
    );
    let creates_window = source
        .find("AppWindow::new()")
        .expect("the window is created in presentation.rs");
    assert!(
        announces < creates_window,
        "the id has to be announced before any window exists"
    );
}

#[test]
fn every_platform_uses_the_same_icon_artwork() {
    let macos = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../packaging/native/build-macos-pkg.sh"
    ))
    .expect("the macOS build script is in packaging/native");
    assert!(
        macos.contains("data/com.nitum.Pdf.png"),
        "the macOS bundle must be built from the application icon; it used to \
         rasterise data/nitum-family-mark.png, so macOS showed a blue wordmark \
         that has nothing to do with the product"
    );
    assert!(
        !macos.contains("nitum-family-mark"),
        "the family wordmark is not the application icon"
    );

    // macOS runners have `sips` but not `rsvg-convert`, so the master the bundle
    // downscales from is a committed PNG rendered from the same SVG Linux ships.
    let master = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../data/com.nitum.Pdf.png"
    ))
    .expect("the 1024 px master of the application icon ships in data/");
    let width = u32::from_be_bytes(master[16..20].try_into().expect("PNG header"));
    let height = u32::from_be_bytes(master[20..24].try_into().expect("PNG header"));
    assert_eq!(
        (width, height),
        (1024, 1024),
        "the master has to be large enough for the 512@2x entry of the iconset"
    );

    let plist = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../packaging/native/Info.plist.in"
    ))
    .expect("the Info.plist template is in packaging/native");
    assert!(
        plist.contains(&format!("<string>{APP_ID}</string>")),
        "the bundle identifier has to be the same {APP_ID} the rest of the \
         product uses"
    );
}

#[test]
fn the_package_refreshes_the_desktop_caches() {
    let script = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../packaging/native/build-linux-deb.sh"
    ))
    .expect("the Debian build script is in packaging/native");
    // A freshly installed application keeps showing a generic icon until the
    // icon theme cache is rebuilt, and is not offered for PDFs until the MIME
    // database is refreshed.
    assert!(
        script.contains("gtk-update-icon-cache"),
        "the package has to refresh the icon theme cache when it is installed"
    );
    assert!(
        script.contains("update-desktop-database"),
        "the package has to refresh the desktop database when it is installed"
    );
    assert!(
        script.contains("DEBIAN/postinst") && script.contains("DEBIAN/postrm"),
        "those refreshes belong in maintainer scripts, so they run on install \
         and on removal"
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
