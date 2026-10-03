#!/usr/bin/env bash
# Respaldo diario cifrado de la base de datos. Lo corre cron en el VPS (ver DESPLIEGUE.md).
# Necesita ~/.citasegura-clave con una contrasena larga (guardala tambien fuera del servidor).
set -euo pipefail
cd "$(dirname "$0")"

CLAVE="$HOME/.citasegura-clave"
[ -f "$CLAVE" ] || { echo "Falta $CLAVE"; exit 1; }

# 1) copia consistente dentro del contenedor -> datos/respaldos/citasegura-FECHA.db.gz
RUTA=$(docker compose exec -T bot python -m app.respaldo /datos/respaldos | tail -1)
ARCHIVO="datos/respaldos/$(basename "$RUTA")"

# 2) cifrar y borrar la copia sin cifrar
mkdir -p cifrados
gpg --batch --yes --pinentry-mode loopback --symmetric --cipher-algo AES256 --passphrase-file "$CLAVE" \
    -o "cifrados/$(basename "$ARCHIVO").gpg" "$ARCHIVO"
rm -f "$ARCHIVO"

# 3) conservar 30 dias de respaldos cifrados
find cifrados -name 'citasegura-*.gpg' -mtime +30 -delete

# 4) (opcional) copiar fuera del servidor; un respaldo en el mismo VPS se pierde con el VPS.
#    Ejemplo con rclone ya configurado:  rclone copy cifrados remoto:citasegura
echo "Respaldo listo: cifrados/$(basename "$ARCHIVO").gpg"
