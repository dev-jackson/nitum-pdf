#!/usr/bin/env bash
set -euo pipefail

repo=${NITUM_GITHUB_REPOSITORY:-dev-jackson/nitum-pdf}
apple_certificate=''
apple_certificate_password=''
apple_sign_identity=''
apple_installer_identity=''
apple_id=''
apple_team_id=''
apple_app_password=''
windows_certificate=''
windows_certificate_password=''

die() {
  printf 'Error: %s\n' "$1" >&2
  exit 2
}

prompt() {
  local variable=$1 label=$2 value
  read -r -p "$label: " value
  [ -n "$value" ] || die "$label es obligatorio."
  printf -v "$variable" '%s' "$value"
}

prompt_secret() {
  local variable=$1 label=$2 value
  read -r -s -p "$label: " value
  printf '\n'
  [ -n "$value" ] || die "$label es obligatorio."
  printf -v "$variable" '%s' "$value"
}

set_text_secret() {
  local name=$1 value=$2
  printf '%s' "$value" | gh secret set "$name" --repo "$repo"
}

set_file_secret() {
  local name=$1 path=$2
  openssl base64 -A -in "$path" | gh secret set "$name" --repo "$repo"
}

command -v gh >/dev/null || die 'Instala GitHub CLI (gh).'
command -v openssl >/dev/null || die 'Instala OpenSSL.'
gh auth status >/dev/null 2>&1 || die 'Inicia sesión con gh auth login.'
gh repo view "$repo" >/dev/null || die "No se puede acceder a $repo."

prompt apple_certificate 'Ruta del PKCS#12 Apple (.p12/.pfx)'
[ -f "$apple_certificate" ] || die "No existe $apple_certificate."
prompt_secret apple_certificate_password 'Contraseña del PKCS#12 Apple'
openssl pkcs12 -in "$apple_certificate" -passin stdin \
  -info -noout >/dev/null 2>&1 <<<"$apple_certificate_password" \
  || die 'El PKCS#12 Apple o su contraseña no son válidos.'

prompt apple_sign_identity 'Developer ID Application (nombre completo)'
prompt apple_installer_identity 'Developer ID Installer (nombre completo)'
prompt apple_id 'Apple ID para notarización'
prompt apple_team_id 'Apple Team ID'
prompt_secret apple_app_password 'Contraseña específica de aplicación Apple'

prompt windows_certificate 'Ruta del certificado Authenticode (.pfx)'
[ -f "$windows_certificate" ] || die "No existe $windows_certificate."
prompt_secret windows_certificate_password 'Contraseña del PFX Windows'
openssl pkcs12 -in "$windows_certificate" -passin stdin \
  -info -noout >/dev/null 2>&1 <<<"$windows_certificate_password" \
  || die 'El PFX Windows o su contraseña no son válidos.'

printf 'Se cargarán nueve secretos en %s. Los valores no se mostrarán.\n' "$repo"
read -r -p 'Escribe CARGAR para continuar: ' confirmation
[ "$confirmation" = 'CARGAR' ] || die 'Operación cancelada sin modificar secretos.'

set_file_secret NITUM_APPLE_CERTIFICATE_BASE64 "$apple_certificate"
set_text_secret NITUM_APPLE_CERTIFICATE_PASSWORD "$apple_certificate_password"
set_text_secret NITUM_APPLE_SIGN_IDENTITY "$apple_sign_identity"
set_text_secret NITUM_APPLE_INSTALLER_IDENTITY "$apple_installer_identity"
set_text_secret NITUM_APPLE_ID "$apple_id"
set_text_secret NITUM_APPLE_TEAM_ID "$apple_team_id"
set_text_secret NITUM_APPLE_APP_PASSWORD "$apple_app_password"
set_file_secret NITUM_WINDOWS_CERTIFICATE_BASE64 "$windows_certificate"
set_text_secret NITUM_WINDOWS_CERTIFICATE_PASSWORD "$windows_certificate_password"

unset apple_certificate_password apple_app_password windows_certificate_password
configured=$(gh secret list --repo "$repo" --json name --jq '.[].name')
for required in \
  NITUM_APPLE_CERTIFICATE_BASE64 \
  NITUM_APPLE_CERTIFICATE_PASSWORD \
  NITUM_APPLE_SIGN_IDENTITY \
  NITUM_APPLE_INSTALLER_IDENTITY \
  NITUM_APPLE_ID \
  NITUM_APPLE_TEAM_ID \
  NITUM_APPLE_APP_PASSWORD \
  NITUM_WINDOWS_CERTIFICATE_BASE64 \
  NITUM_WINDOWS_CERTIFICATE_PASSWORD
do
  grep -qx "$required" <<<"$configured" || die "GitHub no confirmó el secreto $required."
done
printf 'Credenciales cargadas. Ya puedes fusionar el PR de release y crear la etiqueta firmada.\n'
