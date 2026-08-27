//! Every colour pair the interface actually renders must clear WCAG 2.2.
//!
//! Text and its background need 4.5:1 (SC 1.4.3, <https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html>).
//! The visual boundary of a control and a focus indicator need 3:1
//! (SC 1.4.11, <https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html>).
//!
//! The values are read out of `ui/theme.slint` so the test fails when someone
//! edits a token rather than when someone remembers to update a list here.

use std::collections::HashMap;

const TEXT: f64 = 4.5;
const UI: f64 = 3.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Light,
    Dark,
}

impl Mode {
    fn name(self) -> &'static str {
        match self {
            Mode::Light => "claro",
            Mode::Dark => "oscuro",
        }
    }
}

#[derive(Clone, Copy)]
struct Rgba {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

fn parse(colour: &str) -> Rgba {
    let body = colour.trim_start_matches('#');
    let channel = |index: usize| {
        f64::from(u8::from_str_radix(&body[index..index + 2], 16).expect("hex channel"))
    };
    assert!(
        body.len() == 6 || body.len() == 8,
        "unsupported colour literal: {colour}"
    );
    Rgba {
        r: channel(0),
        g: channel(2),
        b: channel(4),
        a: if body.len() == 8 {
            channel(6) / 255.0
        } else {
            1.0
        },
    }
}

/// Flatten a translucent colour over the opaque surface it is painted on.
fn flatten(colour: Rgba, background: Rgba) -> Rgba {
    Rgba {
        r: colour.r * colour.a + background.r * (1.0 - colour.a),
        g: colour.g * colour.a + background.g * (1.0 - colour.a),
        b: colour.b * colour.a + background.b * (1.0 - colour.a),
        a: 1.0,
    }
}

fn luminance(colour: Rgba) -> f64 {
    let linear = |value: f64| {
        let value = value / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(colour.r) + 0.7152 * linear(colour.g) + 0.0722 * linear(colour.b)
}

fn ratio(foreground: Rgba, background: Rgba) -> f64 {
    let foreground = luminance(flatten(foreground, background));
    let background = luminance(background);
    let (high, low) = if foreground > background {
        (foreground, background)
    } else {
        (background, foreground)
    };
    (high + 0.05) / (low + 0.05)
}

/// Read `name: dark ? <dark> : <light>;` and `name: <both>;` out of theme.slint.
fn tokens() -> HashMap<String, (String, String)> {
    let source = include_str!("../ui/theme.slint");
    let mut found = HashMap::new();
    for line in source.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("out property <color> ") else {
            continue;
        };
        let Some((name, value)) = rest.split_once(':') else {
            continue;
        };
        let value = value.trim().trim_end_matches(';').trim();
        let entry = if let Some(rest) = value.strip_prefix("dark ? ") {
            let (dark, light) = rest.split_once(" : ").expect("dark/light pair");
            (dark.trim().to_owned(), light.trim().to_owned())
        } else {
            (value.to_owned(), value.to_owned())
        };
        found.insert(name.trim().to_owned(), entry);
    }
    found
}

fn colour(tokens: &HashMap<String, (String, String)>, name: &str, mode: Mode) -> Rgba {
    let entry = tokens
        .get(name)
        .unwrap_or_else(|| panic!("theme.slint has no token named {name}"));
    parse(match mode {
        Mode::Dark => &entry.0,
        Mode::Light => &entry.1,
    })
}

#[test]
fn every_theme_pair_meets_wcag() {
    let tokens = tokens();
    let mut failures = Vec::new();

    for mode in [Mode::Light, Mode::Dark] {
        let get = |name: &str| colour(&tokens, name, mode);
        let surface = get("surface");
        let canvas = get("canvas");
        let raised = get("raised");
        let accent = get("accent");

        let pairs: [(&str, Rgba, Rgba, f64); 20] = [
            ("text / surface", get("text"), surface, TEXT),
            (
                "text-secondary / surface",
                get("text-secondary"),
                surface,
                TEXT,
            ),
            (
                "text-tertiary / surface",
                get("text-tertiary"),
                surface,
                TEXT,
            ),
            ("text / canvas", get("text"), canvas, TEXT),
            (
                "text-secondary / canvas",
                get("text-secondary"),
                canvas,
                TEXT,
            ),
            ("text / raised", get("text"), raised, TEXT),
            (
                "text-secondary / raised",
                get("text-secondary"),
                raised,
                TEXT,
            ),
            ("accent / surface", accent, surface, TEXT),
            ("success / surface", get("success"), surface, TEXT),
            ("warning / surface", get("warning"), surface, TEXT),
            ("danger / surface", get("danger"), surface, TEXT),
            (
                "text-on-accent / accent",
                get("text-on-accent"),
                accent,
                TEXT,
            ),
            (
                "accent / accent-soft",
                accent,
                flatten(get("accent-soft"), surface),
                TEXT,
            ),
            (
                "success / success-soft",
                get("success"),
                flatten(get("success-soft"), surface),
                TEXT,
            ),
            (
                "warning / warning-soft",
                get("warning"),
                flatten(get("warning-soft"), surface),
                TEXT,
            ),
            (
                "danger / danger-soft",
                get("danger"),
                flatten(get("danger-soft"), surface),
                TEXT,
            ),
            ("border-strong / surface", get("border-strong"), surface, UI),
            ("focus-ring / surface", get("focus-ring"), surface, UI),
            ("focus-ring / canvas", get("focus-ring"), canvas, UI),
            ("accent / surface [control]", accent, surface, UI),
        ];

        for (name, foreground, background, required) in pairs {
            let value = ratio(foreground, background);
            if value < required {
                failures.push(format!(
                    "{}: {name} is {value:.2}:1, needs {required}:1",
                    mode.name()
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "contraste insuficiente:\n{}",
        failures.join("\n")
    );
}
