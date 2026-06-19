#!/usr/bin/env bash
# Borra el volumen de PostgreSQL y vuelve a ejecutar setup.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "→ Deteniendo contenedor y eliminando volumen de datos..."
docker compose down -v

exec "$ROOT/scripts/setup.sh"
