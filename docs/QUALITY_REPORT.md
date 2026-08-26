# Evidencia de calidad nativa

Esta bitácora registra mediciones reproducibles; no sustituye las matrices de
GitHub Actions ni convierte una medición local en garantía universal.

## Línea base del 26 de agosto de 2026

Equipo de referencia: macOS 27 arm64, compilación `release`, backend de pruebas
Slint por software.

| Escenario | Tiempo real | RSS máximo | Resultado |
|---|---:|---:|---|
| 20 escenas light/dark, diálogos, búsqueda y ventana 720×560 | 0,22 s | 117,2 MiB | Aprobado |
| Abrir PDF real, renderizar, buscar y extraer texto con PDFium | 0,41 s | 15,6 MiB | Aprobado |
| Firma PAdES B-T contra DigiCert TSA y verificación RFC 3161 | 0,54 s | — | Aprobado |
| Dos firmas PAdES B-B incrementales consecutivas | 0,19 s | — | Aprobado |
| Certificación DocMDP P=1 con comprobación criptográfica y estructural | 0,17 s | — | Aprobado |
| PAdES B-LT con DSS/VRI y conservación de B-T | 0,60 s | — | Aprobado |
| PAdES B-LTA con DSS/VRI y segundo sello de archivo | 0,85 s | — | Aprobado |
| Validación externa DSS 6.4 (Comisión Europea), B-LTA de ensayo | — | — | Baseline-T reconocido; estructura y sellos sin fallos de formato |

Comandos utilizados, después de compilar las pruebas en modo release:

```bash
/usr/bin/time -l native/target/release/deps/visual_shell-* \
  --exact light_dark_and_signature_center_render_headlessly

DYLD_LIBRARY_PATH="$PWD/native/target/release" /usr/bin/time -l \
  native/target/release/deps/pdfium_adapter-* --ignored \
  --exact opens_and_renders_a_real_pdf
```

## Cobertura visual actual

Las capturas headless incluyen inicio claro/oscuro, documento continuo,
colocación, centro y confirmación de firma, bibliotecas de identidades y firmas
visuales en claro/oscuro y ancho compacto, búsqueda y selección de texto
compactas, actualización disponible, aplicación actualizada y certificación
DocMDP con sus permisos.

Las metas de inicio interactivo y 60 FPS todavía deben medirse en cada sistema
operativo sobre los paquetes finales firmados; no se consideran demostradas por
esta línea base headless.

La prueba B-T exige una firma de aprobación íntegra, un `/DocTimeStamp`
íntegro y confiable, y que el verificador detecte el nivel efectivo B-T. La
prueba sucesiva vuelve a verificar ambas firmas después de la segunda revisión
incremental. La certificación comprueba además `/Perms /DocMDP`, `/Reference` y
el permiso elegido. B-LT exige una cadena completa y datos OCSP o CRL cuando
existe una autoridad emisora; B-LTA vuelve a sellar el documento después de
incorporar el DSS. Ambos fallan cerrados si el material requerido no existe.

## Interoperabilidad externa PAdES

El 26 de agosto de 2026 se envió exclusivamente un PDF de prueba generado con
un certificado efímero —nunca un documento del usuario— al servicio oficial de
demostración DSS 6.4 de la Comisión Europea. La revisión detectó y permitió
corregir dos defectos que las pruebas internas no exponían: la ausencia de `/M`
impedía clasificar la firma como Baseline y dos sellos reutilizaban el mismo
nombre de campo. Después de la corrección, DSS reconoce
`PAdES-BASELINE-T`, acepta `SigningCertificateV2`, no informa
`FORMAT_FAILURE`, identifica dos sellos RFC 3161 distintos y confirma que el
último cubre el PDF completo.

El certificado del firmante de esta prueba es autofirmado. Por eso DSS termina
en `INDETERMINATE/NO_CERTIFICATE_CHAIN_FOUND` y no puede demostrar externamente
LT/LTA: no existe una autoridad emisora ni evidencia pública de revocación para
él. En producción, Nitum incorpora al DSS la cadena y OCSP/CRL tanto del
firmante como de la TSA, y rechaza la creación de LT/LTA cuando una cadena
emitida está incompleta, revocada o carece de evidencia válida. Una afirmación
externa de LT/LTA queda pendiente de repetir esta prueba con una identidad de
ensayo emitida y confiable; no se sustituye por el resultado autofirmado.
