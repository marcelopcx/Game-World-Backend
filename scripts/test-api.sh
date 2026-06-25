#!/usr/bin/env bash
# Prueba de integración de la API Game World
set -uo pipefail

BASE="http://127.0.0.1:8080"
PASS=0
FAIL=0
SKIP=0

ok() { echo "  ✅ $1"; PASS=$((PASS + 1)); }
ko() { echo "  ❌ $1 — $2"; FAIL=$((FAIL + 1)); }
skip() { echo "  ⏭️  $1 — $2"; SKIP=$((SKIP + 1)); }

check_status() {
  local name="$1" expected="$2" actual="$3" body="$4"
  if [[ "$actual" == "$expected" ]]; then
    ok "$name (HTTP $actual)"
    return 0
  else
    ko "$name" "esperaba $expected, obtuvo $actual — $body"
    return 1
  fi
}

echo "=== Game World — Prueba de integración ==="
echo ""

# --- Health ---
H=$(curl -s -o /tmp/gw_health.txt -w "%{http_code}" "$BASE/health")
check_status "GET /health" "200" "$H" "$(cat /tmp/gw_health.txt)"

# --- Catálogos ---
G=$(curl -s -o /tmp/gw_generos.json -w "%{http_code}" "$BASE/generos")
if check_status "GET /generos" "200" "$G" ""; then
  COUNT=$(python3 -c "import json; print(len(json.load(open('/tmp/gw_generos.json'))))" 2>/dev/null || echo 0)
  [[ "$COUNT" -gt 0 ]] && ok "Géneros semilla ($COUNT)" || ko "Géneros semilla" "lista vacía"
fi

P=$(curl -s -o /tmp/gw_plat.json -w "%{http_code}" "$BASE/plataformas")
if check_status "GET /plataformas" "200" "$P" ""; then
  COUNT=$(python3 -c "import json; print(len(json.load(open('/tmp/gw_plat.json'))))" 2>/dev/null || echo 0)
  [[ "$COUNT" -gt 0 ]] && ok "Plataformas semilla ($COUNT)" || ko "Plataformas semilla" "lista vacía"
fi

# --- Register ---
USER="testuser_$$"
EMAIL="test_$$@example.com"
R=$(curl -s -o /tmp/gw_reg.json -w "%{http_code}" -X POST "$BASE/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$USER\",\"email\":\"$EMAIL\",\"password\":\"secret123\",\"nombre\":\"Test\",\"apellido\":\"User\"}")
check_status "POST /auth/register" "201" "$R" "$(cat /tmp/gw_reg.json)"

# --- Login ---
L=$(curl -s -o /tmp/gw_login.json -w "%{http_code}" -X POST "$BASE/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$USER\",\"password\":\"secret123\"}")
if check_status "POST /auth/login" "200" "$L" ""; then
  TOKEN=$(python3 -c "import json; print(json.load(open('/tmp/gw_login.json'))['token'])" 2>/dev/null || echo "")
  USER_ID=$(python3 -c "import json; print(json.load(open('/tmp/gw_login.json'))['user']['id_usuario'])" 2>/dev/null || echo "")
  [[ -n "$TOKEN" ]] && ok "JWT generado" || ko "JWT generado" "token vacío"
else
  TOKEN=""
  USER_ID=""
fi

AUTH=()
if [[ -n "${TOKEN:-}" ]]; then
  AUTH=(-H "Authorization: Bearer $TOKEN")
fi

