# Auditoría visual de Nitum PDF

Fecha: 26 de agosto de 2026. Base: las 20 capturas de `visual-audit/`, contrastadas
con `native/ui/app.slint` (832 líneas) y `native/src/presentation.rs` (1414 líneas).
Cada hallazgo cita lo que se ve en una imagen concreta y el punto exacto del código
que lo produce. Las medidas en píxeles son lecturas sobre las capturas
(1180×820 en las vistas normales, 720×560 en las compactas).

Regla de severidad:

- **blocker**: rompe una regla de `DESIGN.md`, corta contenido o hace que un
  control principal se lea como roto.
- **major**: se percibe de inmediato como falta de acabado.
- **minor**: detalle de refinamiento.

---

## 1. `shell-light.png` / `shell-dark.png` — pantalla inicial sin documento

**Qué es.** Ventana vacía con la tarjeta de bienvenida y la cabecera completa.

**Defectos visibles.**

1. **El título de la cabecera no está centrado.** El bloque «Nitum PDF» tiene su
   centro en x≈435 de una ventana de 1180 px; el centro real es 590. Está
   desplazado ~155 px a la izquierda y se lee como un error de maquetación, no
   como una decisión. La causa es exacta: hay dos espaciadores de igual peso
   (`app.slint:316` y `app.slint:331`) pero el grupo izquierdo mide 68 px
   («Abrir») y el derecho ~372 px (tres `HeaderButton` de 68 px + el primario de
   144 px + espaciados). El desfase teórico (372−68)/2 = 152 px coincide con lo
   medido. Severidad: **blocker** (aparece en las 14 capturas con cabecera).
   Código: `native/ui/app.slint:313-336`.
2. **La fila de la cabecera se pega al borde superior.** Los controles ocupan
   y≈1–41 dentro de una barra de 64 px: quedan ~23 px muertos debajo y ninguno
   arriba. La barra parece descolgada. Código: `native/ui/app.slint:313-314`
   (`HorizontalLayout { alignment: center }` sólo centra en el eje horizontal).
   Severidad: **major**.
3. **La cabecera dibuja borde en los cuatro lados.** En `shell-light.png` se ve
   la línea vertical en x=0 y en x=1179, además de la superior. Un `Rectangle`
   con `border-width: 1px` no puede tener sólo borde inferior.
   Código: `native/ui/app.slint:312`. Severidad: **minor**.
4. **«Firmar este PDF» deshabilitado es casi invisible en claro.** `opacity: 0.45`
   sobre azul #2563eb da un botón lavanda pálido sobre blanco; parece un error de
   render más que un botón inactivo. Código: `native/ui/app.slint:71`, usado en
   `:335`. Severidad: **major**.
5. **La tarjeta de bienvenida desperdicia su mitad inferior.** Mide 680×390 px,
   el contenido termina en y≈535 y la tarjeta llega a y≈615: ~80 px vacíos
   abajo y ~85 arriba, sin ser simétricos. Código: `native/ui/app.slint:342`
   (altura fija 390 px) + `:347-364`. Severidad: **major**.
6. **El icono heroico es un «✓» de 42 px.** Un tick no significa «abre un PDF»;
   además es un glifo de texto, no un icono, y se ve desalineado ópticamente
   dentro de su cuadro de 88 px. Código: `native/ui/app.slint:353`.
   Severidad: **major**.
7. **Las etiquetas de la cabecera cambian de significado según el ancho.**
   «Abrir» → «+ PDF», «Oscuro/Claro» → «Tema», «Versión» → «Actualizar»/«Info».
   El mismo control no se llama igual en dos tamaños de ventana.
   Código: `native/ui/app.slint:315, 333, 334`. Severidad: **major**.

**Reglas rotas.** Regla 2 («una acción principal por paso»): en la pantalla vacía
compiten el primario «Abrir un PDF» de la tarjeta y «Abrir» en la cabecera, con
un tercer primario deshabilitado («Firmar este PDF») que no lleva a ningún sitio.
Referente Apple («alineación consistente»): incumplido por los puntos 1 y 2.

---

## 2. `continuous-document-dark.png` — documento abierto, vista continua

**Qué es.** PDF de dos páginas en desplazamiento continuo con la barra flotante.

**Defectos visibles.**

1. **La barra de herramientas tapa el documento.** La barra opaca de 680×52 px
   se posa sobre la página; en la captura oculta cuatro renglones del PDF. La
   lista de páginas no reserva ningún hueco inferior. Código: barra en
   `native/ui/app.slint:464-467`; altura de ítem sin margen inferior en
   `:370`. **Rompe la Regla 1** («el PDF es la superficie principal»).
   Severidad: **blocker**.
2. **La página no se ajusta al ancho útil.** La página mide 760 px en una
   ventana de 1180: 210 px de canal vacío a cada lado, el 36 % de la ventana.
   El ancho es una constante, no una medida de la ventana:
   `native/ui/app.slint:374` (`760px * zoom / 100`) y
   `native/src/presentation.rs:412` (`set_zoom_percent(100)` al abrir).
   **Rompe la Regla 1** («abre ajustado al ancho útil») de forma literal.
   Severidad: **blocker**.
3. **«Ajustar» no ajusta nada.** `fit_width()` en
   `native/src/application.rs:146` sólo hace `zoom_percent = 100`; no consulta el
   ancho de la ventana. El botón existe, se ve pulsable y no hace lo que promete.
   Código del botón: `native/ui/app.slint:479`. Severidad: **blocker**.
4. **Glifos ASCII como iconos en la barra.** `<`, `>`, `-`, `+` en
   `native/ui/app.slint:470, 474, 476, 477`. El `-` de alejar es una raya de
   ~10 px que ópticamente flota alta dentro de un botón de 40 px; el `+` y el `-`
   no comparten peso ni tamaño con las flechas. Severidad: **blocker** para el
   acabado.
