//! The visible signature has to carry its own information.
//!
//! It used to be built from `TextConfig::default()`, whose `lines` field is
//! empty, so a visible signature was stamped onto the page as a blank
//! rectangle: no signer, no issuer, no date. The only way to make anything
//! appear was to supply an image by hand, which is backwards — everything the
//! stamp needs is already inside the certificate doing the signing.
//!
//! This signs a real document with a real certificate and reads the stamp back
//! out of the resulting file, so it fails if the rectangle goes blank again.

use nitum_pdf::{
    application::{PdfSigning, SignRequest},
    domain::{PadesLevel, SignaturePlacement},
    infrastructure::NativePadesSigning,
};

#[test]
#[ignore = "requires NITUM_TEST_P12 and NITUM_TEST_P12_PASSWORD"]
fn the_visible_signature_states_who_signed_and_when() {
    let identity = std::env::var_os("NITUM_TEST_P12").expect("NITUM_TEST_P12");
    let password = std::env::var("NITUM_TEST_P12_PASSWORD").expect("password");
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.pdf");
    let target =
        std::env::temp_dir().join(format!("nitum-appearance-test-{}.pdf", std::process::id()));
    let _ = std::fs::remove_file(&target);

    let signing = NativePadesSigning::new().expect("native PAdES adapter");
    signing
        .sign(SignRequest {
            source: &source,
            target: &target,
            identity: std::path::Path::new(&identity),
            secret: &password,
            level: PadesLevel::BaselineB,
            reason: Some("Aprobación"),
            location: Some("Quito"),
            placement: Some(SignaturePlacement {
                page_index: 0,
                left: 60.0,
                bottom: 60.0,
                width: 260.0,
                height: 90.0,
            }),
            appearance: None,
            certification: None,
        })
        .expect("signing a document with a visible signature");

    let signed = std::fs::read(&target).expect("the signed document");
    let contains = |needle: &[u8]| signed.windows(needle.len()).any(|window| window == needle);

    // The appearance is drawn with PDF text-showing operators, so the stamped
    // strings survive in the raw bytes without a full parse. They are compared
    // as bytes because the literal strings are WinAnsi-encoded, not UTF-8:
    // "Ubicación" is stored as `Ubicaci` `F3` `n`.
    for expected in [
        &b"Firmado por"[..],
        &b"Emitido por"[..],
        &b"Fecha"[..],
        &b"Motivo"[..],
    ] {
        assert!(
            contains(expected),
            "the stamp must state {:?}: a visible signature that draws an empty \
             rectangle tells the reader nothing",
            String::from_utf8_lossy(expected)
        );
    }

    // Accented text has to reach the page as single WinAnsi bytes. Written as
    // UTF-8 it drew "Ubicación" as "UbicaciÃ³n", which is every Spanish label in
    // the stamp.
    assert!(
        contains(b"Ubicaci\xF3n"),
        "accented characters must be WinAnsi bytes, not UTF-8"
    );
    assert!(
        !contains("Ubicación".as_bytes()),
        "the UTF-8 form must not appear: a WinAnsi viewer draws it as mojibake"
    );

    // Same convention the visual harness uses: with NITUM_VISUAL_OUTPUT set, the
    // signed document is kept so the stamp can be looked at rather than only
    // asserted on.
    match std::env::var_os("NITUM_VISUAL_OUTPUT") {
        Some(directory) => {
            let kept = std::path::Path::new(&directory).join("firma-visible.pdf");
            std::fs::create_dir_all(&directory).expect("output directory");
            std::fs::copy(&target, &kept).expect("keeping the signed document");
            println!("firmado guardado en {}", kept.display());
        }
        None => {
            let _ = std::fs::remove_file(&target);
        }
    }
}
