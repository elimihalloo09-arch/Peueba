#!/usr/bin/env bash
# Compila la version web y reescribe index.html para publicarla como pagina
# alojada: sin doctype ni html/head/body propios, porque el alojamiento
# envuelve la pagina en su propio esqueleto y ahi un <base> escrito a mano
# queda dentro del <body>, donde Safari lo ignora y nada carga.
set -euo pipefail
export PATH="$HOME/flutter/bin:$PATH"
cd "$(dirname "$0")"

flutter build web --release --web-renderer html --pwa-strategy offline-first

# CanvasKit no se usa con el renderizador html: son 19 MB de mas.
rm -rf build/web/canvaskit build/web/assets/AssetManifest.bin build/web/.last_build_id

python3 empaquetar.py

echo "build/web listo para publicar"
