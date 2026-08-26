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
adjuntar binarios construidos manualmente a un release oficial. El workflow
rechaza una publicación sin Developer ID/notarización en macOS o Authenticode
en Windows.

### Credenciales de distribución

Configura estos secretos del repositorio con `gh secret set <NOMBRE>` antes de
crear la etiqueta. Nunca guardes certificados ni contraseñas en el árbol Git.
Para cargarlos de forma guiada, sin imprimir valores ni crear copias temporales,
ejecuta `packaging/native/configure-release-secrets.sh`. El script valida ambos
PKCS#12 antes de solicitar confirmación y modificar GitHub.

| Secreto | Contenido |
|---|---|
| `NITUM_APPLE_CERTIFICATE_BASE64` | PKCS#12 que contiene Developer ID Application y Developer ID Installer, codificado en Base64 |
| `NITUM_APPLE_CERTIFICATE_PASSWORD` | Contraseña del PKCS#12 anterior |
| `NITUM_APPLE_SIGN_IDENTITY` | Nombre completo de la identidad `Developer ID Application` |
| `NITUM_APPLE_INSTALLER_IDENTITY` | Nombre completo de la identidad `Developer ID Installer` |
| `NITUM_APPLE_ID` | Apple ID usado por `notarytool` |
| `NITUM_APPLE_TEAM_ID` | Team ID de Apple Developer |
| `NITUM_APPLE_APP_PASSWORD` | Contraseña específica de aplicación para notarización |
| `NITUM_WINDOWS_CERTIFICATE_BASE64` | Certificado de firma de código PFX, codificado en Base64 |
| `NITUM_WINDOWS_CERTIFICATE_PASSWORD` | Contraseña del PFX anterior |

Antes de etiquetar, `gh secret list` debe mostrar los nueve nombres. Los valores
no se imprimen. En macOS puedes confirmar los nombres locales con
`security find-identity -v -p codesigning`; en Windows, el instalador y el
ejecutable se verifican con `signtool verify /pa /v` dentro del workflow.

El repositorio y los artefactos son íntegramente Rust/Slint: no se acepta añadir
Python, entornos virtuales ni pasos `pip` a la aplicación, las pruebas o CI.

### Hotfix

Un `hotfix/*` se fusiona tanto en `main` como en `develop` y recibe una nueva
etiqueta de versión. Nunca se reutiliza ni se mueve una etiqueta publicada.
