use nitum_pdf::{application::IdentityStore, infrastructure::NativeIdentityStore};

#[test]
fn imports_reuses_and_separates_identities_safely() {
    let temporary = tempfile::tempdir().unwrap();
    let incoming = temporary.path().join("incoming");
    let stored = temporary.path().join("stored");
    std::fs::create_dir(&incoming).unwrap();
    let first = incoming.join("Mi Identidad.p12");
    let second = incoming.join("Mi Identidad.pfx");
    std::fs::write(&first, b"first identity").unwrap();
    std::fs::write(&second, b"second identity").unwrap();

    let store = NativeIdentityStore::at(stored);
    let imported = store.import(&first).unwrap();
    let reused = store.import(&first).unwrap();
    let other = store.import(&second).unwrap();

    assert_eq!(imported.path, reused.path);
    assert_ne!(imported.path, other.path);
    assert_eq!(store.list().unwrap().len(), 2);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(imported.path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}

#[test]
fn rejects_files_outside_pkcs12_formats() {
    let temporary = tempfile::tempdir().unwrap();
    let source = temporary.path().join("not-an-identity.pem");
    std::fs::write(&source, b"not a PKCS12 identity").unwrap();
    let store = NativeIdentityStore::at(temporary.path().join("stored"));
    assert!(store.import(&source).is_err());
}