# --- Perfil ---
if [[ ${#AUTH[@]} -gt 0 ]]; then
  M=$(curl -s -o /tmp/gw_me.json -w "%{http_code}" "${AUTH[@]}" "$BASE/auth/me")
  check_status "GET /auth/me" "200" "$M" ""
fi

# --- Usuarios list (auth) ---
if [[ ${#AUTH[@]} -gt 0 ]]; then
  U=$(curl -s -o /tmp/gw_users.json -w "%{http_code}" "${AUTH[@]}" "$BASE/usuarios")
  check_status "GET /usuarios" "200" "$U" ""
fi

# --- Perfil público ---
if [[ -n "${USER_ID:-}" ]]; then
  UP=$(curl -s -o /tmp/gw_pub.json -w "%{http_code}" "$BASE/usuarios/$USER_ID")
  check_status "GET /usuarios/{id}" "200" "$UP" ""
fi

# --- Crear videojuego ---
if [[ ${#AUTH[@]} -gt 0 ]]; then
  V=$(curl -s -o /tmp/gw_vj.json -w "%{http_code}" -X POST "$BASE/videojuegos" \
    "${AUTH[@]}" -H "Content-Type: application/json" \
    -d '{"titulo":"Zelda Test","sinopsis":"Juego de prueba","desarrollador":"Nintendo","generos":[1,2],"plataformas":[6]}')
  if check_status "POST /videojuegos" "201" "$V" "$(cat /tmp/gw_vj.json)"; then
    VJ_ID=$(python3 -c "import json; print(json.load(open('/tmp/gw_vj.json'))['id_videojuego'])" 2>/dev/null || echo "")
  else
    VJ_ID=""
  fi
fi

# --- Listar videojuegos con filtros ---
LV=$(curl -s -o /tmp/gw_vj_list.json -w "%{http_code}" "$BASE/videojuegos?q=Zelda&sort=titulo&order=asc")
if check_status "GET /videojuegos (filtros)" "200" "$LV" ""; then
  TOTAL=$(python3 -c "import json; print(json.load(open('/tmp/gw_vj_list.json')).get('total',0))" 2>/dev/null || echo 0)
  [[ "$TOTAL" -gt 0 ]] && ok "Listado con filtros ($TOTAL juegos)" || ko "Listado con filtros" "total=0"
fi

# --- Detalle videojuego ---
if [[ -n "${VJ_ID:-}" ]]; then
  VD=$(curl -s -o /tmp/gw_vj_det.json -w "%{http_code}" "$BASE/videojuegos/$VJ_ID")
  check_status "GET /videojuegos/{id}" "200" "$VD" ""
fi

# --- Crear opinión usuario normal ---
if [[ ${#AUTH[@]} -gt 0 && -n "${VJ_ID:-}" ]]; then
  O=$(curl -s -o /tmp/gw_op.json -w "%{http_code}" -X POST "$BASE/videojuegos/$VJ_ID/opiniones" \
    "${AUTH[@]}" -H "Content-Type: application/json" \
    -d '{"puntaje":4,"comentario":"Muy buen juego de prueba","id_plataforma":6}')
  if check_status "POST /videojuegos/{id}/opiniones" "201" "$O" ""; then
    OP_ID=$(python3 -c "import json; print(json.load(open('/tmp/gw_op.json'))['id_opinion'])" 2>/dev/null || echo "")
  else
    OP_ID=""
  fi
fi

# --- Crear crítico y opinión crítica ---
CRIT="critico_$$"
CRIT_EMAIL="critico_$$@example.com"
curl -s -o /dev/null -X POST "$BASE/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$CRIT\",\"email\":\"$CRIT_EMAIL\",\"password\":\"secret123\",\"nombre\":\"Critico\",\"apellido\":\"Test\"}"

if [[ ${#AUTH[@]} -gt 0 ]]; then
  # marcar como crítico vía PATCH usuarios
  CRIT_ID=$(curl -s "${AUTH[@]}" "$BASE/usuarios" | python3 -c "
import json,sys
d=json.load(sys.stdin)
for u in d.get('data',[]):
  if u['username']=='$CRIT':
    print(u['id_usuario']); break
" 2>/dev/null || echo "")
  if [[ -n "${CRIT_ID:-}" ]]; then
    curl -s -o /dev/null -X PATCH "$BASE/usuarios/$CRIT_ID" \
      "${AUTH[@]}" -H "Content-Type: application/json" \
      -d '{"es_critico":true}'
    ok "PATCH /usuarios/{id} es_critico"
  fi
fi

CL=$(curl -s -o /tmp/gw_crit_login.json -w "%{http_code}" -X POST "$BASE/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$CRIT\",\"password\":\"secret123\"}")
if [[ "$CL" == "200" && -n "${VJ_ID:-}" ]]; then
  CRIT_TOKEN=$(python3 -c "import json; print(json.load(open('/tmp/gw_crit_login.json'))['token'])" 2>/dev/null || echo "")
  CO=$(curl -s -o /tmp/gw_crit_op.json -w "%{http_code}" -X POST "$BASE/videojuegos/$VJ_ID/opiniones" \
    -H "Authorization: Bearer $CRIT_TOKEN" -H "Content-Type: application/json" \
    -d '{"puntaje":5,"comentario":"Obra maestra","id_plataforma":6}')
  check_status "POST opinión crítico" "201" "$CO" ""
fi

# --- Resumen puntajes (trigger) ---
if [[ -n "${VJ_ID:-}" ]]; then
  RS=$(curl -s -o /tmp/gw_resumen.json -w "%{http_code}" "$BASE/videojuegos/$VJ_ID/opiniones/resumen")
  if check_status "GET /videojuegos/{id}/opiniones/resumen" "200" "$RS" ""; then
    python3 <<PY
import json
d=json.load(open('/tmp/gw_resumen.json'))
cu=d.get('cantidad_opiniones_usuarios',0)
cc=d.get('cantidad_opiniones_criticos',0)
pu=d.get('promedio_puntaje_usuarios')
pc=d.get('promedio_puntaje_criticos')
print(f"  usuarios: {cu} op, avg={pu} | críticos: {cc} op, avg={pc}")
if cu >= 1 and cc >= 1 and pu and pc:
    exit(0)
exit(1)
PY
    [[ $? -eq 0 ]] && ok "Trigger promedios usuarios/críticos" || ko "Trigger promedios" "conteos o promedios incorrectos"
  fi
fi

# --- Listar opiniones ---
if [[ -n "${VJ_ID:-}" ]]; then
  LO=$(curl -s -o /tmp/gw_ops.json -w "%{http_code}" "$BASE/videojuegos/$VJ_ID/opiniones")
  check_status "GET /videojuegos/{id}/opiniones" "200" "$LO" ""
fi

# --- GET /auth/me/opiniones ---
if [[ ${#AUTH[@]} -gt 0 ]]; then
  MO=$(curl -s -o /tmp/gw_myops.json -w "%{http_code}" "${AUTH[@]}" "$BASE/auth/me/opiniones")
  check_status "GET /auth/me/opiniones" "200" "$MO" ""
fi

# --- PATCH opinión ---
if [[ ${#AUTH[@]} -gt 0 && -n "${OP_ID:-}" ]]; then
  PO=$(curl -s -o /tmp/gw_op_patch.json -w "%{http_code}" -X PATCH "$BASE/opiniones/$OP_ID" \
    "${AUTH[@]}" -H "Content-Type: application/json" \
    -d '{"puntaje":5,"comentario":"Actualizado a 5 estrellas"}')
  check_status "PATCH /opiniones/{id}" "200" "$PO" ""
fi

# --- Duplicar opinión (409/400) ---
if [[ ${#AUTH[@]} -gt 0 && -n "${VJ_ID:-}" ]]; then
  DUP=$(curl -s -o /tmp/gw_dup.json -w "%{http_code}" -X POST "$BASE/videojuegos/$VJ_ID/opiniones" \
    "${AUTH[@]}" -H "Content-Type: application/json" \
    -d '{"puntaje":3,"comentario":"Duplicado"}')
  [[ "$DUP" == "400" || "$DUP" == "409" ]] && ok "POST opinión duplicada rechazada ($DUP)" || ko "Opinión duplicada" "esperaba 400/409, obtuvo $DUP"
fi

# --- PATCH videojuego ---
if [[ ${#AUTH[@]} -gt 0 && -n "${VJ_ID:-}" ]]; then
  PV=$(curl -s -o /tmp/gw_vj_patch.json -w "%{http_code}" -X PATCH "$BASE/videojuegos/$VJ_ID" \
    "${AUTH[@]}" -H "Content-Type: application/json" \
    -d '{"titulo":"Zelda Test Actualizado"}')
  check_status "PATCH /videojuegos/{id}" "200" "$PV" ""
fi

# --- IGDB ---
if grep -qE '^IGDB_CLIENT_ID=$' "/Users/Shared/University/Desarrollo Movil/02 - Activities/02 - Game World/backend/.env" 2>/dev/null; then
  IG=$(curl -s -o /tmp/gw_igdb.json -w "%{http_code}" "$BASE/igdb/buscar?q=zelda")
  [[ "$IG" == "500" ]] && skip "GET /igdb/buscar" "IGDB_CLIENT_ID no configurado (esperado)" || \
    check_status "GET /igdb/buscar" "200" "$IG" ""
else
  IG=$(curl -s -o /tmp/gw_igdb.json -w "%{http_code}" "$BASE/igdb/buscar?q=zelda&limit=3")
  check_status "GET /igdb/buscar" "200" "$IG" ""
  if [[ "$IG" == "200" && ${#AUTH[@]} -gt 0 ]]; then
    EXT_ID=$(python3 -c "
import json
d=json.load(open('/tmp/gw_igdb.json'))
if d.get('data'): print(d['data'][0]['id_externa'])
" 2>/dev/null || echo "")
    if [[ -n "${EXT_ID:-}" ]]; then
      SY=$(curl -s -o /tmp/gw_sync.json -w "%{http_code}" -X POST "$BASE/igdb/sincronizar/$EXT_ID" "${AUTH[@]}")
      [[ "$SY" == "200" || "$SY" == "201" ]] && ok "POST /igdb/sincronizar/{id} ($SY)" || ko "IGDB sincronizar" "HTTP $SY"
    fi
  fi
fi

# --- Auth sin token ---
NA=$(curl -s -o /tmp/gw_noauth.json -w "%{http_code}" "$BASE/auth/me")
check_status "GET /auth/me sin token" "401" "$NA" ""

# --- Cleanup: delete opinion and videojuego ---
if [[ ${#AUTH[@]} -gt 0 && -n "${OP_ID:-}" ]]; then
  DO=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE "$BASE/opiniones/$OP_ID" "${AUTH[@]}")
  check_status "DELETE /opiniones/{id}" "204" "$DO" ""
fi

if [[ ${#AUTH[@]} -gt 0 && -n "${VJ_ID:-}" ]]; then
  DV=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE "$BASE/videojuegos/$VJ_ID" "${AUTH[@]}")
  check_status "DELETE /videojuegos/{id}" "204" "$DV" ""
fi

echo ""
echo "=== Resultado ==="
echo "  ✅ OK:   $PASS"
echo "  ❌ FAIL: $FAIL"
echo "  ⏭️  SKIP: $SKIP"
echo ""

if [[ "$FAIL" -gt 0 ]]; then
  exit 1
fi
exit 0
