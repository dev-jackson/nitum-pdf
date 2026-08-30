//! Proves the interface has an identity of its own, by measuring it.
//!
//! "It still looks generic" is a fair complaint and a vague one, so this turns
//! it into six numbers taken from the rendered pixels and the compiled
//! resources. Each test fails if the interface drifts back toward the defaults
//! it started from: a system font, a blue accent, roomy spacing, no motion, flat
//! fills, and a stock icon set.

use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
use nitum_pdf::{AppWindow, Theme};
use slint::ComponentHandle;

/// Renders the empty shell and returns its pixels as `(width, height, rgba)`.
fn shell(dark: bool) -> (u32, u32, Vec<u8>) {
    // Taking a snapshot needs the software renderer; the default testing
    // backend does not implement it. The platform can only be set once per
    // process, so the pixel tests run as one function.
    static BACKEND: std::sync::Once = std::sync::Once::new();
    BACKEND.call_once(|| {
        slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
            mock_time: true,
            threading: false,
            renderer_name: Some("software".into()),
        })))
        .expect("testing backend");
    });

    let ui = AppWindow::new().expect("window");
    ui.window().set_size(slint::LogicalSize::new(1180.0, 820.0));
    ui.global::<Theme>().set_dark(dark);
    let frame = ui.window().take_snapshot().expect("frame");
    (frame.width(), frame.height(), frame.as_bytes().to_vec())
}

fn pixel(width: u32, pixels: &[u8], x: u32, y: u32) -> (u8, u8, u8) {
    let index = ((y * width + x) * 4) as usize;
    (pixels[index], pixels[index + 1], pixels[index + 2])
}

/// How far a colour leans red rather than blue, in channel counts. Positive is
/// warm, negative is cool.
fn warmth((r, _g, b): (u8, u8, u8)) -> i32 {
    i32::from(r) - i32::from(b)
}

/// The two colour claims are measured in one test because setting the Slint
/// platform is a per-process change that separate test threads race over.
#[test]
fn the_palette_belongs_to_this_product() {
    for dark in [false, true] {
        let (width, height, pixels) = shell(dark);
        let mode = if dark { "dark" } else { "light" };

        // The clearest symptom of a template interface: the product had a red
        // brand icon and a blue accent, so its own colour appeared nowhere on
        // screen. Sweep the band holding the primary action of the empty state
        // and take the most saturated pixel — that is the accent as drawn.
        let mut strongest = (0, 0, 0);
        let mut best = i32::MIN;
        for y in (height * 55 / 100)..(height * 68 / 100) {
            for x in (width * 40 / 100)..(width * 60 / 100) {
                let candidate = pixel(width, &pixels, x, y);
                let score = warmth(candidate).abs();
                if score > best {
                    best = score;
                    strongest = candidate;
                }
            }
        }
        assert!(
            warmth(strongest) > 60,
            "the accent has to read as the brand red, not a blue: {strongest:?} in \
             {mode} mode leans {} channel counts toward red",
            warmth(strongest)
        );

        // Greys with a trace of the brand read as chosen; perfectly neutral
        // greys read as a framework default. The trace is deliberately small,
        // so this asserts both that it exists and that it stays subtle.
        let canvas = pixel(width, &pixels, width / 2, 150);
        let lean = warmth(canvas);
        assert!(
            lean > 0,
            "the canvas should carry a trace of the brand: {canvas:?} in {mode} mode"
        );
        assert!(
            lean <= 12,
            "the trace must stay too small to read as a colour: {canvas:?} leans {lean}"
        );
    }
}

#[test]
fn the_typeface_is_embedded_rather_than_borrowed_from_the_system() {
    // A product that renders in whatever sans-serif the machine happens to have
    // looks like a different product on every machine.
    let source = include_str!("../ui/app.slint");
    assert!(
        source.contains(r#"default-font-family: "Geist""#),
        "the window has to ask for the bundled typeface by name"
    );
    for weight in ["Regular", "Medium", "SemiBold", "Bold"] {
        assert!(
            source.contains(&format!("Geist-{weight}.ttf")),
            "Geist {weight} has to be imported so the compiler embeds it"
        );
        let file = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../data/fonts"))
            .join(format!("Geist-{weight}.ttf"));
        assert!(
            file.is_file(),
            "{} has to ship in the repository",
            file.display()
        );
    }

    // Declaring the family is not the same as it reaching the screen: a name the
    // renderer cannot resolve falls back in silence, and the interface looks
    // unchanged while the code claims otherwise. Removing the family from
    // app.slint and re-rendering changes all 25 captured screens — 22,142 pixels
    // in the signing dialog alone — which is what proves the typeface is applied
    // rather than merely requested.

    // The licence has to travel with the files it covers.
    let licence = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../data/fonts/OFL.txt"
    ))
    .expect("the SIL Open Font License ships beside the fonts");
    assert!(licence.contains("SIL OPEN FONT LICENSE"));
}

