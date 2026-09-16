#!/usr/bin/env bash
# Deja el SDK de Flutter listo dentro del Codespace, para poder compilar
# desde el navegador del telefono sin instalar nada en el.
set -euo pipefail

VERSION=3.24.5
DEST=/opt/flutter
URL="https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_${VERSION}-stable.tar.xz"

if [ ! -x "$DEST/bin/flutter" ]; then
  echo "Bajando Flutter $VERSION (660 MB, tarda unos minutos)..."
  curl -fSL "$URL" -o /tmp/flutter.tar.xz
  sudo mkdir -p "$(dirname "$DEST")"
  sudo tar -xJf /tmp/flutter.tar.xz -C "$(dirname "$DEST")"
  sudo chown -R "$(id -u):$(id -g)" "$DEST"
  rm -f /tmp/flutter.tar.xz
fi

# Sin esto git se queja de que el SDK es de otro dueño y flutter no arranca.
git config --global --add safe.directory "$DEST"

"$DEST/bin/flutter" --version
"$DEST/bin/flutter" precache --web
"$DEST/bin/flutter" pub get --directory movil

echo
echo "Listo. Para verla:"
echo "  cd movil && flutter run -d web-server --web-port 8080 --web-hostname 0.0.0.0"
