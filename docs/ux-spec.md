# Especificación de UX e interfaz — Nitum PDF

Fecha: 26 de agosto de 2026. Documento normativo: cada regla se puede aplicar
editando `native/ui/theme.slint`, `native/ui/components.slint` y
`native/ui/app.slint`. No contiene consejos genéricos; cuando dice un número,
ese número va al código.

**Base de trabajo.** El repositorio ya tiene un sistema de tokens
(`native/ui/theme.slint`) y primitivas (`native/ui/components.slint`). Esta
especificación **no lo sustituye**: lo verifica contra fuentes documentadas,
corrige cinco colores que no cumplen WCAG AA (medidos, no estimados) y define
lo que todavía no existe (elevación, tipografía con altura de línea, rejilla de
ventana, flujo de firma, mapa de teclado).

**Unidades.** En Slint `px` es un píxel lógico que escala con el factor de la
pantalla, así que equivale 1:1 al `pt` de las Human Interface Guidelines de
Apple. Todas las cifras de Apple citadas aquí se transfieren sin conversión.

**Referentes y qué se toma de cada uno.**

| Referente | Lo que se adopta | Lo que no |
| --- | --- | --- |
| Apple HIG | Escala tipográfica, tamaños mínimos de control, estructura de barra y panel lateral, reglas de foco y teclado | Materiales translúcidos y «Liquid Glass» |
| Linear | Densidad, jerarquía por peso y espacio en vez de cajas, superficies por elevación, paleta con pocas variables, `?` para atajos | Su acento cromático y su tipografía de pago |
| Stripe | Redacción por consecuencia, validación en línea, una acción principal por paso, escala de color perceptual | Su densidad de formulario web |
| Figma | Herramienta contextual junto al lienzo, manipulación directa, guías de alineación, lectura de zoom | Su barra de herramientas flotante con muchos modos |
| Adobe Acrobat | Vocabulario de firma y el modelo «colocar y luego confirmar» | Sus diálogos apilados y su densidad de opciones |

---

## A. Tokens de diseño

### A.1 Escala tipográfica

Apple documenta para macOS un cuerpo por defecto de **13 pt** y un mínimo de
**10 pt**, y desaconseja explícitamente los pesos Ultralight, Thin y Light. La
escala de abajo es la escala macOS de Apple, recortada a los ocho pasos que esta
aplicación necesita.

| Token | px | Peso | Caja de línea (px) | Equivalente Apple | Uso exacto |
| --- | --- | --- | --- | --- | --- |
| `Type.caption` | 11 | 500 / 600 | 14 | Subheadline 11/14 | `StatusPill`, metadatos, texto de ayuda secundario |
| `Type.small` | 12 | 400 / 500 | 15 | Callout 12/15 | Detalle de fila, ayuda bajo un campo, etiqueta de campo |
| `Type.body` | 13 | 400 / 500 | 16 | Body 13/16 | Texto por defecto de toda la interfaz y de los botones |
| `Type.emphasis` | 14 | 600 | 18 | — | Título de fila seleccionable, etiqueta de acción principal |
| `Type.heading` **(nuevo)** | 15 | 600 | 20 | Title 3 15/20 | Título de sección dentro de un diálogo |
| `Type.title` | 17 | 700 | 22 | Title 2 17/22 | Título de diálogo |
| `Type.display` | 22 | 700 | 26 | Title 1 22/26 | Título de la pantalla sin documento |
| `Type.hero` **(nuevo)** | 26 | 700 | 32 | Large Title 26/32 | Sólo el estado vacío inicial |

Reglas:

1. Mínimo absoluto **11 px**. Apple permite 10 pt en macOS; dejamos 1 px de
   margen porque la UI está en español y las palabras son más largas.
2. Pesos permitidos: 400, 500, 600, 700. Ningún peso por debajo de 400.
3. **Slint 1.17 no tiene propiedad `line-height` en `Text`.** La caja de línea de
   la tabla se realiza como `height` fija del contenedor de una sola línea, o
   como `spacing` del `VerticalLayout` en párrafos: `spacing = caja − px`
   (ejemplo: párrafo de `Type.body` → `spacing: 3px`).
4. Un nivel de jerarquía se marca **primero por peso, después por color, y sólo
   al final por tamaño**. Dos textos consecutivos no pueden diferir a la vez en
   tamaño, peso y color.
5. El título del documento en la barra usa `Type.body` a peso 600, no un tamaño
   mayor: la barra no compite con el documento.

### A.2 Escala de espaciado

Base de 4 px. Los tokens actuales son correctos; se añade uno.

| Token | px | Uso |
| --- | --- | --- |
| `Space.xxs` **(nuevo)** | 2 | Separación título/detalle dentro de una fila |
| `Space.xs` | 4 | Icono ↔ texto en una píldora |
| `Space.sm` | 8 | Icono ↔ texto en un botón; separación entre controles del mismo grupo |
| `Space.md` | 12 | Padding horizontal de botón; separación entre filas de una lista |
| `Space.lg` | 16 | Padding de tarjeta; separación entre campos de un formulario |
| `Space.xl` | 24 | Padding de diálogo; separación entre grupos de controles |
| `Space.xxl` | 32 | Margen lateral del lienzo en tamaño regular |
| `Space.section` | 40 | Separación entre bloques conceptualmente distintos |

Reglas:

1. **Padding interior < separación exterior.** Si una tarjeta tiene 16 de
   padding, la separación entre tarjetas es ≥ 24. Es lo que hace que un grupo se
   lea como grupo sin necesitar borde.
2. Apple recomienda ~12 pt de separación alrededor de elementos con bisel y
   ~24 pt alrededor de elementos sin bisel. Aplicado: botones con relleno se
   separan `Space.md`; acciones de sólo icono sin fondo, `Space.xl` de cualquier
   otro grupo.
3. Ningún valor fuera de la escala. Los `padding: 14px`, `28px`, `18px` y
   `54px` que hoy hay en `app.slint` se sustituyen por el token más cercano.

### A.3 Radios

| Token | px | Uso |
| --- | --- | --- |
| `Radius.xs` **(nuevo)** | 4 | Chip de icono 20×20, marca de verificación |
| `Radius.sm` | 6 | Campo de texto, acción de sólo icono |
| `Radius.md` | 10 | Botón, fila de lista, tarjeta |
| `Radius.lg` | 14 | Barra flotante, aviso, panel |
| `Radius.xl` **(nuevo)** | 20 | Diálogo y tarjeta de bienvenida |
| `Radius.pill` | 999 | `StatusPill` |

Regla de concentricidad (Apple): el radio interior = radio exterior − padding.
Un icono con `Radius.xs` dentro de una fila con `Radius.md` y 6 de padding es
correcto; `Radius.lg` dentro de `Radius.lg` no lo es. Los `border-radius: 28px`,
`26px`, `17px`, `13px`, `12px` y `11px` actuales se colapsan a esta escala.

### A.4 Grosores de línea

| Token | px | Uso |
| --- | --- | --- |
| Hairline | 1 | Separador y borde de tarjeta: `Theme.border` |
| Borde de control | 1 | Único límite visible de un campo o control: `Theme.border-control` **(nuevo)** |
| Anillo de foco | 2 | Todos los controles enfocables |

WCAG 2.2 SC 1.4.11 exige 3:1 sólo para la información visual **necesaria para
identificar un control**. Un separador decorativo está exento, y por eso el
hairline a 1.28:1 es legítimo. El borde de un `LineEdit` **no** está exento: es
lo único que dice dónde se escribe. De ahí el token nuevo (valores en A.7).

### A.5 Elevación

Slint expone `drop-shadow-blur`, `drop-shadow-color` y `drop-shadow-offset-y` en
`Rectangle`. Cuatro niveles, ni uno más:

