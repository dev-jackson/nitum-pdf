# Especificación de interfaz de Nitum PDF

Escrita el 27 de agosto de 2026 contra el estado real del código, no sobre un
lienzo en blanco. Los valores viven en `native/ui/theme.slint` y los componentes
en `native/ui/components.slint`; este documento explica **por qué** cada número
es ese y **cómo se comprueba**.

La regla de oro sigue siendo la de `DESIGN.md`:

> ¿Una persona que viene de Acrobat entiende qué puede hacer, qué ocurrirá y
> cuál es el siguiente paso sin conocer certificados?

## Nota sobre las fuentes

Las Human Interface Guidelines de Apple y la documentación de diseño de Figma
son aplicaciones JavaScript: al descargarlas sólo se obtiene el armazón HTML,
sin el texto. **No se han podido citar directamente y por tanto no se citan.**
Lo que sí se ha podido verificar está en la sección F, y todo lo demás de este
documento se justifica con mediciones sobre nuestras propias capturas.

---

## A. Tokens

### Escala tipográfica

Seis tamaños. Antes había trece, que es lo mismo que no tener ninguno.

| Token | Tamaño | Uso |
| --- | --- | --- |
| `Type.caption` | 11 px | Metadatos que se pueden ignorar sin perder nada |
| `Type.small` | 12 px | Texto de apoyo, ayuda de un campo, acción de una fila. **Mínimo para texto que aporta algo** |
| `Type.body` | 13 px | Texto normal, etiquetas de botón, títulos de fila |
| `Type.emphasis` | 14 px | Texto principal de un estado vacío |
| `Type.title` | 17 px | Título de diálogo |
| `Type.display` | 22 px | Único titular de una pantalla |

Pesos: `regular` 400, `medium` 500, `semibold` 600, `bold` 700. **La jerarquía
la lleva el peso, no el tamaño**: entre un título de fila y su descripción hay
un salto de un punto de tamaño (13 → 12) y dos de peso (600 → 400).

Slint 1.17 **no tiene `line-height`**, comprobado contra el compilador. De ahí
dos reglas prácticas:

- Si un texto de apoyo necesita dos líneas, se reescribe más corto. Es la razón
  de que los subtítulos de los diálogos sean tan breves.
- Todo texto de varias líneas lleva `min-height` explícito, porque sin suelo el
  layout lo comprime y le corta los ascendentes.

**A 11 px el renderizador corta los ascendentes y los acentos**, verificado
ampliando las capturas a 3×: en «firmar» se perdían la `f` y la `t`, y en
«ningún» la tilde. Por eso `caption` queda reservado a metadatos prescindibles y
el texto de ayuda usa `small` (12 px). El mismo texto a 17 px sale íntegro, así
que no es un defecto del renderizador de pruebas sino del tamaño.

### Espaciado

Escala de 4: `xs` 4, `sm` 8, `md` 12, `lg` 16, `xl` 24, `xxl` 32, `section` 40.
Antes convivían dieciséis valores, incluidos 1, 3, 7, 18, 22, 54 px.

Cómo se usa:

- `xs` entre filas de un mismo grupo (una lista de opciones es **un** elemento).
- `md` entre un control y su icono.
- `lg` entre secciones distintas dentro de un diálogo.
- `xl` como sangría del contenido de un diálogo.

### Radios

Tres, más la píldora: `sm` 6, `md` 10, `lg` 14, `pill` 999. Antes trece.

### Movimiento

`instant` 90 ms para realimentación de puntero, `quick` 140 ms para desplegar,
`calm` 220 ms para cambios de contexto.

**Un elemento que se reconstruye no se anima.** Al recrearse el subárbol de un
diálogo, una animación de fondo sigue pintando el color de la instancia
anterior, y el resultado fue que la pantalla de certificación resaltaba
«Firmar para aprobar» cuando la opción elegida era «Certificar el documento».
Está comentado en `components.slint` para que nadie lo reintroduzca.

### Color

Dos paletas completas. **Ningún archivo salvo `theme.slint` nombra un color**,
sombras incluidas.

Contraste medido, no estimado, con la fórmula de luminancia relativa de WCAG 2.2
y aplanando los colores translúcidos sobre su fondo real. `native/tests/theme_contrast.rs`
comprueba 40 pares en cada compilación y falla si alguno baja del mínimo.

Umbrales: 4.5:1 para texto (SC 1.4.3) y 3:1 para el límite visual de un control
y el indicador de foco (SC 1.4.11).

