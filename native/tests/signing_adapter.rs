use nitum_pdf::{
    application::{PdfSigning, SignRequest},
    domain::{CertificationPermission, PadesLevel, SignatureKind, SignaturePlacement},
    infrastructure::NativePadesSigning,
};

#[test]
#[ignore = "requires NITUM_TEST_P12 and NITUM_TEST_P12_PASSWORD"]
fn signs_incrementally_and_verifies_the_result() {
    let identity = std::env::var_os("NITUM_TEST_P12").expect("NITUM_TEST_P12");
    let password = std::env::var("NITUM_TEST_P12_PASSWORD").expect("password");
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.pdf");
    let target = std::env::temp_dir().join(format!("nitum-pades-test-{}.pdf", std::process::id()));
    let second_target = std::env::temp_dir().join(format!(
        "nitum-pades-test-{}-second.pdf",
        std::process::id()
    ));
    let appearance = std::env::var_os("NITUM_TEST_APPEARANCE").map(std::path::PathBuf::from);
    let _ = std::fs::remove_file(&target);
    let _ = std::fs::remove_file(&second_target);

    let signing = NativePadesSigning::new().expect("native PAdES adapter");
    let reports = signing
        .sign(SignRequest {
            source: &source,
            target: &target,
            identity: std::path::Path::new(&identity),
            secret: &password,
            level: std::env::var("NITUM_TEST_PADES_LEVEL")
                .ok()
                .as_deref()
                .map_or(PadesLevel::BaselineB, |level| match level {
                    "B-T" => PadesLevel::BaselineT,
                    "B-LT" => PadesLevel::BaselineLt,
                    "B-LTA" => PadesLevel::BaselineLta,
                    _ => PadesLevel::BaselineB,
                }),
            reason: Some("Aprobación"),
            location: Some("Quito"),
            placement: Some(SignaturePlacement {
                page_index: 0,
                left: 36.0,
                bottom: 36.0,
                width: 220.0,
                height: 72.0,
            }),
            appearance: appearance.as_deref(),
            certification: std::env::var_os("NITUM_TEST_CERTIFICATION")
                .map(|_| CertificationPermission::NoChanges),
        })
        .expect("PAdES signature");

    assert_eq!(
        &std::fs::read(&source).unwrap(),
        &std::fs::read(&target).unwrap()[..std::fs::metadata(&source).unwrap().len() as usize]
    );
    let approval = reports
        .iter()
        .find(|report| report.kind == SignatureKind::Approval)
        .expect("approval signature report");
    assert!(approval.cryptographically_intact);
    let extracted =
        underskrift::verify::extractor::extract_signatures(&std::fs::read(&target).unwrap())
            .expect("extract signed PDF fields");
    let approval_field = extracted
        .iter()
        .find(|item| item.signature_type == underskrift::verify::SignatureType::Pades)
        .expect("approval field");
    assert!(
        approval_field
            .signing_time
            .as_deref()
            .is_some_and(|value| value.starts_with("D:")),
        "PAdES Baseline requires the PDF /M signing time"
    );
    if std::env::var("NITUM_TEST_PADES_LEVEL").as_deref() == Ok("B-T") {
        assert_eq!(approval.pades_level, Some(PadesLevel::BaselineT));
        assert!(reports.iter().any(|report| {
            report.kind == SignatureKind::DocumentTimestamp && report.cryptographically_intact
        }));
    }
    if let Ok(level @ ("B-LT" | "B-LTA")) = std::env::var("NITUM_TEST_PADES_LEVEL").as_deref() {
        let expected = if level == "B-LTA" {
            PadesLevel::BaselineLta
        } else {
            PadesLevel::BaselineLt
        };
        assert_eq!(approval.pades_level, Some(expected));
        let inspection = underskrift::inspect::inspect_signatures(&std::fs::read(&target).unwrap())
            .expect("inspect DSS");
        let dss = inspection.dss.expect("document security store");
        assert!(dss.num_certs > 0);
        assert!(dss.vri.iter().any(|entry| entry.num_certs > 0));
        let timestamp_count = reports
            .iter()
            .filter(|report| report.kind == SignatureKind::DocumentTimestamp)
            .count();
        assert_eq!(timestamp_count, if level == "B-LTA" { 2 } else { 1 });
        let timestamp_names = extracted
            .iter()
            .filter(|item| item.signature_type == underskrift::verify::SignatureType::DocTimestamp)
            .map(|item| item.field_name.as_str())
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(timestamp_names.len(), timestamp_count);
        assert!(
            reports
                .iter()
                .filter(|report| report.kind == SignatureKind::DocumentTimestamp)
                .all(|report| report.cryptographically_intact)
        );
    }
    if std::env::var_os("NITUM_TEST_CERTIFICATION").is_some() {
        let inspection = underskrift::inspect::inspect_signatures(&std::fs::read(&target).unwrap())
            .expect("inspect certification");
        assert_eq!(inspection.num_signatures, 1);
        assert_eq!(
            inspection.catalog_doc_mdp_obj_num,
            inspection.signatures[0].obj_num
        );
        assert_eq!(inspection.signatures[0].doc_mdp_permissions, Some(1));
    }
    assert!(signing.verify(&target).unwrap().iter().all(|report| {
        report.kind != SignatureKind::Approval || report.cryptographically_intact
    }));
    if std::env::var_os("NITUM_TEST_SUCCESSIVE").is_some() {
        let successive = signing
            .sign(SignRequest {
                source: &target,
                target: &second_target,
                identity: std::path::Path::new(&identity),
                secret: &password,
                level: PadesLevel::BaselineB,
                reason: Some("Segunda aprobación"),
                location: None,
                placement: None,
                appearance: None,
                certification: None,
            })
            .expect("successive PAdES signature");
        assert_eq!(successive.len(), 2);
        assert!(
            successive
                .iter()
                .all(|report| report.cryptographically_intact)
        );
        assert_eq!(signing.verify(&second_target).unwrap().len(), 2);
        std::fs::remove_file(&second_target).unwrap();
    }
    if std::env::var_os("NITUM_KEEP_TEST_OUTPUT").is_some() {
        eprintln!("NITUM_TEST_OUTPUT={}", target.display());
    } else {
        std::fs::remove_file(target).unwrap();
    }
}
