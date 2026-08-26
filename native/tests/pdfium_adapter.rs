use nitum_pdf::{
    application::PdfEngine,
    domain::{PdfPasswordRequired, PdfRect},
    infrastructure::NativePdfEngine,
};

#[test]
#[ignore = "requires the platform PDFium binary next to the test executable"]
fn opens_and_renders_a_real_pdf() {
    let engine = NativePdfEngine::new().expect("PDFium should be bundled");
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.pdf");
    let document = engine.open(&fixture, None).expect("fixture should open");
    assert!(document.page_count() > 0);

    let size = document.page_size(0).expect("page geometry");
    assert!(size.width_points > 0.0);
    assert!(size.height_points > 0.0);

    let bitmap = document.render_page(0, 0.5).expect("page bitmap");
    assert!(bitmap.width > 0);
    assert!(bitmap.height > 0);
    assert_eq!(
        bitmap.rgba.len(),
        bitmap.width as usize * bitmap.height as usize * 4
    );
    assert!(document.search("", 100).unwrap().is_empty());
    let text = document.page_text(0).expect("page text extraction");
    let query = text
        .split_whitespace()
        .find(|word| word.chars().count() >= 4)
        .expect("fixture should contain searchable text");
    let hits = document.search(query, 100).expect("text search");
    assert!(!hits.is_empty());
    assert!(hits.iter().all(|hit| hit.bounds.right > hit.bounds.left));
    assert!(!text.trim().is_empty());
    let selected = document
        .text_in_rect(
            0,
            PdfRect {
                left: 0.0,
                bottom: 0.0,
                right: size.width_points,
                top: size.height_points,
            },
        )
        .expect("rectangular text selection");
    assert!(selected.contains(query));
    assert!(
        document
            .text_in_rect(
                0,
                PdfRect {
                    left: size.width_points + 10.0,
                    bottom: 0.0,
                    right: size.width_points + 20.0,
                    top: 10.0,
                },
            )
            .unwrap()
            .is_empty()
    );
}

#[test]
#[ignore = "requires NITUM_TEST_ENCRYPTED_PDF and the platform PDFium binary"]
fn encrypted_pdf_requires_and_accepts_its_password() {
    let path = std::env::var("NITUM_TEST_ENCRYPTED_PDF").expect("encrypted PDF path");
    let password = std::env::var("NITUM_TEST_PDF_PASSWORD").expect("PDF password");
    let engine = NativePdfEngine::new().expect("PDFium must load");

    let missing = engine
        .open(path.as_ref(), None)
        .err()
        .expect("must be locked");
    assert!(missing.downcast_ref::<PdfPasswordRequired>().is_some());

    let wrong = engine
        .open(path.as_ref(), Some("incorrecta"))
        .err()
        .expect("wrong password must fail");
    assert!(wrong.downcast_ref::<PdfPasswordRequired>().is_some());

    let document = engine
        .open(path.as_ref(), Some(&password))
        .expect("correct password should open the PDF");
    assert!(document.page_count() > 0);
    let bitmap = document.render_page(0, 1.0).expect("render first page");
    assert_eq!(
        bitmap.rgba.len(),
        (bitmap.width * bitmap.height * 4) as usize
    );
}

#[test]
#[ignore = "requires the platform PDFium binary next to the test executable"]
fn damaged_pdf_is_rejected_cleanly() {
    let directory = tempfile::tempdir().expect("temporary fixture directory");
    let damaged = directory.path().join("damaged.pdf");
    std::fs::write(&damaged, b"%PDF-1.7\ncontenido truncado")
        .expect("write intentionally damaged PDF");
    let engine = NativePdfEngine::new().expect("PDFium must load");
    let error = engine
        .open(&damaged, None)
        .err()
        .expect("damaged input must not open");
    assert!(!error.to_string().trim().is_empty());
}
