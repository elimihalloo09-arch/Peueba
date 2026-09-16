#!/usr/bin/env bash
# Compila el sitio que sirve GitHub Pages y lo deja en docs/, que es de donde
# Pages publica. La direccion queda en:
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

rm -rf ../docs
mkdir -p ../docs
cp -r build/web/. ../docs/
# Sin esto GitHub Pages ignora los archivos y carpetas que empiezan con guion
# bajo, que Flutter si genera.
touch ../docs/.nojekyll

echo "docs/ listo, $(du -sh ../docs | cut -f1)"
