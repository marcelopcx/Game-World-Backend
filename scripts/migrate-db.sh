#!/usr/bin/env bash
# Aplica migraciones incrementales sobre una BD game_world existente.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ ! -d db/migrations ]]; then
  echo "→ Sin carpeta db/migrations, nada que aplicar."
  exit 0
fi

shopt -s nullglob
files=(db/migrations/*.sql)
shopt -u nullglob

if [[ ${#files[@]} -eq 0 ]]; then
  echo "→ Sin archivos de migración."
  exit 0
fi

echo "→ Aplicando migraciones en orden..."
for f in "${files[@]}"; do
  echo "   • $(basename "$f")"
  docker compose exec -T db psql -U gameworld -d game_world -v ON_ERROR_STOP=1 < "$f"
done
echo "→ Migraciones aplicadas."