| Nivel | Uso | blur | offset-y | color claro | color oscuro |
| --- | --- | --- | --- | --- | --- |
| `e0` | Tarjetas, filas, barra superior | 0 | 0 | — | — |
| `e1` | Página del PDF sobre el lienzo | 12px | 2px | `#0f172a1f` | `#00000066` |
| `e2` | Barra flotante, aviso, popover | 24px | 8px | `#0f172a29` | `#00000080` |
| `e3` | Diálogo modal | 40px | 16px | `#0f172a33` | `#0000009e` |

Regla (Linear): **la elevación es para lo que flota, no para lo que agrupa.**
Una tarjeta que no flota no lleva sombra; se distingue por `Theme.raised` y una
hairline. Hoy la tarjeta de bienvenida usa `drop-shadow-blur: 32px` sin flotar:
pasa a `e0`.

### A.6 Movimiento

| Token | ms | Curva | Uso |
| --- | --- | --- | --- |
| `Motion.instant` | 90 | `ease-out` | Relleno de hover y pressed |
| `Motion.quick` | 140 | `ease-out` | Aparición de avisos, píldoras, cambios de estado |
| `Motion.calm` | 220 | `ease-in-out` | Diálogo + velo, apertura/cierre del panel lateral |

Slint soporta `linear`, `ease`, `ease-in`, `ease-out`, `ease-in-out`, las
variantes `quad/quart/quint/expo/sine/circ/back/elastic/bounce` y
`cubic-bezier(a,b,c,d)`. Se usan sólo las tres primeras familias: nada rebota en
una aplicación que firma documentos.

Reglas:

1. Nada por encima de **220 ms**. Nada por debajo de 90 ms (se percibe como
   parpadeo).
2. Se anima `background`, `opacity`, `x`, `y`, `height`. **No** se anima
   `border-width` (provoca salto de layout) ni `font-size`.
3. **Movimiento reducido:** token nuevo `Theme.reduced-motion: bool`, alimentado
   desde Rust leyendo `org.gnome.desktop.interface enable-animations` (GNOME) o
   la variable `NO_ANIMATIONS`. Cuando es `true`, las tres duraciones valen
   `0ms`. Se declara como propiedad `in-out` del global `Motion` para que un solo
   `set` desde Rust lo apague todo.

### A.7 Color

Medidas hechas con la fórmula de luminancia relativa de WCAG 2.x, componiendo
previamente los colores con alfa sobre su fondo real. Objetivo: **4.5:1** para
texto normal, **3:1** para texto ≥ 18 px o ≥ 14 px en negrita, elementos de
interfaz y anillos de foco (WCAG 2.2 SC 1.4.3, 1.4.11 y 2.4.13).

#### Modo claro

| Rol | Hex | Contra | Ratio | Veredicto |
| --- | --- | --- | --- | --- |
| `canvas` | `#f4f5f8` | — | — | Fondo del lienzo |
| `surface` | `#ffffff` | `canvas` | 1.09 | Se distingue por hairline, no por contraste |
| `raised` | `#ffffff` | `canvas` | 1.09 | Igual que `surface` en claro (correcto) |
| `sunken` | `#e9ebf1` | `surface` | 1.19 | Fondo de control deshabilitado |
| `text` | `#14171f` | `surface` / `canvas` | **17.92 / 16.43** | AA y AAA |
| `text-secondary` | `#5a6070` | `surface` / `canvas` | **6.28 / 5.76** | AA |
| `text-tertiary` | ~~`#767d8d`~~ → **`#666d7c`** | `surface` / `canvas` | 4.13 / 3.79 → **5.19 / 4.76** | **Corrección 1** |
| `accent` | `#2f5fe0` | `surface` / `canvas` | **5.48 / 5.02** | AA |
| `accent-hover` | `#2450c4` | blanco encima | **6.95** | AA |
| `accent-pressed` | `#1c40a3` | blanco encima | **9.08** | AA |
| `accent-soft` | `#2f5fe014` | `accent` encima | **4.90 / 4.52** | AA sobre `surface` y sobre `canvas` |
| `success` | ~~`#0f8f5b`~~ → **`#00764a`** | `surface` / tinte | 4.12 / 3.44 → **5.69 / 4.68** | **Corrección 2** |
| `warning` | ~~`#a86800`~~ → **`#8a5300`** | `surface` / tinte | 4.52 / 3.78 → **6.33 / 5.20** | **Corrección 3** |
| `danger` | `#c4242f` | `surface` / tinte | **5.77 / 5.09** | AA |
| `text-on-accent` | `#ffffff` | `accent` | **5.48** | AA |
| `border` | `#0f172a1f` | `surface` | 1.28 | Decorativo, exento |
| `border-control` **(nuevo)** | `#0f172a7a` | `surface` / `canvas` | **3.20 / 3.14** | Cumple 1.4.11 |
| `focus-ring` | `#1c40a3` | `surface` / `canvas` / `sunken` | **9.08 / 8.33 / 7.62** | Cumple 2.4.13 |
| `scrim` | `#0f172a66` | — | — | Velo de modal |
| `page` **(nuevo)** | `#ffffff` | — | — | Papel del PDF, idéntico en ambos modos |
| `page-edge` **(nuevo)** | `#0f172a29` | `canvas` | 1.4 + sombra `e1` | Borde del papel |

#### Modo oscuro

| Rol | Hex | Contra | Ratio | Veredicto |
| --- | --- | --- | --- | --- |
| `canvas` | `#0f1115` | — | — | Fondo del lienzo |
| `surface` | `#171a21` | `canvas` | 1.09 | Superficie base |
| `raised` | `#1e222b` | `surface` | 1.09 | Superficie elevada |
| `sunken` | `#0b0d11` | `surface` | 1.12 | Hueco |
| `text` | `#f6f7fa` | `surface` / `raised` | **16.25 / 14.86** | AA y AAA |
| `text-secondary` | `#b3bac7` | `surface` / `raised` | **8.93 / 8.16** | AA |
| `text-tertiary` | `#8b94a5` | `surface` | **5.70** | AA |
| `accent` | `#5b8cff` | `surface` | **5.51** | AA como texto |
| `accent-hover` | `#7ba2ff` | tinta oscura encima | **7.61** | AA |
| `accent-pressed` | `#4a79e6` | tinta oscura encima | **4.65** | AA |
| `success` | `#3ecf8e` | `surface` | **8.72** | AA |
| `warning` | `#f0b429` | `surface` | **9.34** | AA |
| `danger` | `#ff6b6b` | `surface` | **6.27** | AA |
| `text-on-accent` | ~~`#ffffff`~~ → **`#0f1115`** | `accent` / `danger` / `success` / `warning` | 3.16 / 2.78 / 2.00 / 1.86 → **5.97 / 6.81 / 9.47 / 10.14** | **Corrección 4** |
| `border` | `#ffffff1f` | `surface` | 1.44 | Decorativo, exento |
| `border-control` **(nuevo)** | `#ffffff59` | `surface` / `canvas` | **3.21 / 3.19** | Cumple 1.4.11 |
| `focus-ring` | `#8fb3ff` | `surface` / `raised` | **8.34 / 7.63** | Cumple 2.4.13 |
| `scrim` | `#00000099` | — | — | Velo de modal |

#### Las cinco correcciones, en una frase cada una

1. **`text-tertiary` claro** `#767d8d` → `#666d7c`: a 3.79:1 sobre el lienzo no
   era texto legible, era texto decorativo.
2. **`success` claro** `#0f8f5b` → `#00764a`: fallaba como texto (4.12:1), como
   texto sobre su propio tinte (3.44:1) y como fondo de botón con tinta blanca
   (4.12:1). Los tres pasan a la vez con el valor nuevo.
3. **`warning` claro** `#a86800` → `#8a5300`: pasaba raspando sobre blanco
   (4.52:1) y fallaba sobre `warning-soft` (3.78:1), que es exactamente donde se
   usa.