| Par | Claro | Oscuro | Mínimo |
| --- | --- | --- | --- |
| `text` / `surface` | 17.92:1 | 16.25:1 | 4.5 |
| `text-secondary` / `surface` | 6.28:1 | 8.93:1 | 4.5 |
| `text-tertiary` / `surface` | 4.83:1 | 5.70:1 | 4.5 |
| `text` / `raised` | 16.87:1 | 13.23:1 | 4.5 |
| `accent` / `surface` | 5.48:1 | 7.00:1 | 4.5 |
| `success` / `surface` | 5.38:1 | 8.72:1 | 4.5 |
| `warning` / `surface` | 6.21:1 | 9.34:1 | 4.5 |
| `danger` / `surface` | 5.77:1 | 6.27:1 | 4.5 |
| `text-on-accent` / `accent` | 5.48:1 | 7.51:1 | 4.5 |
| `accent` / `accent-soft` | 4.90:1 | 5.74:1 | 4.5 |
| `border-strong` / `surface` | 3.90:1 | 4.90:1 | 3.0 |
| `focus-ring` / `surface` | 9.08:1 | 8.34:1 | 3.0 |

Cuatro decisiones que salieron de medir:

1. **En tema oscuro la tinta sobre el acento es oscura, no blanca.** Un azul lo
   bastante claro para leerse como acento sobre fondo oscuro no puede sostener
   texto blanco: daba 3.16:1. Con tinta `#0d1220` da 7.51:1.
2. **`raised` ya no es igual que `surface`.** En claro eran los dos blancos, así
   que un botón secundario sólo se distinguía por una línea de 1.28:1 y parecía
   deshabilitado.
3. **`success` y `warning` en claro se oscurecieron** (`#0f8f5b` → `#0a7a4c`,
   `#a86800` → `#8a5500`): no llegaban a 4.5:1 sobre blanco.
4. **El rojo de marca no es el acento de la interfaz.** El rojo identifica al
   icono y señala peligro; si además fuera el color de las acciones normales, un
   borrado no se distinguiría de un guardado.

---

## B. Rejilla

| Medida | Valor | Por qué |
| --- | --- | --- |
| Alto de barra superior | 52 px | Controles de 36 px con `sm` arriba y abajo |
| Alto de control | 36 px | Objetivo de puntero cómodo sin robar altura al documento |
| Columna de contenido | 24 px de sangría | Verificado: cabecera, cuerpo y filas empiezan en la misma x |
| Ancho de diálogo | 460 / 520 / 560 / 620 px | Según cuánto contiene, no según una constante |
| Alto de diálogo | el de su contenido, tope: ventana − 48 px | Antes era fijo y sobraban 190 px o se cortaba |
| Ancho de página | ventana − 80 px, entre 320 y 1400 | Zoom 100 % **es** ajustar al ancho |
| Hueco bajo el documento | 76 px (44 + 2 × `lg`) | La barra flotante nunca tapa un renglón |
| Umbral compacto | 900 px | Debajo, el título cede y las acciones pasan a iconos |

Alineación comprobada midiendo las capturas, no a ojo: en el centro de firma la
cabecera y las cuatro filas terminan todas en x=845 y empiezan en x=336.

---

## C. Componentes

### `ActionButton`

Alto 36, radio `md`, sangría horizontal `md`, hueco icono-texto `sm`, ancho
mínimo 96.

| Variante | Fondo | Borde | Uso |
| --- | --- | --- | --- |
| `primary` | `accent` | ninguno | **Una por pantalla** |
| `secondary` | `raised` | `border` | Acciones normales |
| `ghost` | transparente | ninguno | Acciones de barra |
| `danger` | `danger` | ninguno | Sólo lo irreversible |

Estados: reposo, `hover` (fondo `accent-hover` o `accent-soft`), `pressed`
(`accent-pressed`), `focus` (borde 2 px en `focus-ring`), `disabled` (superficie
`raised` con tinta `text-tertiary` y borde, nunca un acento desvaído: un botón
difuminado se lee como un fallo de dibujo, no como un botón inactivo).

No hay animación de fondo, por la misma razón que en `ChoiceRow`.

**Los botones vecinos comparten variante.** Un `ghost` termina donde acaba su
etiqueta y un `secondary` donde acaba su marco: mezclados, sus bordes derechos
no coinciden aunque el código diga que sí.

### `IconAction`

36 × 36, glifo de 20 px centrado. Todo icono lleva `accessible-label`.

### `ChoiceRow`

Alto 56 con descripción, 44 sin ella. Dos modos:

- **acción** (`option: false`): icono a la izquierda, verbo a la derecha.
- **opción** (`option: true`): indicador redondo de 18 px, como un radio.

Los dos usan la misma columna de 24 px, así que todos los títulos de un diálogo
empiezan en la misma x independientemente del modo.

La propiedad se llama `is-selected` y no `selected` **a propósito**: `selected`
choca con un nombre que Slint ya resuelve sobre el elemento, y el resultado era
que el punto de llamada y el componente leían propiedades distintas.

### `Dialog`

Cabecera fija, cuerpo desplazable, pie fijo. Una acción principal a la derecha,
la secundaria y «Atrás» a la izquierda. Cierre siempre disponible con Esc y con
el aspa.

