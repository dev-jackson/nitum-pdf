use nitum_pdf::{application::AppearanceStore, infrastructure::NativeAppearanceStore};

#[test]
fn validates_saves_and_reuses_a_visual_signature() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("mi firma.png");
    std::fs::write(&source, include_bytes!("../../data/nitum-family-mark.png")).unwrap();
    let store = NativeAppearanceStore::at(directory.path().join("store"));

    let first = store.import(&source).unwrap();
    let second = store.import(&source).unwrap();
    assert_eq!(first, second);
    assert_eq!(store.list().unwrap(), vec![first]);
}

#[test]
fn rejects_a_fake_image_even_when_its_extension_looks_valid() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("firma.png");
    std::fs::write(&source, b"not an image").unwrap();
    let store = NativeAppearanceStore::at(directory.path().join("store"));
    assert!(store.import(&source).is_err());
}

#[test]
fn normalizes_common_adobe_appearance_formats_to_png() {
    let directory = tempfile::tempdir().unwrap();
    let formats = [
        ("jpg", image::ImageFormat::Jpeg),
        ("gif", image::ImageFormat::Gif),
        ("bmp", image::ImageFormat::Bmp),
        ("tiff", image::ImageFormat::Tiff),
        ("webp", image::ImageFormat::WebP),
    ];
    for (extension, format) in formats {
        let source = directory.path().join(format!("firma.{extension}"));
        let mut encoded = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(8, 4)
            .write_to(&mut encoded, format)
            .unwrap();
        std::fs::write(&source, encoded.into_inner()).unwrap();
        let store = NativeAppearanceStore::at(directory.path().join(format!("store-{extension}")));
        let imported = store.import(&source).unwrap();
        assert_eq!(
            imported.path.extension().and_then(|value| value.to_str()),
            Some("png")
        );
        assert!(image::open(imported.path).is_ok());
    }
}
