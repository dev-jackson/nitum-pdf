"""All user-visible text, in Spanish, in one place.

Kept apart from the logic so wording can be argued about (and translated) without
touching code. The signature wording is the whole point of this app: integrity and
identity are two different questions and are never collapsed into one scary line.
"""

APP_NAME = "Nitum PDF"

OPEN = "Abrir"
OPEN_TOOLTIP = "Abrir un PDF (Ctrl+O)"
SIGN = "Firmar este PDF"
SIGN_TOOLTIP = "Firmar o comprobar firmas (Ctrl+Mayús+S)"
CANCEL = "Cancelar"
CLOSE = "Cerrar"
UNDERSTOOD = "Entendido"

EMPTY_TITLE = "Abre un PDF para empezar"
EMPTY_BODY = "Lee, busca y firma documentos sin subirlos a ningún servidor.\nArrastra un PDF aquí o pulsa Abrir."

FILE_DIALOG_TITLE = "Abrir PDF"
FILE_FILTER = "Documentos PDF"

PAGE_OF = "de {total}"
ZOOM_IN = "Acercar"
ZOOM_OUT = "Alejar"
FIT_WIDTH = "Ajustar al ancho"

SEARCH_PLACEHOLDER = "Buscar en el documento"
SEARCH_TOOLTIP = "Buscar en el documento (Ctrl+F)"
MORE_OPTIONS = "Más opciones"
PAGE_LABEL = "Página"
SEARCH_NO_HITS = "Sin resultados para «{needle}»"
SEARCH_HITS = "{current} de {total}"
SEARCH_NEXT = "Siguiente resultado"
SEARCH_PREV = "Resultado anterior"
COPY_TEXT = "Copiar el texto de la página"
COPIED = "Texto de la página copiado"
CHECK_UPDATES = "Buscar actualizaciones"
UPDATE_CHECKING = "Buscando actualizaciones…"
UPDATE_AVAILABLE_TITLE = "Nitum PDF {version} está disponible"
UPDATE_AVAILABLE_BODY = (
    "La actualización se descargará desde GitHub, se comprobará con SHA-256 y "
    "el sistema pedirá tu contraseña para instalarla."
)
UPDATE_INSTALL = "Descargar e instalar"
UPDATE_LATEST = "Nitum PDF está actualizado"
UPDATE_FAILED = "No se pudo buscar la actualización: {reason}"
UPDATE_DOWNLOADING = "Descargando Nitum PDF {version}…"
UPDATE_READY = "Actualización descargada y verificada"
UPDATE_INSTALL_FAILED = "No se pudo iniciar el instalador: {reason}"

# --- signing ---------------------------------------------------------------
SIGN_CENTER_TITLE = "Firmas"
SIGN_CENTER_HEADING = "¿Qué necesitas hacer?"
SIGN_CENTER_BODY = (
    "Elige según quién debe firmar. Una firma digital protege el PDF y permite "
    "comprobar quién lo firmó y si cambió después."
)
SIGN_MY_DOCUMENT = "Firmar este PDF con mi identidad"
SIGN_MY_DOCUMENT_BODY = "Usar mi certificado, token o tarjeta para firmarlo digitalmente"
SIGN_REVIEW_DOCUMENT = "Comprobar las firmas de este PDF"
SIGN_REVIEW_DOCUMENT_BODY = "Ver quién firmó, si el archivo cambió y si la identidad es confiable"
SIGN_SAVE_APPEARANCE = "Guardar mi firma visual"
SIGN_SAVE_APPEARANCE_BODY = "Importar una imagen una sola vez y reutilizarla en otros PDF"
SIGN_ADD = "Añadir"
SIGN_CHANGE = "Cambiar"
SIGN_IMAGE_FILTER = "Imágenes de firma"
SIGN_IMAGE_SAVED = "Firma visual guardada para próximos documentos"
SIGN_IMAGE_FAILED = "No se pudo guardar esa imagen: {reason}"
SIGN_VISUAL_NOTE = (
    "¿Buscas escribir o dibujar una rúbrica? Una imagen por sí sola no demuestra "
    "identidad. Nitum la acompaña con un certificado digital para que pueda verificarse."
)
CONTINUE = "Continuar"
CHECK = "Comprobar"
SIGN_BANNER = "Paso 1 de 2 · Arrastra un recuadro donde debe verse la firma digital"
SIGN_HERE = "Tu firma aquí"
SIGN_TOO_SMALL = "El recuadro es demasiado pequeño; inténtalo de nuevo"
SIGN_BANNER_INVISIBLE = "Firmar sin sello visible"
SIGN_DIALOG_TITLE = "Confirmar firma digital"
SIGN_STEP_TWO = "Paso 2 de 2"
SIGN_CONFIRM_BODY = "Revisa la identidad, la apariencia y la protección antes de guardar."
SIGN_GROUP_IDENTITY = "Quién firma y cómo se verá"
SIGN_GROUP_DETAILS = "Contexto"
SIGN_GROUP_DETAILS_BODY = "Opcional · ayuda a explicar por qué y dónde se firmó"
SIGN_GROUP_PROTECTION = "Protección del documento"
SIGN_IDENTITY = "Identidad digital"
SIGN_IDENTITY_HINT = "Este nombre quedará vinculado criptográficamente al PDF"
SIGN_APPEARANCE = "Apariencia del sello"
SIGN_APPEARANCE_SAVED = "Mi firma guardada + datos verificables"
SIGN_APPEARANCE_DETAILS = "Nombre, fecha y detalles"
SIGN_APPEARANCE_MINIMAL = "Solo nombre y fecha"
SIGN_REASON = "Motivo (opcional)"
SIGN_REASON_HINT = "Ej.: Conforme con el contenido"
SIGN_LOCATION = "Lugar (opcional)"
SIGN_STRONG = "Sello de tiempo y validez a largo plazo"
SIGN_STRONG_HINT = (
    "Añade la hora certificada por un tercero y guarda dentro del PDF lo necesario "
    "para poder validarlo dentro de años. Si no hay conexión, se firma igual."
)
SIGN_WHERE_PAGE = "Aparecerá en la página {page}"
SIGN_WHERE_INVISIBLE = "Sin sello visible: la firma queda dentro del archivo"
SIGN_CERTIFY = "Bloquear cambios después de firmar"
SIGN_CERTIFY_HINT = (
    "Certifica el PDF como versión final. Solo está disponible antes de la primera firma."
)
SIGN_BUTTON = "Firmar y guardar"
SIGN_WORKING = "Firmando…"

