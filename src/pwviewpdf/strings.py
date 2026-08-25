"""All user-visible text, in Spanish, in one place.

Kept apart from the logic so wording can be argued about (and translated) without
touching code. The signature wording is the whole point of this app: integrity and
identity are two different questions and are never collapsed into one scary line.
"""

APP_NAME = "Nitum PDF"

OPEN = "Abrir"
OPEN_TOOLTIP = "Abrir un PDF (Ctrl+O)"
SIGN = "Firmar"
SIGN_TOOLTIP = "Firmar este documento (Ctrl+F)"
CANCEL = "Cancelar"
CLOSE = "Cerrar"
UNDERSTOOD = "Entendido"

EMPTY_TITLE = "Abre un PDF para empezar"
EMPTY_BODY = "Arrastra un archivo aquí o pulsa Abrir."

FILE_DIALOG_TITLE = "Abrir PDF"
FILE_FILTER = "Documentos PDF"

PAGE_OF = "de {total}"
ZOOM_IN = "Acercar"
ZOOM_OUT = "Alejar"
FIT_WIDTH = "Ajustar al ancho"

SEARCH_PLACEHOLDER = "Buscar en el documento"
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
SIGN_BANNER = "Arrastra un recuadro donde quieres que aparezca tu firma"
SIGN_HERE = "Tu firma aquí"
SIGN_TOO_SMALL = "El recuadro es demasiado pequeño; inténtalo de nuevo"
SIGN_BANNER_INVISIBLE = "Firma invisible"
SIGN_DIALOG_TITLE = "Firmar documento"
SIGN_IDENTITY = "Firmar como"
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
SIGN_BUTTON = "Firmar"
SIGN_WORKING = "Firmando…"

NO_IDENTITIES_TITLE = "No hay ninguna identidad disponible"
NO_IDENTITIES_BODY = (
    "Conecta tu token o copia tu certificado .p12/.pfx a:\n{path}\n\n"
    "Si vienes de Windows: expórtalo desde «Administrar certificados de usuario» "
    "marcando «Exportar la clave privada»."
)
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
BANNER_ALL_GOOD = "Firmado y verificado"
BANNER_UNVERIFIED = "Firmado · identidad sin verificar"
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

SUMMARY_ALL_GOOD = "Todo correcto: {n} firma(s) íntegra(s) y verificada(s)"
SUMMARY_UNVERIFIED = "{n} firma(s) sin identidad verificada"
SUMMARY_BROKEN = "El documento fue alterado después de firmarse"

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
