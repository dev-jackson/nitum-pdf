# Desarrollo de Nitum PDF

## GitFlow

- `main`: código publicado y etiquetado.
- `develop`: integración de la próxima versión.
- `feature/<nombre>`: nace de `develop` y vuelve a `develop` mediante pull request.
- `release/<versión>`: nace de `develop`; solo admite estabilización y cambio de versión.
- `hotfix/<versión>`: nace de `main` para corregir una versión publicada.

### Publicar una versión

```bash
git switch develop
git switch -c release/<versión>
# actualizar la versión en native/Cargo.toml y regenerar native/Cargo.lock
cd native
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets --locked
cd ..
git switch main
git merge --no-ff release/0.3.0
git tag -s v<versión> -m "Nitum PDF <versión>"
git switch develop
git merge --no-ff release/0.3.0
git push origin main develop v<versión>
```

La etiqueta debe estar firmada. GitHub Actions genera paquetes, checksums y
attestations Sigstore verificables con `gh attestation verify`; no se deben
adjuntar binarios construidos manualmente a un release oficial. Cuando estén
configuradas las credenciales de plataforma, macOS se firma y notariza y Windows
aplica Authenticode antes de publicar.

El repositorio y los artefactos son íntegramente Rust/Slint: no se acepta añadir
Python, entornos virtuales ni pasos `pip` a la aplicación, las pruebas o CI.

### Hotfix

Un `hotfix/*` se fusiona tanto en `main` como en `develop` y recibe una nueva
etiqueta de versión. Nunca se reutiliza ni se mueve una etiqueta publicada.
