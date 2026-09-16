#!/usr/bin/env bash
# Levanta todo. Si algo falta, te dice qué y se detiene.
set -euo pipefail
cd "$(dirname "$0")"

echo "==> Revisando herramientas"
for h in cargo node npm; do
  command -v "$h" >/dev/null || { echo "  falta $h"; exit 1; }
done
echo "  cargo $(cargo --version | cut -d' ' -f2) · node $(node --version)"

echo "==> Base de datos"
if command -v docker >/dev/null && docker info >/dev/null 2>&1; then
  docker compose up -d
  echo -n "  esperando a Postgres"
  for _ in $(seq 1 30); do
    if docker compose exec -T bd pg_isready -U carlos -d finanzas >/dev/null 2>&1; then
      echo " · lista"; break
    fi
    echo -n "."; sleep 1
  done
else
  echo "  Docker no está corriendo. Usa tu propio Postgres y ajusta DATABASE_URL en backend/.env"
fi

[ -f backend/.env ] || { cp .env.example backend/.env; echo "==> Creé backend/.env"; }

echo "==> Dependencias del frontend"
[ -d frontend/node_modules ] || (cd frontend && npm install --no-audit --no-fund)

echo "==> Backend en el puerto 3000"
(cd backend && cargo run) &
API=$!
sleep 3

echo "==> Frontend en el puerto 4200"
(cd frontend && npm start) &
WEB=$!

echo ""
echo "  Abre http://localhost:4200"
echo "  Ojo: tiene que ser localhost, no 127.0.0.1, o CORS lo bloquea."
echo "  Ctrl+C para detener todo."
trap 'kill $API $WEB 2>/dev/null || true' INT TERM
wait
