# Migración de Nitum PDF a Rust + Slint

Nitum PDF se implementa íntegramente en Rust + Slint. La antigua implementación
GTK no forma parte del árbol, los artefactos ni las pruebas.

## Arquitectura

1. **Aplicación y experiencia:** Rust + Slint 1.17, sin WebView.
2. **PDF:** renderizado virtualizado fuera del hilo de UI; solo se conservan en
   memoria las páginas próximas al área visible.
3. **Firma:** Rust nativo mediante un puerto de firma PAdES; el adaptador de
   software usa `underskrift` y el de hardware usa `cryptoki`. Python y pyHanko
   no se incluyen en la aplicación, ni siquiera como respaldo opcional.
4. **Migración:** se reutilizan identidades, firmas visuales, confianza y
   preferencias existentes; nunca se copian contraseñas.
5. **Plataformas:** Linux es el primer destino verificable; Windows y macOS se
   habilitan cuando firma, almacenamiento seguro y empaquetado tienen paridad.

No existe respaldo opcional en Python. La paridad se protege con pruebas Rust de
dominio, adaptadores reales, firmas, PDF cifrado y capturas visuales headless.

## Reglas de diseño del código

- **Responsabilidad única:** Slint presenta estado; los casos de uso coordinan;
  los adaptadores leen PDF, firman, guardan y actualizan. Ninguna capa mezcla
  esas responsabilidades.
- **Abierto a extensión:** formatos de identidad, motores PDF y proveedores de
  firma se agregan implementando puertos estables, sin modificar los casos de
  uso existentes.
- **Sustitución segura:** cada implementación debe cumplir el mismo contrato de
  resultados, cancelación y errores; las pruebas de contrato se ejecutan contra
  los adaptadores Rust nativos.
- **Interfaces pequeñas:** se separan lectura, renderizado, búsqueda, firma,
  verificación, preferencias y actualización. Un consumidor depende únicamente
  de las capacidades que utiliza.
- **Inversión de dependencias:** dominio y casos de uso no importan Slint,
  `underskrift`, PDFium, red ni almacenamiento. Esos detalles dependen de interfaces
  definidas en el núcleo.

Estructura prevista: `domain` para tipos e invariantes; `application` para casos
de uso y puertos; `infrastructure` para adaptadores; `presentation` para el
estado y los controladores Slint. El punto de entrada solo compone dependencias.

## Principios de producto

- **Linear:** el documento domina; controles secundarios retroceden y la
  estructura se percibe con espacio, no con bordes innecesarios.
- **Apple:** jerarquía, adaptación, accesibilidad y convenciones de plataforma.
- **Stripe:** acciones sensibles explícitas, una acción principal y estados que
  siempre explican la consecuencia.
- **Adobe:** modelo mental familiar para identidades, colocación, firma,
  certificación y verificación, con menos pasos y errores más accionables.

## Puertas de paridad

- Abrir PDF normal, cifrado y dañado.
- Desplazamiento continuo, zoom, ajuste al ancho, búsqueda y copia.
- `.p12/.pfx` y PKCS#11 con los mismos controles de seguridad.
- Firma visible/invisible, sello de tiempo, LTV, certificación y firmas sucesivas.
- Verificación separada de integridad y confianza.
- Actualización firmada/verificada con reinicio y recuperación del documento.
- Temas claro/oscuro/sistema, teclado completo, lector de pantalla y ventanas
  desde 720×560 hasta alta densidad.
- Inicio < 1 s, documento inicial < 500 ms, interacción a 60 FPS y memoria en
  reposo < 150 MiB en el equipo de referencia.
