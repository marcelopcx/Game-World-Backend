# Esquema SQL — Game World

| Archivo | Descripción |
|---------|-------------|
| `game_world.sql` | Esquema completo: tablas, índices y triggers |
| `seed_data.sql` | Géneros y plataformas iniciales para filtros locales |

## Tablas principales

- `usuarios` — Credenciales, username, avatar y flag `es_critico` (nombre/apellido opcionales)
- `generos` / `plataformas` — Catálogos para filtros
- `videojuegos` — Juegos locales sincronizados desde IGDB (`id_externa`)
- `videojuegos_generos` / `videojuegos_plataformas` — Relaciones N:M
- `opinion` — Comentarios y puntajes (1 reseña por usuario y juego)

## Promedios de puntaje

La tabla `videojuegos` mantiene automáticamente (vía trigger):

- `promedio_puntaje_usuarios` / `cantidad_opiniones_usuarios`
- `promedio_puntaje_criticos` / `cantidad_opiniones_criticos`

Esto permite diferenciar el rating de usuarios normales y críticos en la app.

## Aplicar manualmente

```bash
docker compose exec -T db psql -U gameworld -d game_world < db/game_world.sql
docker compose exec -T db psql -U gameworld -d game_world < db/seed_data.sql
```

O con el script automatizado:

```bash
make setup
```