4. **`text-on-accent` en oscuro** debe ser tinta oscura `#0f1115`, no blanco.
   Blanco sobre `#5b8cff` da 3.16:1 y sobre `#3ecf8e` da 2.00:1: los botones
   rellenos del modo oscuro son hoy ilegibles según AA. Con tinta oscura, los
   cuatro rellenos pasan (5.97 / 6.81 / 9.47 / 10.14).
5. **`border-control`** es un token nuevo, no una corrección de valor: sin él
   ningún campo de texto cumple SC 1.4.11 en ninguno de los dos modos.

**Efecto colateral que resuelve un quinto problema:** `ActionButton` ya dibuja el
anillo de foco de los botones rellenos con `Theme.text-on-accent`. Con la
corrección 4, ese anillo pasa de 3.16:1 a 5.97:1 sobre el relleno, y cumple el
requisito de 3:1 entre estado enfocado y no enfocado de SC 2.4.13 sin tocar
`components.slint`. El anillo de foco **nunca** se dibuja con `focus-ring` sobre
un relleno de acento: `#8fb3ff` sobre `#5b8cff` da 1.52:1.

#### Reglas de color

1. **Un solo acento para la interacción.** El rojo de marca pertenece al icono de
   la aplicación y a `danger`; nunca a una acción normal.
2. **Ningún estado depende sólo del color** (regla 8 de `DESIGN.md`, y guía de
   Apple sobre color): icono con forma distinta + palabra + tinte. Ver C.8.
3. Los tintes `*-soft` son fondos, nunca texto ni bordes.
4. El papel del PDF es blanco en los dos modos. Un documento no se «oscurece»:
   sería falsear el contenido. En modo oscuro el papel destaca solo; en modo
   claro necesita `page-edge` + sombra `e1`, porque blanco sobre `#f4f5f8` es
   1.09:1 y el papel se pierde por el borde.

---

## B. Sistema de disposición

### B.1 Ventana

| Propiedad | Valor | Motivo |
| --- | --- | --- |
| `preferred-width` / `preferred-height` | 1180 × 820 | Actual; cabe una página A4 al 100 % con panel lateral |
| `min-width` / `min-height` | **880 × 620** (hoy 720 × 560) | A 720 px la barra flotante de 680 px deja 20 px por lado y se lee como desbordada |
| Estado guardado | Tamaño y posición entre sesiones | Apple: la ventana debe volver donde estaba |
| Título | `«nombre.pdf» — Nitum PDF` | Apple: no titular sólo con el nombre de la app |

### B.2 Puntos de ruptura

| Nombre | Ancho | Qué cambia |
| --- | --- | --- |
| `compact` | < 900 px | Panel lateral oculto; barra flotante colapsa a 5 controles + desbordamiento; título del documento sale de la barra y pasa a la barra de estado |
| `regular` | 900 – 1439 px | Panel lateral disponible pero cerrado por defecto; barra completa |
| `wide` | ≥ 1440 px | Panel lateral abierto por defecto; margen lateral del lienzo a `Space.section` |

Regla (Apple, panel lateral): el panel se **oculta y se revela automáticamente**
al redimensionar, pero si la persona lo cerró a mano, esa decisión gana hasta que
cambie de punto de ruptura. Nunca cambia la etiqueta de un control al colapsar:
«Abrir» no se convierte en «+ PDF». Si no cabe el texto, se queda sólo el icono
con el mismo `accessible-label`.

### B.3 Barra superior

Altura **48 px** = control de 32 px + `Space.sm` arriba y abajo. Hoy son 64 px
con los controles pegados al borde superior.

```
┌ 48 px ────────────────────────────────────────────────────────────────┐
│ [◧] [Abrir]        contrato.pdf · 12 páginas        [Buscar][◐][Firmar]│
└───────────────────────────── hairline inferior ───────────────────────┘
  12                                                                  12
```

- `padding-left` / `padding-right`: `Space.md` (12).
- Separación **dentro** de un grupo: `Space.xs` (4). **Entre** grupos:
  `Space.lg` (16). Máximo **3 grupos** (Apple).
- Grupo inicial (no personalizable, Apple): alternar panel lateral, «Abrir».
- Centro: título del documento en `Type.body`/600 y, debajo, `12 páginas · local`
  en `Type.caption`/`text-secondary`.
- Grupo final (siempre visible, Apple): «Buscar», tema, versión, y la acción
  principal «Firmar este PDF».
- **El título se centra respecto a la ventana, no respecto al espacio sobrante.**
  Se implementa como un `Text` posicionado con
  `x: (parent.width - self.width) / 2` dentro del `Rectangle` de la barra, fuera
  del `HorizontalLayout`. Con espaciadores de igual peso el centro se desplaza
  tanto como la diferencia entre los dos grupos.
- **Hairline sólo abajo.** Un `Rectangle` con `border-width: 1px` dibuja los
  cuatro lados. Se usa un `Rectangle { height: 1px; background: Theme.border; }`
  como último hijo de un `VerticalLayout`.
- La barra usa `e0`. La separación con el lienzo la da la hairline.

### B.4 Panel lateral de páginas

| Propiedad | Valor |
| --- | --- |
| Ancho por defecto | 200 px |
| Mínimo / máximo redimensionando | 160 / 280 px |
| Miniatura | Ancho del panel − 2·`Space.lg`; alto según proporción de la página |
| Separación entre miniaturas | `Space.md` |
| Número de página | `Type.caption`, `text-secondary`, centrado bajo la miniatura |
| Página actual | Anillo de 2 px `accent` alrededor de la miniatura **y** número en peso 700 |
| Jerarquía | Un solo nivel (Apple: máximo dos) |
| Abajo | Nada. Apple: no poner información crítica al fondo de un panel lateral |

### B.5 Lienzo del documento

| Propiedad | `compact` | `regular` | `wide` |
| --- | --- | --- | --- |
| Margen lateral | `Space.lg` (16) | `Space.xxl` (32) | `Space.section` (40) |
| Margen superior/inferior | `Space.xl` (24) | `Space.xl` (24) | `Space.xl` (24) |
| Separación entre páginas | `Space.lg` (16) | `Space.lg` (16) | `Space.lg` (16) |
| Ancho de página | `min(ancho útil, 980px)` al ajustar | igual | igual |

- La página se dibuja con `Theme.page`, borde `page-edge` de 1 px, `Radius.xs` y
  sombra `e1`.
- Regla 1 de `DESIGN.md`: al abrir, `fit-width`. El zoom por defecto es el que
  resulte de ajustar al ancho útil, no 100 %.
- La barra flotante nunca tapa la página: el margen inferior del lienzo es
  `20 + 44 + 12 = 76 px` cuando la barra está visible.

### B.6 Barra contextual flotante (modelo Figma)

| Propiedad | Valor |
| --- | --- |
| Altura | 44 px |
| Ancho | Contenido, máximo 680 px |
| Posición | Centrada; `y = parent.height - self.height - 20` |
| Radio | `Radius.lg` |
| Fondo / borde / sombra | `Theme.raised` / hairline `Theme.border` / `e2` |
| Padding | `Space.sm` |
| Separación | `Space.xs` dentro de grupo, `Space.md` entre grupos, separador de 1×20 px |

Contenido en `regular`: `◀  Página [3] de 12  ▶ │ − 120 % + │ Ajustar │
Seleccionar │ Copiar`.

En `compact` se conservan `◀ página ▶`, `−`, lectura de zoom, `+`, y el resto
pasa a un botón de desbordamiento `···` (Apple: el sistema colapsa en un menú de
desbordamiento; aquí lo hacemos explícito). **El indicador de página es un campo
editable**, no una etiqueta: escribir «7» + Enter salta a la página 7.

La lectura de zoom es un botón que abre un menú con `50 % · 100 % · 200 % ·
Ajustar ancho · Ajustar página` (Figma pone el porcentaje de zoom como control,
no como texto muerto).

### B.7 Modos del lienzo

