#!/usr/bin/env bash
# Ejecuta la colección Bruno de Game World (requiere backend en marcha).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
API_DIR="$ROOT/API"

if ! curl -sf "${BASE_URL:-http://127.0.0.1:8080}/health" >/dev/null; then
  echo "ERROR: el backend no responde en ${BASE_URL:-http://127.0.0.1:8080}"
  echo "       Levanta el servidor con: make run"
  exit 1
fi

cd "$API_DIR"

if command -v bru >/dev/null 2>&1; then
  BRU=(bru)
else
  BRU=(npx --yes @usebruno/cli)
fi

echo "=== Game World — Bruno API ==="
echo "Colección: $API_DIR"
echo ""

"${BRU[@]}" run --env local --reporter-junit reports/junit.xml

echo ""
echo "Reporte JUnit: API/reports/junit.xml"
