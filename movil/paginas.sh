#!/usr/bin/env bash
# Compila el sitio igual que lo hace el workflow de Pages, para poder verlo
# en local antes de publicarlo. La direccion queda en:
#   https://<usuario>.github.io/<repositorio>/
set -euo pipefail
export PATH="$HOME/flutter/bin:$PATH"
cd "$(dirname "$0")"

REPO="${1:-Peueba}"

flutter build web --release \
  --web-renderer html \
  --base-href "/$REPO/" \
  --pwa-strategy offline-first

# CanvasKit no se usa con el renderizador html: son 19 MB de mas.
rm -rf build/web/canvaskit build/web/.last_build_id

echo "build/web listo, $(du -sh build/web | cut -f1)"