Colocar la firma y seleccionar texto son **modos**, y un modo se anuncia y se
puede abandonar:

1. El cursor cambia (`crosshair` para colocar, `text` para seleccionar).
2. La barra flotante se sustituye por una barra de modo con el mismo tamaño y
   posición: título del modo a la izquierda, «Cancelar» a la derecha.
3. `Esc` siempre sale del modo antes de cerrar nada más.
4. **El velo azul sobre la página desaparece.** Hoy el modo de colocación pinta
   `#2563eb18` sobre toda la página y un texto centrado encima del contenido: eso
   oculta justo el documento sobre el que hay que decidir. El modo se indica con
   el cursor, la barra de modo y un borde de 2 px `accent` alrededor de la página.

---

## C. Especificación de componentes

### C.1 Botones

Cuatro variantes, una sola geometría:

| Propiedad | Valor |
| --- | --- |
| Altura | **36 px** (por defecto) · **32 px** en barras y filas densas |
| Padding horizontal | `Space.md` (12); 16 si la etiqueta supera 18 caracteres |
| Icono | 16×16, `Space.sm` de separación con el texto |
| Radio | `Radius.md` (10) |
| Ancho mínimo | 96 px |
| Tipografía | `Type.body`; peso 600 en `primary`/`danger`, 500 en el resto |
| Área de pulsación | ≥ 32×32 (Apple macOS: control por defecto 28×28, mínimo 20×20) |

Estados, con tokens exactos:

| Variante | Reposo | Hover | Pulsado | Foco | Deshabilitado |
| --- | --- | --- | --- | --- | --- |
| `primary` | fondo `accent`, tinta `text-on-accent`, sin borde | fondo `accent-hover` | fondo `accent-pressed` | anillo interior 2 px `text-on-accent` | fondo `sunken`, tinta `text-tertiary`, borde `border` |
| `secondary` | fondo `raised`, borde 1 px `border-control`, tinta `text` | fondo `accent-soft` | fondo `accent-soft`, borde `accent` | anillo 2 px `focus-ring` | fondo `sunken`, tinta `text-tertiary`, borde `border` |
| `ghost` | sin fondo ni borde, tinta `text` | fondo `accent-soft` | fondo `border` | anillo 2 px `focus-ring` | tinta `text-tertiary` |
| `danger` | fondo `danger`, tinta `text-on-accent` | `danger` al 92 % de luminosidad | `danger` al 84 % | anillo interior 2 px `text-on-accent` | fondo `sunken`, tinta `text-tertiary` |

Reglas:

1. **`opacity: 0.4` no se usa para deshabilitar.** Un primario azul al 40 % sobre
   blanco parece un fallo de render. Se usan los tokens de la columna
   «Deshabilitado». WCAG exime a los controles inactivos del contraste, pero no
   de parecer inactivos en vez de rotos.
2. **Una sola acción principal por vista** (Apple: «una o dos como máximo»;
   Stripe: «un movimiento siguiente evidente»). Hoy la pantalla vacía tiene
   «Abrir un PDF» en la tarjeta y «Abrir» en la barra: en esa pantalla el de la
   barra pasa a `ghost`.
3. **Nunca se asigna el rol primario a una acción destructiva** (Apple). En un
   diálogo de confirmación destructiva, el botón peligroso es `danger` y el
   primario por defecto (el que responde a Enter) es «Cancelar».
4. La etiqueta empieza por verbo y nombra la consecuencia: «Firmar y guardar
   copia», no «Aceptar».
5. Puntos suspensivos «…» sólo si el botón abre otra ventana o un selector de
   archivos: «Elegir archivo…» sí, «Firmar…» no.
6. El borde con foco se dibuja **hacia dentro** (`border-width` 2 y padding
   reducido en 1) para no desplazar el layout.

### C.2 Campos de texto

| Propiedad | Valor |
| --- | --- |
| Altura | 30 px |
| Padding horizontal | 10 px |
| Radio | `Radius.sm` (6) |
| Borde reposo | 1 px `border-control` |
| Borde hover | 1 px `text-tertiary` |
| Borde foco | 2 px `focus-ring` + fondo `surface` |
| Borde error | 2 px `danger` |
| Fondo | `surface` en claro, `sunken` en oscuro |
| Etiqueta | Encima, `Type.small`/500, `text-secondary`, `Space.xs` de separación |
| Ayuda | Debajo, `Type.caption`, `text-tertiary` |
| Error | Debajo, `Type.small`, `danger`, precedido de un icono `!` de 13 px |

Reglas (Apple + Stripe):

1. **Etiqueta y marcador de posición a la vez.** El marcador desaparece al
   escribir; la etiqueta es lo que queda. Hoy «Contraseña de la identidad» y
   «Se usa una vez y no se guarda» ya cumplen esto: se mantiene el patrón.
2. **Validación al salir del campo**, nunca por pulsación. El error se muestra
   junto al campo que lo causa, con texto accionable: «El PIN debe tener entre 4
   y 8 dígitos», no «Valor no válido».
3. Nunca se borra lo que la persona escribió al mostrar un error. Excepción
   deliberada: contraseñas y PIN se limpian tras un intento fallido, y se dice
   («Vuelve a escribir la contraseña»).
4. El ancho del campo sugiere la longitud esperada: PIN 120 px, contraseña
   ancho completo, «Motivo» ancho completo.

### C.3 Filas de lista

| Propiedad | Valor |
| --- | --- |
| Altura | 40 px (densa) · **56 px** (con detalle, actual `ChoiceRow`) |
| Padding horizontal | `Space.md` |
| Radio | `Radius.md` |
| Icono | 20×20, `text-secondary`; `accent` si está seleccionada |
| Título | `Type.body`/600, `overflow: elide` |
| Detalle | `Type.small`, `text-secondary`, `overflow: elide` |
| Hover | fondo `accent-soft` |
| Seleccionada | fondo `accent-soft` **+ icono de verificación 13 px `accent` al final** |
| Foco | anillo 2 px `focus-ring` |
| Separación | Hairline entre filas, nunca antes de la primera ni después de la última |

La marca de verificación es obligatoria: `accent-soft` como único indicador de
selección es color solo (regla 8 de `DESIGN.md`). Hoy `ChoiceRow` pinta el mismo
fondo en hover y en seleccionada, así que además de indistinguibles para quien no
ve el color, son indistinguibles para todo el mundo.

### C.4 Tarjetas

Fondo `raised`, hairline `border`, `Radius.md`, padding `Space.lg`, **sin
sombra**. Altura por contenido, nunca fija. La tarjeta de bienvenida es la única
excepción de radio (`Radius.xl`) y usa padding `Space.xl` con altura natural: hoy
tiene 390 px fijos y 80 px muertos abajo.

### C.5 Diálogos

| Tipo | Ancho | Uso |
| --- | --- | --- |
| Confirmación | 420 px | Una pregunta, dos botones |
| Formulario | 560 px | Elegir identidad, contraseña del PDF, actualización |
| Revisión | 720 px | Confirmar firma, comprobar firmas |

- Alto máximo: 80 % de la ventana; el cuerpo desplaza, la cabecera y el pie no.
- Radio `Radius.xl`, sombra `e3`, velo `Theme.scrim` con `Motion.calm`.
- Cabecera 56 px: título `Type.title`, cierre `×` de 32×32 al final. «Atrás» sólo
  si hay un paso anterior real.
- Pie 56 px, hairline superior, alineado al final:
  `[Cancelar] [Acción principal]`. Apple: en una fila, Cancelar va al principio y
  el botón por defecto al final. **Máximo tres botones.**
- Un solo diálogo a la vez (Apple). Si de un diálogo sale otro, se cierra el
  primero. Hoy «Firmas visuales guardadas» y «Identidades digitales» vuelven a un
  diálogo anterior con `appearance-return-dialog`: eso es correcto porque es
  navegación dentro del mismo diálogo, y debe presentarse así, con «Atrás», no
  como diálogos apilados.