NO_IDENTITIES_TITLE = "No hay ninguna identidad disponible"
NO_IDENTITIES_BODY = (
    "Para una firma digital necesitas un certificado. Conecta tu token o importa "
    "un archivo .p12/.pfx.\n\nCarpeta: {path}\n\n"
    "Si vienes de Windows: expórtalo desde «Administrar certificados de usuario» "
    "marcando «Exportar la clave privada»."
)
IMPORT_NOW = "Importar certificado"
IMPORT_IDENTITY = "Importar certificado…"
IMPORT_DONE = "Certificado importado: {name}"

SIGNED_TOAST = "Firmado como {name}"
SIGN_FAILED_TITLE = "No se pudo firmar"
DOWNGRADED_TITLE = "Firmado, pero sin sello de tiempo"
DOWNGRADED_BODY = (
    "No se pudo contactar con el servicio de sellado de tiempo, así que el documento "
    "se firmó sin él. La firma es válida.\n\nDetalle técnico: {reason}"
)

# --- verification ----------------------------------------------------------
VERIFY = "Verificar firmas"
BANNER_DETAILS = "Ver detalles"
BANNER_ALL_GOOD = "{count} · Documento íntegro · Identidad verificada"
BANNER_UNVERIFIED = "{count} · Documento íntegro · Identidad por verificar"
BANNER_BROKEN = "Este documento fue alterado después de firmarse"
VERIFY_TITLE = "Estado de las firmas"
NO_SIGNATURES_TITLE = "Este documento no está firmado"
NO_SIGNATURES_BODY = "No contiene ninguna firma digital."

INTACT_YES = "Documento íntegro"
INTACT_YES_BODY = "No se ha modificado desde que se firmó."
INTACT_NO = "Documento alterado"
INTACT_NO_BODY = "El contenido cambió después de firmarse. No confíes en él."
TRUST_YES = "Identidad verificada"
TRUST_YES_BODY = "El certificado procede de una autoridad en la que confías."
TRUST_NO = "Identidad no verificada"
TRUST_NO_BODY = (
    "La firma es correcta, pero el certificado no está en ninguna lista de confianza. "
    "Comprueba con quien te lo envió que el titular es quien dice ser."
)
SIGNED_BY = "Firmado por {signer}"
SIGNED_AT = "Fecha: {when}"
LEVEL = "Nivel: {level}"
LEVEL_HUMAN = {
    "PAdES B-LT": "Con sello de tiempo y datos para validarla dentro de años",
    "PAdES B-T": "Con sello de tiempo certificado por un tercero",
    "PAdES B-B": "Firma básica, sin sello de tiempo",
}
ORGANIZATION = "Organización: {org}"
ZOOM_LABEL = "{percent}%"

TRUST_BUTTON = "Confiar en este emisor"
TRUST_CONFIRM_TITLE = "¿Confiar en «{name}»?"
TRUST_CONFIRM_BODY = (
    "A partir de ahora, cualquier documento firmado con este certificado aparecerá "
    "como verificado en este equipo.\n\n"
    "Hazlo solo si has comprobado por otra vía (llamada, en persona) que la huella "
    "digital coincide:\n\n{fingerprint}"
)
TRUST_CONFIRM_ACTION = "Confiar"
TRUST_DONE = "Emisor añadido a tus certificados de confianza"

SUMMARY_ALL_GOOD = "Todo correcto: {count}, todas íntegras y verificadas"
SUMMARY_UNVERIFIED = "{count} sin identidad verificada"
SUMMARY_BROKEN = "El documento fue alterado después de firmarse"


def signature_count(n: int) -> str:
    return "1 firma digital" if n == 1 else f"{n} firmas digitales"

PASSWORD_TITLE = "Este PDF está protegido"
PASSWORD_BODY = "Escribe la contraseña para abrir «{name}»."
PASSWORD_WRONG = "Contraseña incorrecta"
OPEN_ACTION = "Abrir"

ERROR_WRONG_SECRET = "La contraseña del certificado no es correcta."
ERROR_WRONG_PIN = "El PIN del token no es correcto."
ERROR_PIN_LOCKED = "El token está bloqueado por demasiados intentos fallidos."
ERROR_NO_TOKEN = "No se encuentra el token. ¿Está conectado?"
ERROR_NO_MODULE = "No se pudo cargar el controlador PKCS#11 del token."
ERROR_TITLE = "Algo ha fallado"
