# Nitum PDF

![Nitum](data/org.pwview.PdfViewer.svg)

**Nitum PDF** es un visor de PDF para Linux con firma digital comprensible.
Permite abrir, buscar, firmar y verificar documentos sin confundir la integridad
del archivo con la identidad del firmante.

Es el primer producto de la familia Nitum: herramientas de productividad claras,
confiables y sin complejidad innecesaria.

## Funciones

- Desplazamiento continuo, zoom, búsqueda y copia de texto.
- Firma visible o invisible con certificados `.p12`/`.pfx` y tokens PKCS#11.
- Apariencia de firma visual guardada y reutilizable, siempre respaldada por el certificado.
- Flujo guiado para firmar, certificar o comprobar firmas recibidas.
- Sello de tiempo RFC 3161 y validación a largo plazo activados por defecto.
- Firmas incrementales; nunca sobrescribe el documento original.
- Comprobación automática diaria de nuevas versiones publicadas en GitHub.
- Descarga del `.deb`, verificación SHA-256 y actualización mediante `apt`/polkit.

### Identidades digitales compatibles

- **Archivos de Acrobat `.p12` y `.pfx`:** Nitum pide la contraseña, comprueba
  que exista una clave privada y muestra el nombre real del titular antes de firmar.
- **Tarjetas, DNI y tokens PKCS#11:** aparecen automáticamente cuando Linux y
  `p11-kit` reconocen el dispositivo; la firma solicita su PIN.
- **Identidad local:** puede crearse dentro de Nitum para uso personal o interno
  y se guarda como PKCS#12 protegido por contraseña, compatible con Acrobat.
- **`.cer`, `.crt` y certificados `.pem`:** normalmente solo contienen la parte
  pública. Sirven para comprobar firmas, pero no pueden firmar sin la clave privada.
- **Windows Certificate Store:** es una integración exclusiva de Windows. En
  Linux, el equivalente interoperable es exportar la identidad como `.pfx/.p12`.
- **Identidades remotas:** dependen del servidor y proveedor concreto; no son un
  formato de archivo que pueda importarse de manera genérica.

## Instalar

Descarga el `.deb` más reciente desde
[GitHub Releases](https://github.com/dev-jackson/nitum-pdf/releases/latest) y ábrelo
con el instalador del sistema, o ejecuta:

```bash
sudo apt install ./nitum-pdf_*_amd64.deb
```

Nitum PDF busca versiones nuevas al iniciarse, como máximo una vez al día. También
puedes usar **Buscar actualizaciones** desde el menú. Nunca se instala en silencio:
primero pide confirmación y después el sistema solicita autorización administrativa.

## Ejecutar desde el código

```bash
sudo apt install python3-venv python3-gi gir1.2-gtk-4.0 gir1.2-adw-1 opensc pcscd
python3 -m venv --system-site-packages .venv
.venv/bin/pip install -e ".[dev,pkcs11]"
.venv/bin/nitum-pdf documento.pdf
```

## Construir y probar

```bash
python -m pytest
./packaging/build-deb.sh
sudo apt install ./dist/nitum-pdf_*.deb
```

## Atajos

| Tecla | Acción |
|---|---|
| Ctrl+F o Ctrl+K | Buscar dentro del PDF |
| Ctrl+Mayús+S | Abrir el centro de firmas |
| Ctrl+O | Abrir otro PDF |
| Ctrl+C | Copiar el texto de la página |
| Ctrl+0 | Ajustar al ancho |
| Esc | Cancelar la colocación de firma |

Las decisiones de interfaz y sus referentes están documentados en
[DESIGN.md](DESIGN.md).

## GitFlow y releases

El proyecto usa `main` para publicaciones y `develop` para integración. Consulta
[CONTRIBUTING.md](CONTRIBUTING.md) para crear `feature/*`, `release/*` y `hotfix/*`.
Cada etiqueta `vX.Y.Z` creada en `main` construye automáticamente el `.deb`, su
checksum SHA-256 y un GitHub Release que la aplicación puede instalar.

## Privacidad y seguridad

Los documentos permanecen en el equipo. La actualización solo consulta la API
pública de GitHub. El paquete debe coincidir con el SHA-256 del release antes de
que Nitum PDF permita instalarlo mediante `apt`.

Las identidades existentes de `pw-view-pdf` se conservan durante la migración.

## Licencia

MIT. Consulta [LICENSE](LICENSE).
