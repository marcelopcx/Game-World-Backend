#!/usr/bin/env bash
# Inicializa el proyecto: .env, PostgreSQL y esquema game_world.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ ! -f .env ]]; then
  cp .env.example .env
  echo "→ Creado .env desde .env.example"
fi

# shellcheck disable=SC1091
set -a
source .env
set +a

echo "→ Levantando PostgreSQL (Docker)..."
docker compose up -d --wait

schema_exists() {
  docker compose exec -T db psql -U gameworld -d game_world -tAc \
    "SELECT 1 FROM information_schema.schemata WHERE schema_name = 'game_world'" \
    | grep -q 1
}

if schema_exists; then
  echo "→ Esquema game_world ya está aplicado."
  "$ROOT/scripts/migrate-db.sh"
else
  echo "→ Aplicando esquema desde db/game_world.sql..."
  docker compose exec -T db psql -U gameworld -d game_world < db/game_world.sql
  echo "→ Esquema aplicado."
fi

catalog_seeded() {
  docker compose exec -T db psql -U gameworld -d game_world -tAc \
    "SELECT 1 FROM game_world.generos LIMIT 1" \
    | grep -q 1
}

if catalog_seeded; then
  echo "→ Catálogo (géneros/plataformas) ya tiene datos."
else
  echo "→ Cargando catálogo inicial (db/seed_data.sql)..."
  docker compose exec -T db psql -U gameworld -d game_world < db/seed_data.sql
fi

echo ""
echo "Listo. Puedes ejecutar: cargo run"
echo "DATABASE_URL=${DATABASE_URL}"
