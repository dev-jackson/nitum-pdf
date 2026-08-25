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
git switch -c release/0.3.0
# actualizar la versión en pyproject.toml y src/pwviewpdf/__init__.py
python -m pytest
git switch main
git merge --no-ff release/0.3.0
git tag -s v0.3.0 -m "Nitum PDF 0.3.0"
git switch develop
git merge --no-ff release/0.3.0
git push origin main develop v0.3.0
```

La etiqueta debe estar firmada. GitHub Actions genera el paquete y el checksum;
no se deben adjuntar binarios construidos manualmente a un release oficial.

### Hotfix

Un `hotfix/*` se fusiona tanto en `main` como en `develop` y recibe una nueva
etiqueta de versión. Nunca se reutiliza ni se mueve una etiqueta publicada.