- `Esc` cierra. `Enter` activa la acción principal salvo que el foco esté en un
  campo multilínea.
- **Hoy el pie del diálogo de firma pone «Cancelar» a ancho completo debajo de un
  contenido desplazable.** Un botón de 512 px de ancho lee como acción principal.
  Pasa a `secondary` de ancho natural en el pie, junto al primario.

### C.6 Avisos (toasts)

| Propiedad | Valor |
| --- | --- |
| Altura | 44 px (una línea) · natural con `wrap` si son dos |
| Ancho | Contenido, mínimo 280, máximo 460 |
| Posición | Centrado, `y = altura de la barra flotante + 20 + 12` |
| Radio / fondo / sombra | `Radius.lg` / `Theme.raised` / `e2` |
| Contenido | Icono 16 px + texto `Type.small` + acción opcional `ghost` |
| Duración | 4 s para confirmación; **sin auto-cierre** para error |
| Simultáneos | Uno. El nuevo sustituye al anterior |

Un aviso nunca es el único sitio donde aparece un error importante: el error de
firma vive en el diálogo, no en un aviso que se va.

### C.7 Estados vacíos

Tres elementos y nada más: icono de 48 px con fondo `accent-soft` y `Radius.lg`,
título `Type.title`, texto `Type.small`/`text-secondary` de una línea, y una sola
acción `primary`. Ancho máximo 560 px, padding `Space.xl`, alto natural.

El icono debe significar el contenido que falta. Un «✓» de 42 px no significa
«abre un PDF»; el estado vacío inicial lleva un icono de documento.

### C.8 Indicadores de estado

`StatusPill`: 24 px de alto, `Radius.pill`, padding `Space.sm`, icono 13 px +
texto `Type.caption`/600. **Los tres canales son obligatorios: forma, palabra y
tinte.**

| Estado | Icono | Palabra | Tinta | Fondo |
| --- | --- | --- | --- | --- |
| Válido | `✓` círculo | «Firma válida» | `success` | `success-soft` |
| Desconocido | `?` círculo | «Validez desconocida» | `warning` | `warning-soft` |
| Advertencia | `!` triángulo | «Modificado tras firmar» | `warning` | `warning-soft` |
| No válido | `✕` círculo | «Firma no válida» | `danger` | `danger-soft` |
| En proceso | `···` | «Comprobando…» | `accent` | `accent-soft` |

Las cinco formas son distintas entre sí en monocromo. Prueba de aceptación:
convertir la captura a escala de grises y seguir distinguiendo los cinco.

---

## D. Flujo de firma

### D.1 Vocabulario canónico

Acrobat separa cuatro conceptos que la gente confunde, y esa separación es
precisamente lo que hace que su modelo mental funcione. Traducción fija para
Nitum (usar siempre estos términos, nunca sinónimos):

| Concepto | Término en Nitum | Equivalente Acrobat | Definición de una línea |
| --- | --- | --- | --- |
| Certificado + clave privada | **identidad digital** | Digital ID | Lo que demuestra quién firma |
| Operación criptográfica | **firma digital** | digital signature | Lo que hace verificable el documento |
| Dibujo de la rúbrica | **rúbrica visual** | signature appearance | Decoración; no prueba nada |
| Firma que admite firmas posteriores | **firma de aprobación** | approval signature | Se puede repetir |
| Firma única que bloquea el documento | **certificación** | certifying signature | Sólo una, y debe ser la primera |
| Revisión de firmas existentes | **comprobación** | validation | Integridad, cobertura y confianza |

Frase obligatoria allí donde aparezca una rúbrica visual (regla 4 de
`DESIGN.md`): «La validez proviene de tu certificado, no de la imagen.»

### D.2 El modelo: colocar y luego confirmar

Acrobat coloca primero el rectángulo sobre la página y **después** pide la
identidad y la contraseña; en Nitum se conserva ese orden porque la decisión
espacial se toma mirando el documento, no mirando un formulario. Figma aporta el
resto: el rectángulo se manipula directamente, con guías y lectura de tamaño.

Cinco pasos. Cada paso tiene **una sola** acción principal.

### Paso 1 — Elegir identidad

- **Título:** `Elige tu identidad digital`
- **Ayuda:** `La validez de la firma proviene de este certificado. Nitum no sube el documento a ningún servidor.`
- Dos filas (`ChoiceRow`, 56 px):
  - `Archivo .p12 o .pfx` — detalle: `La clave privada permanece en este equipo` — acción: `Elegir archivo…`
  - `Tarjeta, DNIe o token` — detalle: `La clave privada nunca sale de la tarjeta` — acción: `Buscar dispositivos`
- Una vez elegida, la fila muestra el titular del certificado, el emisor y la
  caducidad: `María Gómez Ruiz · FNMT Clase 2 CA · caduca el 14 mar 2027`.
- **Aviso de caducidad** (`StatusPill` de advertencia) si faltan menos de 30
  días: `Caduca en 12 días`.
- **Acción principal:** `Continuar`
- **Errores:**
  - `Este archivo no es una identidad .p12 o .pfx válida.`
  - `El certificado caducó el 3 may 2026. Una firma hecha ahora se marcará como no válida.`
  - `No encontramos ningún módulo PKCS#11. Instala opensc-pkcs11 o elige el archivo del módulo manualmente.`
  - `La tarjeta se ha desconectado. Vuelve a conectarla y pulsa Buscar dispositivos.`

### Paso 2 — Colocar la firma

- Se entra pulsando `Elegir posición en el documento`. El diálogo se **oculta**,
  no se cierra: el estado del formulario se conserva.
- **Barra de modo** (sustituye a la flotante):
  `Haz clic donde quieres que aparezca la rúbrica` · botón `Cancelar` · pista
  `Esc para cancelar` en `Type.caption`.
- Al hacer clic aparece un rectángulo por defecto de **148 × 48 pt**,
  arrastrable y redimensionable por las cuatro esquinas (mínimo 60 × 24 pt).
- **Retroalimentación de alineación (Figma):** guía de 1 px `accent` cuando el
  borde queda a ≤ 8 pt del margen de la página o alineado con otra firma
  existente; el rectángulo se ajusta a la guía.
- **Lectura de estado del objeto**, siempre visible en la barra de modo:
  `Página 3 · 148 × 48 pt · a 24 pt del margen inferior`.
- `Enter` confirma la posición y devuelve el diálogo. `Esc` cancela y devuelve el
  diálogo con la posición anterior intacta.
- Si no se elige posición: `Posición: esquina inferior izquierda de la página
  actual`, y el botón dice `Elegir posición en el documento`. Una vez elegida:
  `Posición: página 3` y el botón dice `Cambiar posición`.
- **Opción sin rúbrica:** casilla `Mostrar rúbrica visual en la página`
  (activada). Al desactivarla, el bloque de posición y apariencia se oculta y
  aparece: `La firma será invisible en la página, pero igual de verificable.`

### Paso 3 — Previsualizar la apariencia

- **Título de sección:** `Apariencia de la rúbrica`
- Vista previa **al tamaño real** que tendrá en la página, con la línea de datos
  verificables que se incrusta:
  ```
  Firmado por: María Gómez Ruiz
  Fecha: 26 ago 2026 14:03 (CEST)
  Motivo: Conformidad con el pliego
  ```
- Fila de acciones: `Guardadas` (biblioteca) · `Añadir imagen…`
- **Ayuda fija:** `La imagen es decorativa. Quien reciba el PDF comprobará tu certificado, no tu dibujo.`
- Campos opcionales `Motivo` y `Ubicación`, cada uno con etiqueta encima y ayuda
  `Se incrusta en la firma y es visible para quien la compruebe.`

### Paso 4 — Confirmar

Redacción por consecuencia (Stripe): la persona debe poder leer sólo esta
pantalla y saber qué archivo se crea, qué se le hace al original y qué tipo de
firma se aplica.

