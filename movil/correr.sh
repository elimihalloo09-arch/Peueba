#!/usr/bin/env bash
# Levanta la app con recarga en caliente, y la deja alcanzable desde el
# celular por la misma red WiFi. Ctrl+C para cerrar.
set -euo pipefail
export PATH="$HOME/flutter/bin:$PATH"
cd "$(dirname "$0")"

command -v flutter >/dev/null || { echo "Falta Flutter. Corre primero: ./instalar.sh"; exit 1; }

# La IP de la laptop en la red local, que es la que se teclea en el iPhone.
IP="$(ip route get 1.1.1.1 2>/dev/null | awk '{for(i=1;i<=NF;i++) if($i=="src") print $(i+1)}' | head -1)"
[ -n "${IP:-}" ] || IP="$(hostname -I 2>/dev/null | awk '{print $1}')"

echo
echo "  En esta laptop:   http://localhost:8080"
if [ -n "${IP:-}" ]; then
  echo "  En el iPhone:     http://$IP:8080"
  echo "                    (mismo WiFi; en Safari, Compartir > Agregar a inicio)"
else
  echo "  En el iPhone:     no se pudo leer la IP local; corre 'hostname -I'"
fi
echo
echo "  En esta terminal:  R = recargar con tus cambios    q = salir"
echo
flutter run -d web-server --web-port 8080 --web-hostname 0.0.0.0
