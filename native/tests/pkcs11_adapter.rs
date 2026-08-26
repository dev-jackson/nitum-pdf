use nitum_pdf::{
    application::{HardwareSignRequest, HardwareTokenProvider, PdfSigning},
    domain::{PadesLevel, SignaturePlacement},
    infrastructure::{NativeHardwareTokenProvider, NativePadesSigning},
};

#[test]
#[ignore = "requires a provisioned PKCS#11 token and NITUM_TEST_PKCS11_* variables"]
fn discovers_and_signs_with_a_pkcs11_token_without_exporting_its_key() {
    let module = std::path::PathBuf::from(
        std::env::var_os("NITUM_TEST_PKCS11_MODULE").expect("NITUM_TEST_PKCS11_MODULE"),
    );
    let pin = std::env::var("NITUM_TEST_PKCS11_PIN").expect("NITUM_TEST_PKCS11_PIN");
    let provider = NativeHardwareTokenProvider;
    let token = provider
        .tokens(&module)
        .expect("PKCS#11 discovery")
        .into_iter()
        .next()
        .expect("token");

    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.pdf");
    let target = std::env::temp_dir().join(format!("nitum-pkcs11-{}.pdf", std::process::id()));
    let _ = std::fs::remove_file(&target);
    let signing = NativePadesSigning::new().unwrap();
    let reports = signing
        .sign_hardware(HardwareSignRequest {
            source: &source,
            target: &target,
            token: &token,
            pin: &pin,
            level: PadesLevel::BaselineB,
            reason: Some("Prueba PKCS#11"),
            location: None,
            placement: Some(SignaturePlacement {
                page_index: 0,
                left: 36.0,
                bottom: 36.0,
                width: 220.0,
                height: 72.0,
            }),
            appearance: None,
            certification: None,
        })
        .expect("hardware PAdES signature");

    assert!(reports.last().unwrap().cryptographically_intact);
    assert!(
        signing
            .verify(&target)
            .unwrap()
            .last()
            .unwrap()
            .cryptographically_intact
    );
    std::fs::remove_file(target).unwrap();
}