### `StatusPill` y `Note`

Icono + palabra + tinte, en ese orden. **El color nunca va solo** (regla 8 de
`DESIGN.md`): quien no distingue rojo de verde lee el icono y la palabra.

---

## D. Flujo de firma

Cinco pasos, cada uno con una sola decisión.

**1. ¿Qué necesitas hacer?** — «Firmar con mi identidad», «Comprobar las firmas
del PDF», «Mis firmas visuales», «Conectar tarjeta o DNI».
Subtítulo: «Antes de cambiar el PDF te explicamos qué va a ocurrir.»

**2. Identidad** — nombre, y debajo dónde vive la clave: «La clave privada
permanece en este equipo» o «La clave privada nunca sale de la tarjeta».

**3. Contraseña** — ayuda: «Se usa una sola vez para firmar y no se guarda en
ningún sitio.»

**4. Tipo de firma** — la decisión irreversible, en palabras y no en siglas:

- «Firmar para aprobar» / «Otras personas podrán firmar después de ti»
- «Certificar el documento» / «Debe ser la primera firma y fija qué cambios se
  permiten»

Si certifica, aparece «Cambios que seguirán permitidos»: Ninguno / Rellenar
formularios / Formularios y anotaciones.

**5. Firma visible** — «Mostrar la firma sobre la página», con la advertencia
que evita el malentendido de fondo: «La validez la aporta el certificado; lo
visible sólo ayuda a reconocerla.»

**Avanzadas** — el nivel PAdES vive detrás de «Opciones avanzadas» y se describe
por lo que hace, con la sigla como apoyo: «Básica (PAdES B-B)», «Con sello de
tiempo (B-T)», «Validación duradera (B-LT)», «Archivo (B-LTA)». Antes eran
cuatro botones «B-B / B-T / B-LT / B-LTA» en el camino principal.

Acción final: **«Firmar y guardar»**, nunca «Aceptar». Consecuencia declarada
antes de pulsarla: «Se creará un PDF nuevo. El original no se modifica.»

---

## E. Teclado y accesibilidad

| Atajo | Acción |
| --- | --- |
| `Ctrl/Cmd + O` | Abrir |
| `Ctrl/Cmd + F` | Buscar — **nunca se reasigna** |
| `Ctrl/Cmd + K` | Buscar |
| `Ctrl/Cmd + Shift + S` | Centro de firma |
| `Ctrl/Cmd + 0` | Ajustar al ancho |
| `Ctrl/Cmd + C` | Copiar la página |
| `Esc` | Salir del modo activo, luego cerrar el diálogo |

Reglas:

- Todo control es alcanzable con Tab y se activa con Espacio o Intro.
- El foco se dibuja con un borde de 2 px en `focus-ring`, medido a 8.34:1 y
  9.08:1 sobre sus fondos. Es un borde y no un anillo exterior porque en Slint
  un hijo posicionado no puede leer `parent` desde la raíz de un componente.
- Todo control tiene `accessible-label`; `visual_shell.rs` recorre los roles
  botón, casilla y campo de texto en cada pantalla y falla si alguno está vacío.
- El estado nunca depende sólo del color.

---

## F. Fuentes

Sólo se listan páginas efectivamente descargadas y leídas.

- <https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html> — de aquí
  salen el 4.5:1 de texto normal y la definición de texto grande (18 pt, o 14 pt
  en negrita). Nuestro texto más pequeño es de 11 px, muy por debajo de «grande»,
  así que el umbral que aplica siempre es 4.5:1.
- <https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html> — el 3:1
  para el límite visual de un control y para el indicador de foco, y el aviso de
  que el valor calculado no se redondea: 2.999:1 no cumple. Por eso el test
  compara sin redondeo.
- <https://linear.app/now/how-we-redesigned-the-linear-ui> — Linear cuenta que
  subieron el contraste «oscureciendo el texto y los iconos neutros en claro y
  aclarándolos en oscuro», que su objetivo fue «reducir el ruido visual,
  mantener la alineación y aumentar la jerarquía», y que dedicaron tiempo a
  «alinear etiquetas, iconos y botones, vertical y horizontalmente», algo que
  «se siente después de unos minutos de uso». Es exactamente la clase de trabajo
  que aquí se hizo midiendo capturas.
- Apple HIG (Layout, Typography) y la documentación de diseño de Figma: **no
  accesibles**, son SPA de JavaScript y devuelven la página vacía. No se citan.

---

## Cómo se comprueba

```sh
cd native
cargo test --locked --test theme_contrast     # 40 pares contra WCAG 2.2
NITUM_VISUAL_OUTPUT=../visual-audit \
  cargo test --locked --test visual_shell     # 20 pantallas + etiquetas accesibles
```

Un cambio de color que rompa el contraste hace fallar la compilación de tests, no
la revisión de alguien.
