# Visor de PDF simple + firma digital fácil (Linux primero)

Investigación técnica y decisión de arquitectura. Fecha: 2026-08-24.

---

## 1. El problema real

En Linux hay visores (Evince/Papers, Okular, Atril, Xpdf, Zathura) y hay firmadores
(JSignPdf, AutoFirma, pyhanko CLI), pero **casi nada que haga las dos cosas bien y sea
obvio de usar**. Estado actual:

| App | Ve PDF | Firma cripto | Problema |
|---|---|---|---|
| GNOME Papers | sí, bien | sí desde v47 (poppler ≥ 25.01, PKCS#11) | flujo confuso, sin PKCS#12 cómodo, sin TSA/LTV claro, sin verificación explicada |
| Okular | sí, bien | sí (poppler + NSS) | certificados deben estar en la BD NSS (`~/.pki/nssdb`); tokens PKCS#11 a menudo no aparecen |
| JSignPdf | no ve | sí, muy completo (PKCS#11/12, TSA, LTV) | Java, UI de 2008, no es visor |
| AutoFirma | previsualiza | sí (España, GPL) | Java, pesado, orientado a trámites del Estado español |
| Adobe Reader | — | — | ya no existe versión Linux moderna |

Consecuencia: quien viene de Windows/Acrobat encuentra que "abrir y firmar" pasa de ser
2 clics a ser un fin de semana de configurar NSS, OpenSC y p11-kit. **Ese es el hueco que
llenamos.**

Referencias: [Papers digital signature](https://discourse.gnome.org/t/digital-signature-in-papers/33495),
[MR de firma en Papers](https://gitlab.gnome.org/GNOME/Incubator/papers/-/merge_requests/296),
[issue "Signing Capabilities are confusing"](https://gitlab.gnome.org/GNOME/papers/-/issues/330),
[firmar con token + Okular](https://rajeeshknambiar.wordpress.com/2022/06/27/digitally-signing-pdf-documents-in-linux-with-hardware-token-okular/),
[JSignPdf](https://github.com/intoolswetrust/jsignpdf).

---

## 2. Cómo funciona la firma digital en PDF (lo que hay que implementar)

No es "pegar una imagen". Una firma PDF real es:

1. Se añade un **campo de firma** (`/FT /Sig`) en el AcroForm, con un widget de anotación
   en una página (el rectángulo visible; si la caja es 0x0, la firma es invisible).
2. El valor del campo es un diccionario `/Sig` con:
   - `/ByteRange [0 a b c]` — los dos trozos del archivo que se firman: **todo el fichero
     excepto el hueco de `/Contents`**.
   - `/Contents <....>` — string hexadecimal (relleno de ceros) donde se mete el
     **CMS/PKCS#7 SignedData en modo detached** (DER).
   - `/SubFilter`: `/adbe.pkcs7.detached` (clásico) o `/ETSI.CAdES.detached` (PAdES).
   - opcionales: `/Reason`, `/Location`, `/M` (fecha), `/Name`.
3. Todo esto se escribe como **incremental update**: se anexa al final del fichero
   (objetos nuevos + xref nuevo). Nunca se reescribe el PDF original — si lo reescribes,
   invalidas las firmas anteriores.
4. Firmas múltiples = varios incremental updates encadenados.
5. `/DocMDP` = firma de certificación (la primera, "autor"), con `P=1|2|3` para permitir
   nada / rellenar formularios / anotar. `/FieldMDP` bloquea campos concretos.

### Niveles PAdES (ETSI EN 319 142)

| Nivel | Qué añade | Para qué |
|---|---|---|
| B-B | firma CMS básica | mínimo |
| B-T | + sello de tiempo RFC 3161 de una TSA | prueba "cuándo" sin depender del reloj del PC |
| B-LT | + DSS con certs, OCSP y CRL embebidos | validable dentro de 10 años aunque la CA desaparezca (LTV en Acrobat) |
| B-LTA | + document timestamp de archivo | renovable indefinidamente |

**Decisión de producto: por defecto firmamos B-LT (TSA + LTV) sin preguntar.** Acrobat
muestra "LTV enabled" y el usuario no tiene que saber qué significa. Fallback silencioso
a B-T o B-B si no hay red, avisando en la UI.

### Por qué Acrobat dice "validez desconocida"

La firma puede ser criptográficamente perfecta y aun así Acrobat muestra el triángulo
amarillo: significa **"no puedo verificar la identidad"**, no "el documento está alterado".
Solo confía por defecto en certificados encadenados a la **AATL** (Adobe Approved Trust
List) o a la EUTL. Un certificado autofirmado siempre saldrá amarillo hasta que el
receptor lo añada a identidades de confianza.
([explicación](https://helpx.adobe.com/acrobat/using/trusted-identities.html),
[caso típico](https://www.verypdf.com/wordpress/201503/adobe-reader-or-acrobat-displays-a-at-least-one-signature-has-problems-message-when-signed-pdf-is-opened-signature-validity-is-unknown-signers-identity-is-unknown-41508.html))

**Mejora sobre Adobe:** su mensaje ("Al menos una firma tiene problemas") asusta y no
explica nada. Nosotros separamos siempre las dos preguntas en la UI:

```
✅ Documento íntegro — no se modificó después de firmar
⚠️  Identidad no verificada — el certificado no está en ninguna lista de confianza
    [Ver certificado]  [Confiar en este emisor]
```

---

## 3. Elección de stack

### Motor de render

| Opción | Licencia | Notas |
|---|---|---|
| **pdfium (pypdfium2)** | BSD-3 / Apache-2.0 | rápido, ruedas precompiladas, sin dependencias del sistema. **Elegido** |
| poppler-glib | GPL-2/3 | ya en todos los distros, pero API C y GPL contagia |
| MuPDF (PyMuPDF) | **AGPL-3.0** | rapidísimo pero AGPL: obliga a abrir todo o pagar a Artifex |
| pdf.js | Apache-2.0 | obliga a arrastrar Electron/Tauri |

pypdfium2 5.13.0, licencia permisiva, render competitivo con MuPDF.
([comparativa](https://johannesfilter.com/python-and-pdf-a-review-of-existing-tools/))

### Motor de firma

**pyHanko 0.36.2 (MIT, julio 2026)** — es, con diferencia, la mejor implementación libre:

- PAdES B-B / B-T / B-LT / B-LTA completos
- firmas visibles e invisibles, sellos con texto/imagen/QR
- **PKCS#11 nativo** (tokens, DNIe, eToken, HSM) y PKCS#12
- incremental updates correctos, firmas múltiples
- validación de firmas (`pyhanko.sign.validation`) además de creación

Alternativas descartadas: iText/DSS (Java, licencia AGPL o comercial), `@signpdf` (Node,
sin PAdES-LTA ni PKCS#11 serio), Rust (no hay crate de PAdES maduro).

([pyHanko](https://github.com/MatthiasValvekens/pyHanko), [docs](https://docs.pyhanko.eu/en/latest/lib-guide/index.html))

### UI

**GTK4 + libadwaita vía PyGObject.** Razones: pyHanko es Python (evita IPC entre dos
runtimes), integración nativa GNOME, empaquetado Flatpak limpio, y GTK4 `FileDialog` +
portales funcionan bien bajo Wayland/Flatpak.

> Si más adelante el arranque de Python molesta, el núcleo (`firma.py`) sobrevive como
> servicio/CLI y la UI puede reescribirse en Rust-gtk4 sin tocar la criptografía.

### Multiplataforma después

pypdfium2 y pyHanko ya son multiplataforma. El único trabajo real en Windows/macOS será
UI (GTK4 funciona pero se ve extraño) y acceso a los almacenes nativos (CryptoAPI /
Keychain). Nada de la arquitectura lo bloquea.

---

## 4. Identidades: el punto donde todos fallan

El 80% del dolor no es firmar, es **encontrar el certificado**. Diseño:

**Al abrir el diálogo de firma, escaneamos y mostramos una sola lista unificada:**

1. **Tokens/smartcards** vía PKCS#11 — descubrimiento con `p11-kit list-modules`
   (los distros registran OpenSC en `/usr/share/p11-kit/modules/opensc.module`), más
   rutas conocidas de fabricante (`/usr/lib/opensc-pkcs11.so`, SafeNet, eToken).
   ([p11-kit](https://man.archlinux.org/man/p11-kit.8.en),
   [RHEL PKCS#11](https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/9/html/security_hardening/configuring-applications-to-use-cryptographic-hardware-through-pkcs-11_security-hardening))
2. **Ficheros `.p12` / `.pfx`** importados por el usuario (los que exportó de Windows).
3. **NSS `~/.pki/nssdb`** (lo que ya usa Okular/Firefox) — vía su módulo PKCS#11
   `libsoftokn3`, para no obligar a reimportar.

**Importar desde Windows**: el usuario exporta en `certmgr.msc` → *Exportar con clave
privada* → `.pfx`. Nuestro asistente lo copia a `~/.local/share/<app>/identidades/`,
pide la contraseña una vez y (opcional) la guarda en el llavero vía Secret Service.
Esa es literalmente la transición Windows→Linux que hoy no existe.

---

## 5. UX: Adobe como inspiración, pero sin su fricción

| Acrobat | Nosotros |
|---|---|
| Herramientas → Certificados → Firmar digitalmente → leer aviso → arrastrar caja → elegir ID digital → diálogo de apariencia → Firmar → Guardar como | Botón **Firmar** → arrastrar caja (o "firma invisible") → elegir identidad → **Firmar** |
| "ID digital" (jerga) | "Tu identidad" con nombre y validez legibles |
| Sello de tiempo hay que configurarlo en preferencias | activado por defecto, TSA preconfigurada |
| LTV es una casilla escondida | por defecto, sin preguntar |
| "Al menos una firma tiene problemas" | integridad e identidad como dos líneas separadas y explicadas |
| Fill & Sign (dibujo, sin valor legal) mezclado con firma cripto | dos acciones claramente distintas: **Firmar** (criptográfica) y **Anotar** (dibujo) |

Reglas de la UI de firma:
- La caja se arrastra sobre la página con vista previa en vivo del sello.
- Apariencia por defecto: nombre del titular (CN del certificado), fecha con zona horaria,
  motivo opcional. Plantilla recordada entre sesiones.
- **Nunca sobrescribir el original sin decirlo**: por defecto `documento-firmado.pdf`.
- Tras firmar, **verificamos inmediatamente** el resultado con `validate_pdf_signature`
  y mostramos el panel de estado. Si algo salió mal, el usuario lo sabe al segundo.
- El PIN del token se pide en el momento y no se guarda ni se registra.

---

## 6. Alcance del MVP

**Visor (lo básico, hecho bien):**
abrir, scroll continuo, zoom (ajustar ancho/página, ±), ir a página, rotar,
buscar texto, seleccionar/copiar texto, miniaturas + índice, imprimir, recientes.

**Firma:**
firmar con `.p12` o token PKCS#11, firma visible o invisible, TSA + LTV por defecto,
firmas múltiples, panel de verificación, importar identidad desde `.pfx` de Windows,
exportar certificado público.

**Fuera del MVP:** edición de PDF, OCR, formularios avanzados, cifrado, firma remota
(cloud/eIDAS QSCD), Windows/macOS.

## 7. Lo que aprendimos construyendo el prototipo

Cosas que no aparecían en ninguna documentación y que sólo salieron al ejecutarlo.
Cada una tiene su prueba de regresión.

1. **pdfium no dibuja las firmas visibles** a menos que se llame a `init_forms()`
   en el documento. El sello está en el fichero, la firma valida, y en pantalla no
   se ve nada. Es el fallo más traicionero de todos: parece que no firmaste.
   → `test_visible_signature_is_actually_drawn`
2. **pdfium devuelve 3 canales (BGR) en páginas opacas**, no 4. `Gdk.MemoryTexture`
   rechaza el buffer con un `assertion 'stride ... requires 1280 bytes' failed` y la
   página sale en blanco. Hay que forzar `FPDFBitmap_BGRA`.
   → `test_render_is_always_four_channels`
3. **Un certificado autofirmado hace que pyhanko-certvalidator escupa un traceback
   completo a nivel ERROR.** No es un fallo: es la respuesta "identidad no
   verificada". Hay que bajar el nivel de log o el usuario ve un muro rojo.
4. **`verify()` reventaba con un PDF truncado** en vez de informar. Ahora hay un
   error propio (`DocumentUnreadable`) porque un archivo dañado es un caso normal.
   → `test_damaged_file_is_reported_instead_of_crashing`
5. **Sólo Courier se puede usar sin incrustar una fuente.** El motor simple de
   pyHanko asume anchos uniformes; con Helvetica los glifos se solaparían. Para
   tipografía mejor hay que embeber un TTF (extra `opentype`).
6. **El sellado de tiempo real funciona**: firma contra freetsa.org verificada como
   PAdES B-T. → `tests/test_network.py`
7. **La degradación sin red hay que probarla a propósito**: apuntar la TSA al
   puerto 9 (discard) reproduce el caso offline de forma determinista.
   → `test_falls_back_to_a_plain_signature_when_the_tsa_is_unreachable`

Y una decisión de producto que salió de mirar la primera captura de pantalla: el
diálogo de verificación necesitaba un **botón «Confiar en este emisor»** ahí mismo,
mostrando la huella SHA-256 antes de decidir. Mandar al usuario a preferencias,
como hace Acrobat, es justo lo que hace que las firmas se queden en amarillo para
siempre.

## 8. Riesgos conocidos

- **PIN y sesiones PKCS#11**: algunos tokens exigen login por operación; hay que manejar
  `use_raw_mechanism` para tokens sin hash-then-sign.
- **Coordenadas**: resuelto en `geometry.py`, con conversión en los dos sentidos,
  rotación `/Rotate` y `CropBox ≠ MediaBox`, y pruebas de ida y vuelta para los
  cuatro giros. Es el módulo que más pruebas tiene por línea de código y con razón.
- **PDFs firmados y luego "reparados"** por otras herramientas: cualquier reescritura
  rompe firmas. Nuestro guardado siempre debe ser incremental.
- **Flatpak + tokens USB**: hace falta acceso a `pcscd` (`--socket=pcsc` / permisos de
  dispositivo). Probar temprano, es un bloqueante clásico.