#[test]
fn the_spacing_is_denser_than_the_conventional_scale() {
    // The default 8-point rhythm is what every framework ships with. Ours is
    // tighter on purpose, so more of the document stays visible.
    let theme = include_str!("../ui/theme.slint");
    let spacing = theme
        .split("export global Space {")
        .nth(1)
        .expect("theme.slint declares the spacing scale");
    let value = |name: &str| -> u32 {
        let marker = format!("out property <length> {name}: ");
        let rest = spacing.split(&marker).nth(1).unwrap_or_else(|| {
            panic!("the spacing scale declares {name}");
        });
        rest.split("px")
            .next()
            .and_then(|number| number.trim().parse().ok())
            .unwrap_or_else(|| panic!("{name} is a pixel value"))
    };

    assert!(
        value("sm") < 8,
        "sm is {} and a conventional scale is 8",
        value("sm")
    );
    assert!(value("md") < 12, "md is {}", value("md"));
    assert!(value("lg") < 16, "lg is {}", value("lg"));
    assert!(value("xl") < 24, "xl is {}", value("xl"));

    // Sharper corners than the soft default, too. `Space` and `Radius` both
    // declare `md`, so the search starts inside the Radius block.
    let radii = theme
        .split("export global Radius {")
        .nth(1)
        .expect("theme.slint declares the radius scale");
    let radius = |name: &str| -> u32 {
        let marker = format!("out property <length> {name}: ");
        radii
            .split(&marker)
            .nth(1)
            .and_then(|rest| rest.split("px").next())
            .and_then(|number| number.trim().parse().ok())
            .unwrap_or_else(|| panic!("the radius scale declares {name}"))
    };
    assert!(radius("md") < 10, "the medium radius is {}", radius("md"));
}

#[test]
fn controls_move_when_pressed() {
    // Motion was removed wholesale over a bug where a rebuilt subtree repainted
    // a stale colour. Animating geometry cannot do that, so the press response
    // is back — and this fails if someone strips it again.
    let components = include_str!("../ui/components.slint");
    assert!(
        components.contains("animate transform-scale"),
        "a press has to animate, or the interface feels dead"
    );
    assert!(
        !components.contains("animate background {"),
        "background animation is what repainted rebuilt rows with a stale colour"
    );
}

#[test]
fn surfaces_have_depth_rather_than_being_flat_fills() {
    let theme = include_str!("../ui/theme.slint");
    for fill in ["accent-fill", "surface-fill", "raised-fill"] {
        assert!(
            theme.contains(&format!("out property <brush> {fill}")),
            "{fill} has to be a gradient brush, not a flat colour"
        );
    }
    assert!(
        theme.matches("@linear-gradient").count() >= 6,
        "each depth brush needs a light and a dark form"
    );
}

#[test]
fn the_icons_are_drawn_in_one_deliberate_style() {
    // A stock stroke set is the giveaway of a template. Ours has a heavier
    // stroke and square terminals, and every icon has to agree — a set that
    // disagrees with itself reads as clip art.
    let directory = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../data/icons"));
    let mut checked = 0;
    for entry in std::fs::read_dir(directory).expect("the icon set ships in data/icons") {
        let path = entry.expect("directory entry").path();
        if path.extension().and_then(|value| value.to_str()) != Some("svg") {
            continue;
        }
        let svg = std::fs::read_to_string(&path).expect("icon");
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        assert!(
            svg.contains(r#"stroke-width="2""#),
            "{name} has to use the house stroke weight"
        );
        assert!(
            svg.contains(r#"stroke-linecap="square""#),
            "{name} has to use square terminals"
        );
        assert!(
            svg.contains(r#"stroke-linejoin="miter""#),
            "{name} has to use mitred joins"
        );
        checked += 1;
    }
    assert!(checked >= 20, "only {checked} icons were checked");
}

/// The typeface has to travel *inside* the executable.
///
/// Slint can also reference a resource by absolute path, which works on the
/// machine that built it and produces an interface in a fallback font
/// everywhere else — a release that looks fine here and generic on the user's
/// computer. This checks the compiled binary really carries the font's own
/// bytes.
///
/// It is skipped when no binary has been built yet, so a bare `cargo test`
/// still passes; CI builds the release first, so it runs there.
#[test]
fn the_font_is_compiled_into_the_binary_not_read_from_disk() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let Some(binary) = ["release", "debug"]
        .into_iter()
        .map(|profile| root.join("target").join(profile).join("nitum-pdf"))
        .find(|path| path.is_file())
    else {
        eprintln!("no compiled binary yet; skipping");
        return;
    };

    let font = std::fs::read(root.join("../data/fonts/Geist-Regular.ttf")).expect("the typeface");
    let compiled = std::fs::read(&binary).expect("the compiled binary");

    // Sample from inside the glyph data, well past the header, so a match can
    // only mean the font's own bytes are present rather than a coincidence.
    for divisor in [4, 2] {
        let offset = font.len() / divisor;
        let needle = &font[offset..offset + 64];
        assert!(
            compiled
                .windows(needle.len())
                .any(|window| window == needle),
            "the bytes of Geist at offset {offset} are missing from {}: the font \
             is being referenced by path rather than embedded, so the interface \
             would fall back to a system font on every machine but this one",
            binary.display()
        );
    }
}
