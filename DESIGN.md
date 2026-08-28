# Principios de producto de Nitum PDF

La pregunta de control para cada pantalla es:

> ¿Una persona que viene de Acrobat entiende qué puede hacer, qué ocurrirá y
> cuál es el siguiente paso sin conocer certificados?

## Cinco referentes, no imitaciones

- **Adobe Acrobat:** separar firma visual, firma digital, certificación y
  validación; conservar el modelo mental de colocar y luego confirmar.
- **Linear:** el documento recibe el mayor peso visual; navegación y opciones
  secundarias retroceden. Menos bordes, iconos más discretos y una jerarquía que
  se percibe sin convertir cada sección en una caja.
- **Apple:** controles nativos, alineación consistente, adaptación al tamaño de
  ventana y divulgación progresiva. El color siempre se acompaña de icono y texto.
- **Stripe:** los pasos sensibles explican en lenguaje concreto qué va a pasar,
  separan los datos por función y ofrecen una sola acción principal inequívoca.
- **Figma:** herramientas contextuales, estados inmediatos y atajos previsibles
  sin apartar a la persona del documento que está manipulando.

## Reglas verificables

1. El PDF es la superficie principal y abre ajustado al ancho útil.
2. Una acción principal por paso.
3. Los textos describen consecuencia: “Firmar y guardar”, no solo “Aceptar”.
4. Nunca se presenta una rúbrica visual como prueba de identidad.
5. El certificado aporta validez; la imagen guardada aporta reconocimiento.
6. El original nunca se sobrescribe.
7. `Ctrl+F` siempre busca; no se reemplazan convenciones universales.
8. El estado no depende solo del color.
9. Cada flujo importante tiene captura reproducible y prueba funcional.
10. Antes de publicar se repite la pregunta de control de este documento.

## Reglas de superficie

Las diez de arriba son de producto. Estas cinco son de acabado, y salieron de
medir las capturas una por una en agosto de 2026 (ver `docs/ux-audit.md` para lo
que estaba mal y `docs/ux-spec.md` para los números que lo sustituyen).

11. Ningún archivo salvo `native/ui/theme.slint` nombra un color, un tamaño de
    letra, un radio o un espaciado. Si un valor hace falta y no está, se añade a
    la escala; no se escribe suelto.
12. Todo par de colores cumple WCAG 2.2 y se comprueba solo:
    `cargo test --test theme_contrast`. Un color nuevo que no llegue al mínimo
    rompe la compilación de pruebas.
13. Una pantalla tiene una columna de contenido. Título, texto, filas e iconos
    empiezan en la misma x; las acciones al final de una fila terminan en la
    misma x. Se comprueba midiendo la captura, no mirándola.
14. Un control que se reconstruye no se anima. Una animación de fondo sigue
    pintando el color de la instancia anterior, y el resultado es que la
    selección aparece en la fila equivocada.
15. Un control que no se puede usar no se enseña. Si no hay documento abierto no
    hay nada que firmar, así que el botón no está: no se deja apagado.
