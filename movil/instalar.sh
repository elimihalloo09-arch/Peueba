#!/usr/bin/env bash
# Deja Ubuntu listo para trabajar este proyecto. Se corre UNA sola vez.
set -euo pipefail

VERSION=3.24.5
DEST="$HOME/flutter"

echo "==> Paquetes que Flutter necesita"
sudo apt-get update -qq
sudo apt-get install -y -qq curl git unzip xz-utils zip libglu1-mesa

if [ -x "$DEST/bin/flutter" ]; then
  echo "==> Flutter ya estaba en $DEST"
else
  echo "==> Bajando Flutter $VERSION (660 MB, tarda segun tu internet)"
  curl -fSL --progress-bar \
    "https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_${VERSION}-stable.tar.xz" \
    -o /tmp/flutter.tar.xz
  echo "==> Descomprimiendo en $DEST"
  tar -xJf /tmp/flutter.tar.xz -C "$HOME"
  rm -f /tmp/flutter.tar.xz
fi

# Para que 'flutter' sirva en cualquier terminal nueva, no solo en esta.
if ! grep -q 'flutter/bin' "$HOME/.bashrc" 2>/dev/null; then
  echo 'export PATH="$HOME/flutter/bin:$PATH"' >> "$HOME/.bashrc"
  echo "==> Se agrego Flutter al PATH en ~/.bashrc"
fi
export PATH="$DEST/bin:$PATH"

# El SDK viene con su propio .git y git se queja de que es de otro dueno.
git config --global --add safe.directory "$DEST" || true

flutter config --no-analytics >/dev/null
flutter --version
echo "==> Dependencias del proyecto"
flutter pub get

echo
echo "Listo. Ahora:   ./correr.sh"