- **Título:** `Vas a firmar «contrato.pdf»`
- **Resumen** (cinco líneas, etiqueta `Type.small`/`text-secondary` + valor
  `Type.body`/500):

  | Etiqueta | Valor de ejemplo |
  | --- | --- |
  | `Firmas como` | `María Gómez Ruiz · FNMT Clase 2 CA` |
  | `Tipo de firma` | `Aprobación — permite firmas posteriores` |
  | `Nivel` | `PAdES B-T — con sello de tiempo` |
  | `Aparecerá en` | `Página 3, esquina inferior derecha` |
  | `Se guardará como` | `contrato-firmado.pdf` |

- **Campo de secreto:** etiqueta `Contraseña de la identidad` o `PIN de la
  tarjeta`; marcador `Se usa una vez y no se guarda`.
- **Nota de consecuencia**, siempre visible junto al botón:
  `Se creará un PDF nuevo. El original no se modifica.`
- **Acción principal:** `Firmar y guardar copia`
  (durante la operación: `Firmando…`, botón deshabilitado, `ProgressIndicator`).
- **Secundaria:** `Cancelar`
- **Selector de nivel PAdES**, con la consecuencia escrita en cada opción, no una
  sigla suelta:

  | Opción | Texto de ayuda |
  | --- | --- |
  | `B-B` | `Básica. Se comprueba con el certificado mientras sea válido.` |
  | `B-T` | `Con sello de tiempo. Demuestra cuándo se firmó.` |
  | `B-LT` | `Validación duradera. Incluye las pruebas de confianza dentro del PDF.` |
  | `B-LTA` | `Archivo a largo plazo. Recomendado para expedientes que se conservan años.` |

- **Errores** (bajo el campo que los causa):
  - `La contraseña no abre esta identidad. Vuelve a escribirla.`
  - `PIN incorrecto. Te quedan 2 intentos antes de que la tarjeta se bloquee.`
  - `No se pudo contactar con el servidor de sellado de tiempo. Puedes firmar sin sello (B-B) o reintentar.`
  - `No hay permiso de escritura en esa carpeta. Elige otra ubicación.`
  - `Este PDF ya está certificado y no admite cambios. Sólo puedes comprobarlo.`

### Paso 4-bis — Certificar (acción consecuente)

`Certificar` es irreversible en el sentido que le importa a la persona: sólo se
puede hacer una vez, tiene que ser la primera firma y limita lo que cualquiera
podrá hacer después. Por eso **no comparte botón con la firma de aprobación**.

- Selector `Uso`: `Aprobación` (por defecto) · `Certificar`
  - Ayuda de `Aprobación`: `Permite que otras personas firmen después.`
  - Ayuda de `Certificar`: `Debe ser la primera firma del documento. Sólo se puede hacer una vez.`
- Si se elige `Certificar`, aparece `Cambios permitidos después de certificar`:

  | Opción | Texto |
  | --- | --- |
  | `Ninguno` | `Nadie podrá modificar ni firmar el documento.` |
  | `Formularios` | `Se podrán rellenar campos y añadir firmas.` |
  | `Formularios y anotaciones` | `Además, se podrán añadir comentarios.` |

- La acción principal cambia a: `Certificar y bloquear el documento`
- **Confirmación adicional** (diálogo de 420 px), porque es la única operación
  del programa que no se puede deshacer ni repetir:
  - Título: `Certificar «contrato.pdf»`
  - Cuerpo: `Después de certificar, nadie podrá modificar el documento ni añadir firmas. Esta acción no se puede deshacer y sólo se puede hacer una vez.`
  - Botones: `Cancelar` (por defecto, responde a Enter) · `Certificar` (`danger`)
  - Apple: la acción destructiva **no** es el botón por defecto.

### Paso 5 — Estado posterior a la firma

Al terminar, el diálogo se cierra y el documento firmado se abre en la ventana.

- **Banner sobre el lienzo**, fondo `success-soft`, icono `✓` circular,
  desaparece al desplazarse:
  `Firmado y guardado como contrato-firmado.pdf`
  con dos acciones `ghost`: `Mostrar en la carpeta` · `Comprobar la firma`
- **Barra de estado permanente del documento** (24 px, bajo la barra superior,
  sólo si el PDF tiene firmas) — el equivalente al panel de firmas de Acrobat:
  `[✓ Firma válida]  Firmado por María Gómez Ruiz · 26 ago 2026 14:03 · sello de tiempo   [Ver detalles]`
- `Ver detalles` abre el diálogo de comprobación, que separa los tres hechos que
  Acrobat también separa, cada uno con su propia píldora:

  | Hecho | Texto afirmativo | Texto negativo |
  | --- | --- | --- |
  | Integridad | `El documento no ha cambiado desde que se firmó.` | `El documento se modificó después de firmarse.` |
  | Cobertura | `La firma cubre todo el documento.` | `La firma sólo cubre una versión anterior del documento.` |
  | Confianza | `El certificado procede de una autoridad reconocida.` | `No podemos comprobar la autoridad que emitió el certificado.` |

- **Nunca se resume la comprobación en una sola palabra verde.** Un documento
  íntegro firmado con un certificado desconocido no es «válido»: es
  `Validez desconocida`, y las tres líneas explican por qué.
- El original permanece intacto (regla 6 de `DESIGN.md`) y la ruta de ambos
  archivos aparece en el diálogo de detalles.

---

## E. Teclado y accesibilidad

### E.1 Mapa de atajos

Plataforma primaria Linux (`Ctrl`); en macOS se refleja con `Meta`. Las
convenciones universales no se reasignan (regla 7 de `DESIGN.md`; Apple: «no
reutilices atajos estándar para acciones propias»).

| Atajo | Acción | Origen |
| --- | --- | --- |
| `Ctrl+O` | Abrir PDF | Universal |
| `Ctrl+W` | Cerrar documento | Universal |
| `Ctrl+Q` | Salir | Universal |
| `Ctrl+P` | Imprimir | Universal |
| `Ctrl+F` | **Buscar en el documento** | Universal — intocable |
| `F3` / `Shift+F3` | Coincidencia siguiente / anterior | Universal |
| `Ctrl+C` | Copiar la selección | Universal |
| `Ctrl+Shift+C` | Copiar el texto de la página actual | Propio |
| `Ctrl+A` | Seleccionar todo el texto de la página | Universal |
| `Ctrl+K` | **Paleta de comandos** | Linear, Figma |
| `?` | Lista de atajos | Linear |
| `Ctrl+B` | Mostrar/ocultar panel de páginas | Propio |
| `Ctrl++` / `Ctrl+-` | Acercar / alejar | Acrobat |
| `Ctrl+0` | Ajustar al ancho | Acrobat |
| `Ctrl+1` | Zoom 100 % | Acrobat |
| `Ctrl+2` | Ajustar página completa | Acrobat |
| `Re Pág` / `Av Pág` | Página anterior / siguiente | Universal |
| `Inicio` / `Fin` | Primera / última página | Universal |
| `↑ ↓ ← →` | Desplazar el lienzo | Figma |
| `Ctrl+G` | Ir a página… | Acrobat |
| `Ctrl+Shift+S` | Firmar este PDF | Propio |
| `Ctrl+Shift+V` | Comprobar firmas | Propio |
| `Esc` | Salir del modo → cerrar diálogo → cerrar búsqueda | Universal |
| `Tab` / `Shift+Tab` | Grupo de foco siguiente / anterior | Apple |
| `F6` / `Shift+F6` | Zona siguiente / anterior (barra, panel, lienzo) | Convención de escritorio |
| `Enter` | Activar la acción principal del diálogo | Apple |
| `Espacio` | Activar el control enfocado | Universal |

**Dos conflictos que hay que corregir en `app.slint`:**