5. **La barra mezcla cuatro gramáticas de control en 680 px.** Botones-icono de
   40 px, una etiqueta suelta («Página»), una píldora no editable con el número,
   otra etiqueta («de 2»), un separador de 1 px, dos botones-icono, un texto en
   negrita («100 %») y tres botones de texto sin borde. Los tres últimos
   («Ajustar», «Seleccionar», «Copiar») no tienen fondo ni contorno en reposo:
   se leen como etiquetas, no como controles. Código: `native/ui/app.slint:468-482`.
   Severidad: **major**.
6. **La píldora del número de página usa `Palette.line` como relleno.** Ese token
   es un blanco al 9 % pensado para bordes; como fondo produce un recuadro casi
   invisible alrededor del «1». Código: `native/ui/app.slint:472`, token en `:19`.
   Severidad: **major**.
7. **El subtítulo «Documento local» es de 11 px en color `muted`.** Es el texto
   más pequeño de la aplicación y está bajo un título en negrita; se lee como
   suciedad. Código: `native/ui/app.slint:320`. Severidad: **minor**.
8. **La barra de desplazamiento aparece pegada al borde derecho** como una raya
   clara de ~4 px sin margen (x≈1176). Es el `ListView` estándar sin estilizar.
   Código: `native/ui/app.slint:367`. Severidad: **minor**.

---

## 3. `document-compact-dark.png` — documento en ventana compacta (720×560)

**Qué es.** El mismo documento en el ancho mínimo soportado.

**Defectos visibles.**

1. **La página se sale de la ventana por los dos lados.** Los renglones del PDF
   abarcan x≈60–660, es decir, un margen interno de 80 px sobre una página de
   760 px: la página real ocupa de x≈−20 a x≈740 en una ventana de 720. Los
   márgenes izquierdo y derecho del documento están cortados. Es la consecuencia
   directa del ancho fijo de `native/ui/app.slint:374`.
   **Rompe la Regla 1.** Severidad: **blocker**.
2. **La barra flotante no se adapta.** Sigue midiendo 680 px dentro de 720:
   quedan 20 px de margen, «Copiar» termina a 21 px del borde y el conjunto se
   ve apretado contra los bordes. Ancho fijo en `native/ui/app.slint:466`.
   Severidad: **major**.
3. **«+ PDF» usa un `+` ASCII dentro de una etiqueta de texto**, mezclando icono
   y palabra en un control que en modo ancho es sólo palabra.
   Código: `native/ui/app.slint:315`. Severidad: **major**.
