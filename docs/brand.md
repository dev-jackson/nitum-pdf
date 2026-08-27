# Identidad visual de la suite Nitum

Nitum PDF es la primera aplicación de una familia. El icono se diseña como
sistema, no como pieza suelta: lo que se repite identifica la suite, lo que
cambia identifica la aplicación.

## Lo que comparte toda la familia

| Elemento | Valor |
| --- | --- |
| Lienzo | 128 × 128, contenido dentro de `6 6 116 116` |
| Placa | Rectángulo redondeado, radio `28` (21.9 % del lado) |
| Relleno | Degradado diagonal de `(20,10)` a `(108,118)`, claro arriba a la izquierda |
| Hoja | Silueta blanca con esquina doblada arriba a la derecha |
| Doblez | Triángulo en el tono claro de la aplicación |
| Glifo | Una sola forma sobre la hoja, en el tono oscuro de la aplicación |

## Lo que cambia en cada aplicación

- **El color.** Nitum PDF es rojo (`#E4323C` → `#A00E1C`). Es la misma familia
  cromática que Acrobat porque la persona que llega desde Windows busca un
  icono rojo para documentos; el parecido termina ahí: la placa, el doblez y el
  glifo son propios.
- **El glifo.** En Nitum PDF es un trazo de firma continuo, porque firmar es lo
  que distingue a esta aplicación de cualquier visor.

Una aplicación futura de la suite conserva placa, hoja y doblez, y cambia
únicamente el par de colores y el glifo.

## Reglas de dibujo

1. El glifo nunca baja de `8` unidades de grosor de trazo: por debajo desaparece
   a 16 px.
2. La hoja ocupa al menos el 55 % del lado de la placa; menos que eso deja de
   leerse como documento.
3. Nada de sombras internas ni detalles menores de `4` unidades. Se verifica
   rasterizando a 16, 24, 32 y 48 px sobre fondo claro y oscuro antes de aceptar
   un cambio.
4. Un solo trazo de acento por icono. Un segundo trazo decorativo se lee como
   suciedad, no como firma.

## Archivos

| Archivo | Uso |
| --- | --- |
| `data/com.nitum.Pdf.svg` | Icono escalable instalado en `hicolor/scalable/apps` |
| `data/com.nitum.Pdf.desktop` | Entrada de escritorio; `Icon=com.nitum.Pdf` |
| `packaging/native/nitum-pdf.ico` | Icono de Windows, generado desde el SVG a 256 px |
| `data/nitum-family-mark.png` | Marca de familia para material de la suite |

El identificador `com.nitum.Pdf` aparece en tres sitios y los tres deben
coincidir: el nombre del `.desktop`, el nombre del SVG en `hicolor`, y la
llamada `slint::set_xdg_app_id` en `native/src/presentation.rs`. Si uno se
desalinea, Wayland deja de encontrar el icono y la ventana aparece sin él.

## Regenerar el icono de Windows

```sh
rsvg-convert -w 256 -h 256 data/com.nitum.Pdf.svg -o /tmp/nitum-256.png
rustc --edition 2021 -O packaging/native/generate-windows-icon.rs -o /tmp/genico
/tmp/genico /tmp/nitum-256.png packaging/native/nitum-pdf.ico
```

`rustc` necesita `--edition 2021`; sin la bandera usa la edición 2015 y
`u32::try_from` no está en el preludio.