1. `Ctrl+K` está enlazado hoy a abrir la búsqueda, igual que `Ctrl+F`. Un atajo
   duplicado desperdicia el que todo el mundo asocia a la paleta de comandos.
   `Ctrl+K` pasa a abrir la paleta; la búsqueda se queda sólo en `Ctrl+F`.
2. `Ctrl+C` copia hoy la página entera cuando no hay selección. Copiar 3 000
   caracteres al pulsar el atajo de copiar sorprende. `Ctrl+C` copia la selección
   (y no hace nada sin selección); la página entera pasa a `Ctrl+Shift+C`.

Cuando `Ctrl+K` deja de ser un duplicado de la búsqueda, la paleta de comandos
debe listar **toda** acción del programa con su atajo al lado, incluidas las que
sólo existen dentro de un diálogo. Es el único mecanismo de descubrimiento que
escala sin llenar la barra de botones.

### E.2 Orden de foco

1. Orden de lectura: de arriba abajo y de inicio a fin (Apple).
2. Zonas de foco: **barra superior → panel de páginas → lienzo → barra
   flotante**. `Tab` recorre dentro de la zona; `F6` salta de zona.
3. En el lienzo, `Tab` no recorre las páginas una a una: el lienzo es un solo
   destino de foco y las flechas lo desplazan.
4. **Trampa de foco en modal:** al abrir un diálogo el foco va al primer campo
   editable (o al primer botón si no hay campos); `Tab` cicla dentro del diálogo;
   al cerrarlo, el foco vuelve exactamente al control que lo abrió.
5. El foco no se mueve solo (Apple). Un aviso que aparece no roba el foco.
6. Un modo del lienzo no cambia el foco: si se entró desde el diálogo, al salir
   se vuelve al mismo control del diálogo.

### E.3 Tratamiento del foco visible

- Anillo de **2 px**, color `focus-ring`, dibujado hacia dentro, con el radio del
  control. Cumple SC 2.4.13 (perímetro de 2 px) y SC 1.4.11 (3:1): medido, 9.08:1
  en claro y 8.34:1 en oscuro.
- Sobre relleno de acento o de peligro el anillo usa `text-on-accent` (ver la
  corrección 4 de A.7). Nunca `focus-ring` sobre acento: 1.52:1.
- **`focus-visible`:** el anillo se muestra sólo tras interacción de teclado.
  Slint no distingue el origen del foco, así que se implementa con un token nuevo
  `Theme.keyboard-nav: bool`: se pone a `true` en el `key-pressed` del
  `FocusScope` raíz y a `false` en el primer `pointer-event` de tipo `down`. Los
  componentes condicionan el borde a `focus.has-focus && Theme.keyboard-nav`.
- El anillo nunca queda recortado por el contenedor: los `ScrollView` reservan
  2 px de padding interior.

### E.4 Nombres accesibles

Slint expone `accessible-role`, `accessible-label`, `accessible-description`,
`accessible-value`, `accessible-checked`, `accessible-enabled` y
`accessible-placeholder-text`; `accessible-role` es obligatorio para que el resto
tenga efecto.

| Elemento | Reglas |
| --- | --- |
| Acción de sólo icono | `accessible-role: button` + `accessible-label` con el verbo completo: `"Acercar"`, no `"+"` |
| Botón con texto | La etiqueta accesible es el texto; si el texto se acorta en `compact`, la etiqueta **no** se acorta |
| Campo | `accessible-role: text-input`, `accessible-label` = etiqueta visible, `accessible-placeholder-text` = marcador |
| Fila seleccionable | `accessible-role: button`, `accessible-checked` según selección, etiqueta `"<título>. <detalle>"` |
| Página del PDF | `accessible-role: image`, etiqueta `"Página 3 de 12"`; `accessible-description` con el texto extraído si existe |
| Píldora de estado | `accessible-role: text` con la frase completa: `"Firma válida. El documento no ha cambiado desde que se firmó."` |
| Progreso | `accessible-role: progress-indicator` + `accessible-label` que diga qué se está haciendo |
| Velo del modal | Los controles de detrás quedan `accessible-enabled: false` mientras el modal está abierto |

### E.5 Expectativas de lector de pantalla

1. **Todo estado que se anuncia con color se anuncia también con texto.** La
   píldora ya lleva la palabra; el lector lee la palabra.
2. **Avisos:** Slint no tiene región dinámica («live region»). Mientras no la
   tenga, ningún error se comunica **sólo** con un aviso temporal: el error vive
   además en el diálogo o en la barra de estado del documento, que sí son
   navegables con el foco. Es una limitación real de la plataforma, no una
   decisión de diseño.
3. **Operaciones largas:** al empezar a firmar, el foco pasa al indicador de
   progreso con `accessible-label: "Firmando el documento"`, y al terminar al
   banner de resultado. Es la única excepción a la regla «el foco no se mueve
   solo», y existe porque el resultado es justo lo que la persona espera.
4. **Tamaños mínimos** (Apple, macOS): control por defecto 28×28, mínimo absoluto
   20×20. Aquí: 32×32 para acciones de icono, 36 de alto para botones, 30 para
   campos. La píldora de 24 px no es interactiva; si alguna vez lo fuera, su área
   de pulsación sube a 32.
5. **Movimiento reducido:** ver A.6. Con `Theme.reduced-motion` activo, el velo y
   los diálogos aparecen sin transición; nada parpadea.
6. **Prueba de aceptación** (regla 9 de `DESIGN.md`): recorrer el flujo completo
   de firma sin tocar el ratón y con la captura en escala de grises. Si en algún
   paso no se sabe dónde está el foco o qué estado tiene una firma, el paso está
   mal.

---

## F. Fuentes

Todas las páginas de esta lista se descargaron y leyeron para escribir el
documento. Las cifras de contraste **no** vienen de ninguna de ellas: están
calculadas sobre los hexadecimales de `native/ui/theme.slint` con la fórmula de
luminancia relativa de WCAG, componiendo antes los colores con alfa sobre su
fondo real.

**Apple — Human Interface Guidelines.** Se leyó la versión JSON de la
documentación (`/tutorials/data/design/...json`) porque las páginas HTML se
sirven vacías a un cliente sin JavaScript.

| URL | Qué se tomó |
| --- | --- |
| https://developer.apple.com/tutorials/data/design/human-interface-guidelines/typography.json | Escala macOS completa con tamaño y altura de línea (26/32, 22/26, 17/22, 15/20, 13/16, 12/15, 11/14), cuerpo por defecto 13 pt, mínimo 10 pt, prohibición de pesos Light |
| https://developer.apple.com/tutorials/data/design/human-interface-guidelines/accessibility.json | Control macOS 28×28 por defecto y 20×20 mínimo; 12 pt de separación alrededor de elementos con bisel y 24 pt sin bisel; ratios 4.5:1 y 3:1; guía de movimiento reducido |
| https://developer.apple.com/tutorials/data/design/human-interface-guidelines/buttons.json | Una o dos acciones prominentes por vista; no asignar el rol primario a acciones destructivas; puntos suspensivos sólo si abre otra ventana; diferenciar por estilo, no por tamaño |
| https://developer.apple.com/tutorials/data/design/human-interface-guidelines/toolbars.json | Grupo inicial no personalizable, grupo final siempre visible, máximo tres grupos, menú de desbordamiento, título breve |
| https://developer.apple.com/tutorials/data/design/human-interface-guidelines/sidebars.json | Ocultar y revelar el panel al redimensionar; máximo dos niveles; nada crítico al fondo del panel |
| https://developer.apple.com/tutorials/data/design/human-interface-guidelines/sheets.json | Un solo modal a la vez; siempre una salida además del botón de confirmación; tamaño por defecto razonable |
| https://developer.apple.com/tutorials/data/design/human-interface-guidelines/popovers.json | Cuándo un popover y cuándo una hoja; cierre al pulsar fuera; guardar siempre el trabajo al cerrar |
| https://developer.apple.com/tutorials/data/design/human-interface-guidelines/alerts.json | Máximo tres botones; Cancelar al principio de la fila y botón por defecto al final; no hacer destructivo el botón por defecto; etiquetas de una o dos palabras con verbo |
| https://developer.apple.com/tutorials/data/design/human-interface-guidelines/keyboards.json | Atajos reservados del sistema; «no reutilices atajos estándar»; navegación por zonas con Control-F5/F6 |
| https://developer.apple.com/tutorials/data/design/human-interface-guidelines/focus-and-selection.json | Tab entre grupos de foco en orden de lectura y flechas dentro del grupo; no mover el foco sin interacción |
| https://developer.apple.com/tutorials/data/design/human-interface-guidelines/color.json | No usar el color como único canal; definir variantes clara, oscura y de contraste aumentado |
| https://developer.apple.com/tutorials/data/design/human-interface-guidelines/text-fields.json | Etiqueta además del marcador de posición; validar al cambiar de campo; anchos coherentes con el contenido esperado |
| https://developer.apple.com/tutorials/data/design/human-interface-guidelines/windows.json | Estados de ventana; no poner información crítica en la barra inferior; fijar mínimos propios |
| https://developer.apple.com/tutorials/data/design/human-interface-guidelines/layout.json | Se consultó por métricas de macOS y **no las publica**: las cifras de esa página son de tvOS, visionOS y watchOS. Por eso los márgenes de B.5 salen de nuestra escala y no de Apple |