4. **Franja negra entre cabecera y página.** El `canvas` (#111318) queda visible
   como una banda de ~20 px bajo la barra antes de la página blanca, sin ser un
   margen intencionado ni coincidir con el margen lateral. Severidad: **minor**.

---

## 4. `search-compact-dark.png` — búsqueda abierta

**Qué es.** Cabecera con el campo de búsqueda desplegado en ventana compacta.

**Defectos visibles.**

1. **El campo de búsqueda ocupa toda la altura de la cabecera y es gris claro en
   tema oscuro.** Es un rectángulo de 180×64 px que toca el borde superior y el
   inferior de la barra, rompe el ritmo de 40 px del resto de controles y su
   fondo claro no pertenece a la paleta oscura. Es el elemento más llamativo de
   la pantalla y es el que peor se ve. Código: `native/ui/app.slint:324-328`
   (`LineEdit` de `std-widgets` sin altura ni estilo propios, dentro de un
   `HorizontalLayout` que lo estira en vertical). Severidad: **blocker**.
2. **No hay icono de lupa ni contador de resultados visible.** El
   `search-status` (`native/ui/app.slint:329`) se renderiza sólo cuando hay
   texto y en 12 px `muted`; en la captura no aparece nada donde debería estar el
   estado. Severidad: **major**.
3. **Los botones de la derecha se desplazan al abrir la búsqueda.** «Buscar» se
   convierte en «Cerrar» y todo el grupo se mueve; la posición de «Tema», «Info»
   y «Firmar» cambia respecto a `document-compact-dark.png`. Código:
   `native/ui/app.slint:322-332`. Severidad: **major**.
4. **`Ctrl+F` sí funciona** (`native/ui/app.slint:279`), así que la Regla 7 se
   cumple; el problema es la presentación, no el atajo.

---

## 5. `text-selection-compact-dark.png` — modo selección de texto

**Qué es.** Modo «Seleccionar» activo sobre la página.

**Defectos visibles.**

1. **Dos instrucciones distintas para el mismo modo, simultáneas en pantalla.**
   Arriba de la página: «Arrastra sobre el texto que quieres copiar»
   (`native/ui/app.slint:411`). Abajo, en la barra: «Selecciona una zona para
   copiar su texto» (`:491`). Y todavía hay una tercera redacción en
   `viewer-status` (`:480`). Severidad: **major** (y contradice el referente
   Stripe: «lenguaje concreto», una sola voz).
2. **Una línea azul de 2 px cruza la ventana de lado a lado bajo la cabecera**
   (y≈70). Es el borde superior del overlay de selección, que se extiende a todo
   el ancho de la página recortada; parece un artefacto de render.
   Código: `native/ui/app.slint:405-407`. Severidad: **major**.
3. **El texto de la barra inferior no está centrado.** «Selecciona una zona…»
   tiene su centro en x≈313 mientras la barra lo tiene en 360: el
   espaciador de `:492` empuja el texto a la izquierda y el relleno es asimétrico
   (12 px a la izquierda, 8 px a la derecha, `:489`). Severidad: **major**.
4. **«Cancelar» es un `Button` estándar** con radio ~6 px dentro de una barra de
   radio 17 px, y con una altura distinta a la de cualquier otro botón de la
   aplicación. Código: `native/ui/app.slint:493`. Severidad: **major**.
5. **Toda la página se tiñe de azul.** El overlay `#2563eb0d` cubre el documento
   entero en lugar de indicar sólo la zona seleccionable; el PDF deja de ser la
   superficie protagonista. Código: `native/ui/app.slint:406`.
   Roza la **Regla 1**. Severidad: **minor**.

---

## 6. `signature-placement-dark.png` — colocación de la firma

**Qué es.** Modo «elegir posición» antes de firmar.

**Defectos visibles.**

1. **El rótulo «Haz clic donde quieres mostrar la firma» aterriza encima de un
   renglón del documento** (y≈585, justo sobre una línea del PDF) porque está
   centrado verticalmente en la página. El texto azul sobre el renglón gris es
   difícil de leer y sugiere que ahí es donde irá la firma, cuando no es así.
   Código: `native/ui/app.slint:388-392`. Severidad: **blocker**
   (es exactamente la operación que `DESIGN.md` señala como el modelo mental de
   Acrobat: «colocar y luego confirmar»).
2. **Otra vez dos redacciones a la vez**: la de la página (`:389`) y la de la
   barra, «Elige dónde mostrar la firma» (`:504`), más `viewer-status` (`:635`).
   Severidad: **major**.
3. **El texto de la barra está descentrado** (centro en x≈543 frente a 590 de la
   barra), por la misma asimetría de relleno de `:502`. Severidad: **major**.
4. **No hay previsualización de la firma ni cursor de destino.** El usuario
   coloca a ciegas: no se muestra el rectángulo que ocupará la rúbrica ni sus
   dimensiones. Severidad: **major**.
5. **La página entera se tiñe de azul con borde de 2 px**, incluida la zona ya
   fuera de la ventana. Código: `native/ui/app.slint:386-387`.
   Severidad: **minor**.

---

## 7. `signature-center-dark.png` — «¿Qué necesitas hacer?» (diálogo 1)

**Qué es.** Centro de firma: cuatro intenciones.

**Defectos visibles.**

1. **«ID», «IMG», «USB» como iconos.** Tres de las cuatro tarjetas muestran una
   sigla en azul dentro de un cuadro de 38 px; la cuarta muestra un «✓». Se leen
   como marcadores de posición que alguien olvidó sustituir. Es el defecto más
   visible del diálogo. Código: `native/ui/app.slint:537-540`.
   Severidad: **blocker**.
2. **~115 px de vacío entre la descripción y la primera tarjeta.** La
   descripción termina en y≈216 y la primera tarjeta empieza en y≈325. El
   `VerticalLayout` reparte el sobrante entre elementos de altura fija dentro de
   un diálogo de 570 px fijos. Código: `native/ui/app.slint:525` (altura fija) +
   `:529-541`. Severidad: **blocker** (el diálogo se ve incompleto).
3. **Las «acciones» de la derecha no son botones.** «Continuar», «Comprobar»,
   «Abrir» y «Buscar» son `Rectangle` decorativos sin `TouchArea` ni estado de
   hover; el área pulsable es la tarjeta completa. Tres de ellas usan
   `Palette.line` como fondo, con lo que parecen botones deshabilitados.
   Código: `native/ui/app.slint:108-112`. Severidad: **blocker**
   (afordancia falsa; **Regla 2**: cada fila aparenta dos objetivos de clic).
4. **Cajas dentro de cajas.** Ventana → velo → diálogo (radio 22) → tarjeta
   (radio 14) → cuadro de icono (radio 12) + píldora de acción (radio 11).
   Cuatro radios distintos apilados en 76 px de alto. Contradice al referente
   **Linear** («menos bordes… sin convertir cada sección en una caja»).
   Código: `native/ui/app.slint:93, 99, 109`. Severidad: **major**.
5. **Contraste insuficiente entre tarjeta y diálogo.** `elevated` (#242832)
   sobre `surface` (#1b1e25) con borde blanco al 9 %: en la captura los bordes de
   las tarjetas apenas se distinguen. Código: `native/ui/app.slint:15-16, 19`.
   Severidad: **major**.
6. **La «×» de cerrar no está alineada con el título.** El título de 26 px tiene
   su base en y≈170 y la «×» su centro en y≈168, pero el glifo es un signo de
   multiplicación de 18 px que ópticamente cuelga alto y a 40 px del borde.
   Código: `native/ui/app.slint:534`. Severidad: **minor**.
7. **El velo deja ver la tarjeta de bienvenida detrás** con sus bordes
   redondeados asomando a izquierda y derecha del diálogo, lo que produce un
   segundo marco fantasma. Severidad: **minor**.

---

## 8. `signing-flow-dark.png` / `signing-flow-light.png` — confirmar firma (diálogo 2)

**Qué es.** El paso crítico del producto: elegir identidad, contraseña, nivel
PAdES, uso, motivo, ubicación, posición y apariencia.

**Defectos visibles.**

1. **La acción principal está arriba a la derecha y la salida ocupa todo el
   ancho abajo.** «Firmar y guardar» está en la fila de cabecera del diálogo
   (`native/ui/app.slint:553-560`) y «Cancelar» es un botón de ancho completo
   anclado al pie (`:655-663`). El control más ancho y más abajo del diálogo —la
   posición que todo usuario de Acrobat lee como «el botón»— es la cancelación.
   Además hay tres salidas simultáneas: «Atrás», «Cancelar» y `Esc`.
   **Rompe la Regla 2** («una acción principal por paso»).
   Severidad: **blocker**.
2. **Tres filas de opciones con tres alineaciones distintas.** «Nivel PAdES»:
   los chips empiezan en x≈419. «Uso»: empiezan en x≈448. En
   `certification-flow-light.png`, «Cambios permitidos»: empiezan en x≈607.
   Ninguna columna coincide con otra. La causa es que los `Text` de las filas
   absorben el espacio sobrante del `HorizontalLayout`.
   Código: `native/ui/app.slint:590-615`. Severidad: **blocker**.
3. **La casilla «Mostrar firma verificable…» está centrada** (x≈469–719)
   mientras todo lo demás del diálogo se alinea a la izquierda en x=294. Salta a
   la vista. Causa: `alignment: center` en `native/ui/app.slint:150`, usado en
   `:621`. Severidad: **blocker**.
4. **«Archivo» y «Tarjeta» son dos botones altísimos.** Dentro de la tarjeta de
   identidad de 86 px con 16 px de relleno, el `HorizontalLayout` los estira a
   ~54 px de alto y ~64 px de ancho: dos cuadrados enormes junto a un texto de
   dos líneas. Código: `native/ui/app.slint:563-575`. Severidad: **major**.
5. **El campo de contraseña parece deshabilitado.** En oscuro, el `LineEdit` de
   30 px tiene un fondo casi idéntico al del diálogo y un borde apenas visible;
   es el control más importante del paso y el que menos se ve.
   Código: `native/ui/app.slint:578-589`. Severidad: **major**.
6. **El título «Confirmar firma» está descentrado** (centro x≈556 frente a 590),
   por el mismo patrón de espaciadores simétricos con grupos asimétricos que la
   cabecera. Código: `native/ui/app.slint:548-552`. Severidad: **major**.
7. **Dos jerarquías de título pegadas.** «Confirmar firma» (14 px negrita, fila
   de cabecera) y «Elige tu identidad digital» (18 px negrita) quedan a 12 px de
   distancia, y el segundo es más grande que el primero: el título del diálogo
   pesa menos que el de su primera sección.
   Código: `native/ui/app.slint:551` y `:562`. Severidad: **major**.
8. **Información crítica en 11 px `muted`.** «Posición: esquina inferior
   izquierda» y «Apariencia: datos verificables» son estado real de lo que se va
   a escribir en el PDF y son el texto más pequeño y de menor contraste de la
   pantalla. Código: `native/ui/app.slint:626-627, 641`. Severidad: **major**.
9. **Cuatro patrones de fila en un solo diálogo**: etiqueta + chips, etiqueta +
   campo, dos campos en paralelo, y texto + botones a la derecha. Ninguna
   comparte columna de etiquetas. Código: `native/ui/app.slint:590-645`.
   Severidad: **major**.
10. **La barra de desplazamiento interna se dibuja sobre el borde redondeado del
    diálogo** (x≈905, de y≈130 a 670), sin margen respecto al relleno de 24 px.
    Código: `native/ui/app.slint:542-547`. Severidad: **minor**.
11. **En claro, `surface` y `elevated` son ambos #ffffff** (`app.slint:15-16`):
    la tarjeta de identidad y los campos no tienen ninguna elevación respecto al
    diálogo; sólo los separa una línea negra al 9 %. En
    `signing-flow-light.png` la tarjeta es prácticamente invisible.
    Severidad: **major**.
12. **El velo `#00000066` en tema claro** convierte toda la aplicación en gris
    plomo; el documento y la cabecera quedan sucios en lugar de atenuados.
    Código: `native/ui/app.slint:521`. Severidad: **minor**.

**Cumplimientos que sí hay que reconocer.** El texto del primario describe la
consecuencia («Firmar y guardar», Regla 3), y «Se creará un PDF nuevo y el
original no se modificará» (`:646`) hace explícita la Regla 6. El problema es que
esa frase clave va en 12 px `muted` y queda justo encima del pie, donde menos se
lee.

---

## 9. `signing-flow-compact-dark.png` — confirmar firma en ventana compacta

**Defectos visibles.**

1. **El diálogo tapa la cabecera y la corta por la mitad.** El diálogo empieza
   en y=16 y la cabecera mide 64 px: se ven las etiquetas «PDF», «B…», «T…»,
   «Inf…» y el botón «Firmar» seccionados horizontalmente por el borde superior
   del diálogo. Causa: `y: (parent.height - self.height) / 2` con altura
   `min(parent.height - 32px, 650px)` en `native/ui/app.slint:525-526`.
   Severidad: **blocker**.
2. **El contenido queda cortado y el pie se come la última fila.** La última
   fila visible es la casilla; «Posición», «Apariencia», el aviso de que no se
   sobrescribe el original y el estado de firma quedan fuera, con el pie
   «Cancelar» pisando el corte sin separación. Código: `:542-543` y `:655-656`.
   Severidad: **blocker**.
3. **Las esquinas redondeadas de la barra de páginas asoman por debajo del
   diálogo** (abajo a izquierda y derecha), como dos manchas oscuras sueltas.
   Severidad: **minor**.
4. **El velo casi no existe**: 40 px de margen a cada lado en 720 px de ancho.
   Severidad: **minor**.

---

## 10. `certification-flow-light.png` — firma de certificación (diálogo 2, DocMDP)

**Defectos visibles.**

1. **Contenido recortado por el pie.** «Apariencia: datos verificables» y los
   botones «Guardadas» / «Añadir» quedan cortados a media altura por la barra
   «Cancelar» (el corte se ve en y≈678). El contenido no termina, se
   interrumpe. Código: pie opaco superpuesto en `native/ui/app.slint:655-664`
   sobre el `ScrollView` de `:542`. Severidad: **blocker**.
2. **La fila «Cambios permitidos» aparece con sus chips pegados al borde
   derecho** (x≈607–885), en una posición completamente distinta a las dos filas
   de arriba. La aparición de esta tercera fila además desplaza todo lo de abajo
   sin animación ni anclaje. Código: `native/ui/app.slint:609-615`.
   Severidad: **blocker**.
3. **«+ Anotaciones» usa un `+` ASCII como parte de la etiqueta**, en un grupo
   donde las otras dos opciones son palabras («Ninguno», «Formularios»).
   Código: `native/ui/app.slint:614`. Severidad: **major**.
4. **Nada explica qué es certificar frente a aprobar más allá de siete palabras
   en 11 px** («Debe ser la primera firma»). Es la decisión más irreversible del
   producto. Contradice al referente **Stripe** («los pasos sensibles explican en
   lenguaje concreto qué va a pasar»). Código: `native/ui/app.slint:607`.
   Severidad: **major**.
5. **Los niveles PAdES se presentan como jerga sin traducir.** «B-B», «B-T»,
   «B-LT», «B-LTA» con una sola palabra de glosa al lado, que además cambia de
   posición según cuál esté activo. La pregunta de control de `DESIGN.md`
   («¿alguien que viene de Acrobat lo entiende sin conocer certificados?») se
   responde que no. Código: `native/ui/app.slint:592-600`. Severidad: **major**.

---

## 11. `identity-library-dark.png` / `-light.png` / `-compact-dark.png` (diálogo 8)

**Qué es.** Biblioteca de identidades `.p12`/`.pfx` guardadas.

**Defectos visibles.**

1. **~190 px de vacío entre la lista y los botones.** La lista termina en y≈375
   y «Importar .p12 o .pfx» empieza en y≈572. El diálogo se ve a medio construir.
   Causa: `vertical-stretch: 1` en el `ScrollView` (`native/ui/app.slint:818`)
   dentro de un diálogo de altura fija (`:525`). Severidad: **blocker**.
2. **Las identidades son botones de texto centrado, sin ningún dato.**
   «Identidad corporativa» y «Firma personal» son sólo la etiqueta guardada
   (`native/src/presentation.rs:811-816`): no hay titular, emisor, caducidad ni
   huella. El usuario elige un certificado a ciegas. Contradice al referente
   **Stripe** («separan los datos por función») y deja la **Regla 4/5** sin
   apoyo visual: nada distingue una identidad válida de una caducada.
   Código: `native/ui/app.slint:817-826`. Severidad: **blocker**.
3. **El párrafo de dos líneas se solapa consigo mismo.** «…y nunca se
   almacenará.» aparece pegado a la línea superior, sin interlineado; parece
   texto roto. Ocurre en todas las descripciones de dos líneas de la aplicación.
   Código: `native/ui/app.slint:809-812` (sin `line-height`).
   Severidad: **major**.
4. **Tres salidas y dos primarios apilados.** «Atrás», «×», «Cancelar» de ancho
   completo, y encima de él «Importar .p12 o .pfx» también de ancho completo.
   Dos botones idénticos en tamaño, uno azul y otro no. **Rompe la Regla 2**.
   Código: `native/ui/app.slint:827-828`. Severidad: **major**.
5. **El título flota por encima de la fila de botones.** «Identidades digitales»
   tiene su base en y≈173 mientras «Atrás» y «×» están centrados en y≈177; el
   título no comparte línea base con nada. Código: `native/ui/app.slint:772-778`
   (patrón repetido en los diálogos 5, 6, 7 y 8). Severidad: **major**.
6. **En claro, las filas son blancas sobre blanco** (`surface` = `elevated` =
   #ffffff): sólo un hilo al 9 % las separa del diálogo. Severidad: **major**.
7. **Cuatro radios distintos en 550 px de alto**: diálogo 22, fila de lista ~6
   (estándar), primario 13, «Atrás» ~6. Severidad: **minor**.
8. **En compacto, el diálogo vuelve a cortar la cabecera** por la mitad y
   «Cancelar» queda a 36 px del borde inferior de la ventana.
   Severidad: **blocker**.

---

## 12. `appearance-library-dark.png` / `-light.png` / `-compact-dark.png` (diálogo 7)

**Qué es.** Biblioteca de firmas visuales guardadas.

**Defectos visibles.**

1. **Una galería de imágenes que no muestra ni una imagen.** «Firma personal» e
   «Iniciales contrato» son etiquetas de texto centradas. El propósito declarado
   de esta pantalla en `DESIGN.md` (Regla 5: «la imagen guardada aporta
   reconocimiento») es imposible de cumplir sin miniatura.
   Código: `native/ui/app.slint:787-796`, datos en
   `native/src/presentation.rs:869-875`. Severidad: **blocker**.
2. **Los mismos ~190 px de vacío** entre la lista y «Añadir imagen».
   Código: `native/ui/app.slint:788`. Severidad: **blocker**.
3. **No hay forma de borrar ni renombrar una apariencia guardada** desde la
   pantalla que las administra. Severidad: **major**.
4. **Repite íntegro el patrón del diálogo 8**: título fuera de línea base, tres
   salidas, dos botones de ancho completo apilados, descripción sin interlineado.
   Son dos copias del mismo código con distintas cadenas
   (`native/ui/app.slint:770-799` y `:800-829`). Severidad: **major**.

---

## 13. `up-to-date-dark.png` — «Nitum PDF está actualizado» (diálogo 4)

**Defectos visibles.**

1. **«Cerrar» flota en mitad de la nada.** El botón termina en x≈654 mientras el
   borde interno del diálogo está en x≈774: quedan 120 px huérfanos a su
   derecha. La causa es que el `PrimaryButton` con `visible: false`
   (`native/ui/app.slint:726`) sigue reservando su `min-width: 120px` (`:70`) en
   el `HorizontalLayout` de `:723-727`. Es un fallo evidente y de una línea.
   Severidad: **blocker**.
2. **~124 px de vacío** entre la caja de estado (termina en y≈399) y los
   botones (y≈523), en un diálogo de sólo 360 px de alto.
   Código: `native/ui/app.slint:722` (`Rectangle { vertical-stretch: 1 }`).
   Severidad: **major**.
3. **La caja de estado parece un campo de texto vacío.** «Ya tienes la versión
   más reciente.» va centrada dentro de un rectángulo `elevated` de 60 px sin
   borde, sin icono y sin color. Un estado de éxito sin ningún indicador salvo
   la propia frase. Código: `native/ui/app.slint:717-721`. Severidad: **major**.
4. **La descripción de dos líneas vuelve a ir sin interlineado.**
   Código: `native/ui/app.slint:713-716`. Severidad: **major**.
5. **No hay jerarquía entre título (23 px) y el resto**: no existe escala, sólo
   dos tamaños y un salto. Severidad: **minor**.

---

## 14. `update-available-dark.png` — «Nitum PDF 0.7.0 está disponible» (diálogo 4)

**Defectos visibles.**

1. **La etiqueta del botón primario toca los dos bordes redondeados.**
   «Actualizar y reiniciar» ocupa prácticamente todo el ancho del botón
   (x≈670–803 dentro de un botón de 669–802): cero relleno horizontal y la
   última letra montada sobre la curva. `PrimaryButton` no tiene `padding`, sólo
   `min-width: 120px` (`native/ui/app.slint:70, 75`). El botón más importante
   del diálogo se ve roto. Severidad: **blocker**.
2. **«Ahora no» y el primario no comparten altura, radio ni línea base.** 36 px
   frente a 42 px, radio ~6 frente a 13; los textos quedan desalineados ~1 px.
   Código: `native/ui/app.slint:725-726`. Severidad: **major**.
3. **La disponibilidad de una actualización se anuncia sólo cambiando una
   palabra en la cabecera** («Versión» → «Actualizar»), sin punto, sin insignia y
   sin cambio de color. Código: `native/ui/app.slint:334`. Roza la **Regla 8**
   por el lado contrario: no hay ni color ni icono, sólo texto que nadie va a
   releer. Severidad: **major**.
4. **«Actualizar» dentro de un `HeaderButton` de 68 px fijos** deja ~3 px por
   lado; en cuanto la cadena crezca (traducción, versión larga) se saldrá.
   Código: `native/ui/app.slint:50` y `:334`. Severidad: **major**.
5. **Los mismos ~124 px de vacío** y la misma caja de estado sin icono.
   Severidad: **major**.

---

## 15. Pantallas sin captura

`DESIGN.md` Regla 9 exige captura reproducible de cada flujo importante. Faltan
tres diálogos completos en `visual-audit/`:

- **Diálogo 3 — PDF protegido / contraseña** (`native/ui/app.slint:665-706`).
- **Diálogo 5 — Comprobar firmas** (`:729-749`).
- **Diálogo 6 — Tarjetas y tokens** (`:750-769`).

De la lectura del código, el diálogo 5 tiene un defecto grave que conviene
verificar en cuanto haya captura: el resultado de la verificación se compone como
**una única frase corrida** en `native/src/presentation.rs:1329-1339` —
«N firma(s), M sello(s) de tiempo: integridad válida. Nivel(es): B-B. Sin
certificación DocMDP. Firmante(s): … Confianza del certificado: … Cobertura
completa: sí.» — y se pinta centrada dentro de un panel teñido de verde o rojo
(`native/ui/app.slint:739-746`) **sin ningún icono**. Eso incumple la **Regla 8**
(«el estado no depende sólo del color») de forma directa y desperdicia la
separación entre integridad, cobertura y confianza que el propio texto del
diálogo promete. Además «firma(s)» y «Nivel(es)» exponen la pluralización sin
resolver al usuario final. Severidad: **blocker**.

El diálogo 6 lista los tokens como `Button` estándar apilados
(`native/ui/app.slint:760-766`), sin icono, sin estado de conexión y sin
distinguir un módulo detectado de uno inaccesible.

---

## Hallazgos transversales

Ordenados por número de pantallas afectadas.

### T1. No existe escala tipográfica — 20/20 pantallas

13 tamaños de fuente distintos en un solo archivo: 11, 12, 13, 14, 15, 18, 20,
21, 23, 24, 26, 28 y 42 px, más los que hereda `std-widgets` sin declarar. Cuatro
de ellos (20, 21, 23, 24) son títulos de diálogo que deberían ser el mismo. El
11 px `muted` transporta estado real (posición de firma, apariencia elegida,
«Documento local») y es ilegible en la práctica. Sin escala no hay jerarquía, y
sin jerarquía todo compite. Código: `native/ui/app.slint` (13 declaraciones
`font-size` distintas, ver `:320, 536, 592, 626, 641, 668, 711, 734, 755, 775,
805, 357`).

### T2. No existe escala de espaciado — 20/20 pantallas

16 valores distintos de `padding`/`spacing`: 0, 1, 3, 4, 7, 8, 10, 12, 14, 16,
18, 22, 24, 28, 54, 64 px. Los diálogos usan 24 px (`:530, 547`) y 28 px
(`:666, 708, 730, 751, 771, 801`) indistintamente; la barra de páginas usa 7 px
(`:469`); la cabecera 18 px (`:314`). Nada se alinea con nada entre pantallas.

### T3. Alturas fijas de diálogo que generan huecos o cortes — 11/20 pantallas

Una sola línea, `native/ui/app.slint:525`, fija la altura de los ocho diálogos
con una cadena de ternarios. Resultado: ~115 px de hueco en el centro de firma,
~190 px en las dos bibliotecas, ~124 px en los dos diálogos de actualización, y
contenido **cortado** en `certification-flow-light` y `signing-flow-compact-dark`.
Es el defecto que más «producto sin terminar» transmite y tiene un único punto de
arreglo.

### T4. Glifos ASCII en lugar de iconos — 12/20 pantallas

18 usos: `<` `>` `-` `+` en la barra de páginas (`:470, 474, 476, 477`), `×` en
cinco diálogos (`:534, 670, 736, 757, 777, 807`), `ID`/`IMG`/`USB`/`✓` en las
tarjetas de intención (`:537-540`), `✓` en la casilla (`:155`), `✓` de 42 px como
icono heroico (`:353`), «+ PDF» (`:315`) y «+ Anotaciones» (`:614`). No hay ni un
solo icono real en toda la aplicación. Es la causa más directa de que se perciba
como no acabada, y contradice al referente **Apple** («el color siempre se
acompaña de icono y texto»).

### T5. Dos sistemas de botones conviviendo — 14/20 pantallas

20 `Button` de `std-widgets` (radio ~6 px, altura ~30-36 px, tipografía del
sistema) frente a cuatro componentes propios: `PrimaryButton` (radio 13, alto 42),
`HeaderButton` (radio 12, alto 40), `IconButton` (radio 12, 40×40) y
`LevelButton` (radio 10, alto 34). En `signing-flow-dark.png` se ven los cinco a
la vez. Ningún par comparte altura ni radio.

### T6. Radios de esquina sin sistema — 20/20 pantallas

13 valores: 3, 5, 7, 10, 11, 12, 13, 14, 16, 17, 22, 26, 28 px, más los de
`std-widgets`. En el centro de firma se apilan cuatro radios distintos en 76 px
de alto.

### T7. Descentrado por espaciadores simétricos con contenido asimétrico — 9/20

El patrón `Rectangle { horizontal-stretch: 1 }` a ambos lados de un título
aparece en la cabecera (`:316, 331`) y en los encabezados de los diálogos 2, 5, 6,
7 y 8 (`:550-552, 733-735, 754-756, 774-776, 804-806`). Como los grupos laterales
nunca miden lo mismo, **ningún título de la aplicación está realmente centrado**.
El caso más visible es la cabecera: 155 px de desvío.

### T8. `Palette.line` usado a la vez como borde y como relleno — 8/20 pantallas

El token es #ffffff18 / #10182816, es decir, ~9 % de opacidad: correcto como
hairline, inservible como fondo. Se usa como relleno en el hover de todos los
botones (`:31, 51, 129`), en la píldora del número de página (`:472`) y en las
píldoras de acción de las tarjetas de intención (`:110`). Efecto visible:
controles que parecen deshabilitados.

### T9. `surface` y `elevated` son el mismo color en tema claro — 7/20 pantallas

`native/ui/app.slint:15-16`: ambos #ffffff. En claro no hay elevación: tarjetas,
listas y campos flotan sin separación del diálogo. Se ve en
`signing-flow-light`, `certification-flow-light`, `identity-library-light`,
`appearance-library-light`.

### T10. Párrafos de dos líneas sin interlineado — 6/20 pantallas

Ninguna declaración de `line-height` en todo el archivo. Con texto de 13 px que
envuelve, las líneas quedan pegadas: visible en `identity-library-*`,
`appearance-library-compact-dark`, `up-to-date-dark`, `update-available-dark`.

### T11. Colores fuera de la paleta — 6/20 pantallas

Ocho literales que no pasan por `Palette`: `#1746b5` (`:72, 390, 412`), `#667085`
(`:383`), `#b42318` (`:385, 453`), `#d92d20` (`:650, 689, 745`), y los tintes
`#2563eb18/0d/38`. Dos de ellos (`#667085`, `#b42318`) están calibrados para
fondo claro y se usan sobre la página, por lo que no se adaptan al tema. Mientras
tanto `Palette.warning` (`:23`) está definido y **no se usa ni una vez** — no hay
ningún estado de advertencia en toda la aplicación, sólo éxito y error.

### T12. Estado codificado sólo por color — 4/20 pantallas

Los paneles de resultado de firma (`:647-651`) y de verificación (`:739-746`)
cambian entre verde (#16a66a18) y rojo (#d92d2018) **sin icono y sin etiqueta de
categoría**. La cabecera anuncia una actualización disponible sólo cambiando una
palabra. **Rompe la Regla 8** de `DESIGN.md`.

### T13. Foco visible mal resuelto — no verificable en las capturas

Ninguna captura muestra un control enfocado, así que esto es un hallazgo de
código, no de píxel: el foco se dibuja como un borde **interior** de 2 px
(`:32, 52, 73, 94, 130, 148`), sin `offset`, sobre controles de 34-42 px de alto
con radios de 10-13 px; en `PrimaryButton` el anillo es blanco sobre azul (`:73`),
y los 20 `Button` de `std-widgets` usan su propio indicador. No hay un anillo de
foco coherente. Conviene añadir una captura con foco para poder auditarlo.

### T14. Tres redacciones por modo — 2/20 pantallas (pero sistémico)

Colocación: `:389`, `:504`, `:635`. Selección: `:411`, `:491`, `:480`. Dos de
ellas se ven en pantalla simultáneamente en `signature-placement-dark` y
`text-selection-compact-dark`.

### T15. Duplicación de código de diálogo — afecta al mantenimiento

Los diálogos 7 y 8 (`:770-799` y `:800-829`) son el mismo bloque con cadenas
distintas. Cualquier arreglo de alineación hay que hacerlo dos veces, lo que
garantiza que vuelvan a divergir.

---

## Prioridad: los 20 arreglos con más impacto visual por esfuerzo

| # | Arreglo | Objetivo en código | Impacto |
|---|---|---|---|
| 1 | Reemplazar los 18 glifos ASCII por un set de iconos real (SVG en `native/ui/icons/`) y dar a `IconButton`/`IntentCard` una propiedad `image` en lugar de `string` | `native/ui/app.slint:26-44, 86-122, 470-477, 534, 537-540, 670, 736, 757, 777, 807` | Es el cambio que más «producto terminado» aporta por línea tocada |
| 2 | Sustituir la altura fija de diálogo por altura ajustada al contenido con máximo, y eliminar los `vertical-stretch: 1` de relleno | `native/ui/app.slint:525`, `:691, 722, 767, 784, 788, 814, 818` | Elimina de golpe los huecos de 115-190 px en 6 pantallas y los cortes en 2 |
| 3 | Quitar el `min-width` reservado por el botón invisible del diálogo de actualización (usar `if root.update-available:` en vez de `visible:`) | `native/ui/app.slint:726` | Una línea; arregla el «Cerrar» flotando en el vacío |
| 4 | Dar `padding` horizontal (16-20 px) a `PrimaryButton` y que su ancho crezca con el texto | `native/ui/app.slint:70, 75` | Una línea; arregla la etiqueta pegada al borde del primario más importante |
| 5 | Mover «Firmar y guardar» al pie del diálogo, a la derecha, y convertir «Cancelar» en botón secundario de ancho natural | `native/ui/app.slint:553-560, 655-664` | Cumple la Regla 2 en el paso crítico del producto |
| 6 | Introducir una columna de etiquetas de ancho fijo (p. ej. 140 px) para las filas de opciones, en lugar de dejar que los `Text` absorban el sobrante | `native/ui/app.slint:590-615` | Alinea las tres filas que hoy empiezan en x=419, 448 y 607 |
| 7 | Quitar `alignment: center` de `ToggleOption` | `native/ui/app.slint:150` | Una línea; la casilla deja de ser el único elemento centrado del diálogo |
| 8 | Centrar el título de la cabecera con posición absoluta o con espaciadores de peso calculado | `native/ui/app.slint:313-336` | Corrige los 155 px de desvío visibles en 14 capturas |
| 9 | Centrar verticalmente el contenido de la cabecera (envolver en `VerticalLayout { alignment: center }`) y dejar borde sólo abajo | `native/ui/app.slint:312-314` | Recupera los 23 px muertos; misma barra, otro aspecto |
| 10 | Implementar de verdad `fit_width` midiendo el ancho del visor, y usarlo al abrir el documento | `native/src/application.rs:146`, `native/src/presentation.rs:412`, `native/ui/app.slint:374, 479` | Única forma de cumplir la Regla 1; arregla el recorte de la página en compacto |
| 11 | Reservar hueco inferior en la lista de páginas (o hacer la barra semitransparente con retroceso al desplazar) para que no tape el PDF | `native/ui/app.slint:370, 464-467` | El documento deja de estar tapado en todas las vistas de lectura |
| 12 | Definir `line-height` (1.45) en un componente `Body`/`Caption` y usarlo en todas las descripciones | `native/ui/app.slint:536, 674, 715, 738, 759, 780, 810` | Elimina el aspecto de «texto roto» en 6 pantallas |
| 13 | Colapsar 13 tamaños de fuente en una escala de 6 (11→12, 20/21/23/24→22, etc.) mediante propiedades globales en `Palette` | `native/ui/app.slint:11-24` y todos los `font-size` | Base para cualquier jerarquía real |
| 14 | Colapsar 13 radios en 3 (6 / 12 / 20) y 16 espaciados en una escala de 4/8 | `native/ui/app.slint` (global) | Coherencia inmediata en las 20 pantallas |
| 15 | Sustituir los 20 `Button` de `std-widgets` por un `SecondaryButton` propio con la misma altura y radio que `PrimaryButton` | `native/ui/app.slint:455, 493, 506, 549, 573, 574, 630, 643, 644, 659, 694, 725, 732, 748, 753, 762, 768, 773, 791, 798, 803, 821, 828` | Elimina el segundo sistema de botones visible en 14 capturas |
| 16 | Diferenciar `elevated` de `surface` en tema claro (p. ej. #f7f8fb) y subir el contraste del borde | `native/ui/app.slint:15-16, 19` | Las tarjetas y listas dejan de ser blanco sobre blanco en 4 capturas |
| 17 | Dejar de usar `Palette.line` como relleno: crear `Palette.fill-subtle` opaco para hover, píldoras y chips | `native/ui/app.slint:19, 31, 51, 110, 129, 472` | Los controles dejan de parecer deshabilitados |
| 18 | Añadir icono y etiqueta de categoría a los paneles de estado de firma y verificación, y descomponer el resultado en filas (integridad / cobertura / confianza / firmante / nivel) | `native/ui/app.slint:647-651, 739-746`; `native/src/presentation.rs:1329-1339` | Cumple la Regla 8 y la promesa del propio texto del diálogo |
| 19 | Mostrar metadatos en las listas: titular, emisor y caducidad en identidades; miniatura en apariencias | `native/ui/app.slint:787-796, 817-826`; `native/src/presentation.rs:811-816, 869-875` | Convierte dos diálogos vacíos en pantallas útiles (Regla 5) |
| 20 | Unificar las tres redacciones de cada modo en una sola cadena y anclar el rótulo de colocación arriba, fuera del texto del PDF | `native/ui/app.slint:389, 411, 480, 491, 504, 635, 388-392` | Una voz por modo; el rótulo deja de caer sobre un renglón del documento |

---

## Respuesta a la pregunta de control de `DESIGN.md`

> ¿Una persona que viene de Acrobat entiende qué puede hacer, qué ocurrirá y cuál
> es el siguiente paso sin conocer certificados?

Hoy, no. En el diálogo de firma el botón más ancho y más abajo dice «Cancelar»,
la acción real está arriba a la derecha, las opciones se llaman «B-B» y
«B-LTA», la decisión irreversible entre aprobar y certificar se explica con siete
palabras en 11 px, y la biblioteca de identidades no dice de quién es cada
certificado. El producto respeta bien las reglas de *fondo* (no sobrescribe el
original, no vende la rúbrica como identidad, `Ctrl+F` busca) y las incumple casi
todas en la *superficie*.
