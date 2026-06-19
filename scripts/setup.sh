#!/usr/bin/env bash
# Inicializa el proyecto: .env y PostgreSQL local.
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

echo ""
echo "Listo. Agregá el esquema SQL en db/ y ejecutá: cargo run"
echo "DATABASE_URL=${DATABASE_URL}"