**Linear**

| URL | Qué se tomó |
| --- | --- |
| https://linear.app/now/how-we-redesigned-the-linear-ui | Superficies por elevación (fondo, primer plano, paneles, diálogos, modales); reducción de 98 variables de tema a 3 (base, acento, contraste); texto más oscuro en claro y más claro en oscuro; menos ruido visual mediante alineación en vez de cajas |
| https://linear.app/blog/how-we-redesigned-the-linear-ui | Espacio LCH por uniformidad perceptual; temas de contraste alto generados automáticamente; tipografía distinta para titulares y cuerpo |
| https://linear.app/changelog/2021-03-25-keyboard-shortcuts-help | `?` abre la lista de atajos, buscable — adoptado tal cual en E.1 |

**Stripe**

| URL | Qué se tomó |
| --- | --- |
| https://stripe.com/blog/accessible-color-systems | Objetivos 4.5:1 para texto pequeño y 3:1 para iconos y texto grande; paleta generada en espacio perceptual (CIELAB) para que el contraste sea predecible por niveles |
| https://stripe.com/resources/more/checkout-ui-strategies-for-faster-and-more-intuitive-transactions | «Cada paso termina con un único movimiento siguiente evidente»; acciones secundarias atenuadas; errores en línea junto al campo con texto específico; indicar en qué paso se está |
| https://stripe.com/resources/more/credit-card-checkout-ui-design | Etiquetas de acción concretas («Pay now», «Place order»); validar al completar el campo, no en cada pulsación; no borrar lo que la persona escribió; omitir todo campo no imprescindible |

**Figma**

| URL | Qué se tomó |
| --- | --- |
| https://help.figma.com/hc/en-us/articles/360041065034-Adjust-your-zoom-and-view-options | El porcentaje de zoom es un control que abre un menú con valores preestablecidos, no una etiqueta; atajos de ajustar a la ventana y a la selección |
| https://help.figma.com/hc/en-us/articles/360040328653-Use-Figma-products-with-a-keyboard | Las flechas desplazan el lienzo cuando no hay nada seleccionado; los controles de teclado no se pueden desactivar; acceso a la barra de herramientas con F6 |

**W3C — WCAG 2.2**

| URL | Qué se tomó |
| --- | --- |
| https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html | 4.5:1 para texto normal; 3:1 para texto ≥ 18 pt o ≥ 14 pt en negrita |
| https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html | 3:1 para lo necesario para identificar un control y su estado; exención de controles inactivos y de lo decorativo — la base de A.4 |
| https://www.w3.org/WAI/WCAG22/Understanding/focus-appearance.html | Perímetro de 2 px y 3:1 entre estado enfocado y no enfocado — la base de E.3 |

**Adobe Acrobat**

| URL | Qué se tomó |
| --- | --- |
| https://www.globalsign.com/en/blog/certifying-vs-approval-signatures-in-adobe | Las tres opciones de permisos al certificar y el hecho de que la certificación sea única y previa a cualquier firma de aprobación |
| https://community.adobe.com/t5/acrobat-discussions/what-is-the-difference-between-quot-digitally-sign-quot-and-quot-certify-visible-signature-quot/td-p/11358188 | Distinción «Digitally Sign» / «Certify (Visible Signature)» y el bloqueo del documento tras certificar |
| https://acrobatusers.com/tutorials/how-to-sign-using-a-certificate/ | Orden real de la interfaz: arrastrar el rectángulo, elegir apariencia, contraseña, «Sign», guardar — el modelo «colocar y luego confirmar» de D.2 |
| https://knowledge.digicert.com/tutorials/how-to-sign-a-document-in-adobe-acrobat | Nombres de los diálogos en secuencia: «Use a certificate» → «Digitally sign» → «Sign with Digital ID» → apariencia → bloqueo → guardar |

> **Fuentes que no se pudieron leer.** Las páginas de ayuda oficiales de Adobe
> (`helpx.adobe.com/acrobat/desktop/e-sign-documents/…`, incluida la versión en
> español y otras localizaciones) agotaron el tiempo de espera en todos los
> intentos. El vocabulario de Acrobat de la sección D procede por tanto de las
> cuatro fuentes de arriba, que sí se leyeron, y no de la documentación original.
> **La redacción en español de la sección D es nuestra, no la terminología
> literal de Acrobat en español**: antes de congelarla conviene abrir Acrobat en
> español y contrastar «ID digital», «Firmar digitalmente» y «Certificar (firma
> visible)».

**Slint** (para que todo lo anterior sea implementable)

| URL | Qué se tomó |
| --- | --- |
| https://releases.slint.dev/1.7.0/docs/slint/src/language/syntax/animations | Sintaxis `animate`, `duration`, `delay`, `iteration-count` y la lista completa de curvas — la base de A.6 |
| https://docs.slint.dev/latest/docs/slint/reference/elements/text/ | El elemento `Text` **no tiene** `line-height`: por eso A.1 expresa la altura de línea como alto de contenedor o `spacing` de layout |
| https://releases.slint.dev/1.7.0/docs/slint/src/language/builtins/elements | Propiedades `accessible-*` disponibles — la base de E.4 |

---

## Apéndice — Dónde se aplica cada sección

| Sección | Archivo | Naturaleza del cambio |
| --- | --- | --- |
| A.1, A.2, A.3, A.4, A.6, A.7 | `native/ui/theme.slint` | Tokens nuevos y 4 valores corregidos |
| A.5 | `native/ui/theme.slint` + `components.slint` | Global `Elevation` nuevo |
| B.1, B.2 | `native/ui/app.slint` | `min-width`, `min-height`, puntos de ruptura |
| B.3 | `native/ui/app.slint` | Barra a 48 px, título centrado sobre la ventana, hairline sólo inferior |
| B.4 | `native/ui/app.slint` | Panel de páginas (no existe todavía) |
| B.5, B.6, B.7 | `native/ui/app.slint` | Márgenes del lienzo, barra flotante, barras de modo |
| C.1 – C.8 | `native/ui/components.slint` | Estados de deshabilitado, borde de control, marca de selección |
| D | `native/ui/app.slint` + `native/src/presentation.rs` | Microcopia y orden de pasos |
| E.1 | `native/ui/app.slint` | `KeyBinding`: `Ctrl+K`, `Ctrl+C`, `Ctrl+Shift+C`, `F3`, `?` |
| E.3 | `native/ui/theme.slint` + `components.slint` | `Theme.keyboard-nav` |
| E.4 | Todos los `.slint` | `accessible-*` que faltan |
