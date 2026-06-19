# Game World (Backend)

API REST del proyecto **Game World**, desarrollado para el curso de **Desarrollo de Aplicaciones Móviles** en la **Universidad Rafael Urdaneta (URU)**.

Este repositorio contiene la estructura base del backend, tomada del proyecto **Nook's Cookbook**, lista para que agregues el modelo de datos y la lógica de negocio de Game World.

Stack:

- **Rust** con **[Actix Web](https://actix.rs/)**
- **PostgreSQL** con **[SQLx](https://github.com/launchbadge/sqlx)**
- Autenticación **JWT** (Bearer) y contraseñas con **bcrypt** (infraestructura preparada)

El cliente móvil está en el repositorio del frontend dentro de `02 - Game World/frontend` (Ionic + Angular).

---

## Guía de inicialización

### Prerrequisitos

1. **[Rust](https://www.rust-lang.org/tools/install)** (toolchain *stable*).
2. **[Docker](https://www.docker.com/)** y **Docker Compose**.

### Pasos

1. **Entrá al directorio del backend:**
   ```bash
   cd backend
   ```

2. **Dale permisos de ejecución a los scripts** *(solo la primera vez)*:
   ```bash
   chmod +x scripts/*.sh
   ```

3. **Inicializá el entorno** (`.env` y PostgreSQL con Docker):
   ```bash
   make setup
   ```

4. **Revisá las variables de entorno** en `.env` si hace falta:
   ```env
   DATABASE_URL=postgres://gameworld:secret123@127.0.0.1:5432/game_world
   JWT_SECRET=un_secreto_largo_minimo_32_caracteres_cambiar_en_produccion
   JWT_EXPIRATION_HOURS=24
   HOST=0.0.0.0
   PORT=8080
   ```

5. **Iniciá el servidor:**
   ```bash
   cargo run
   ```

6. **Comprobá el health check:**
   ```bash
   curl http://127.0.0.1:8080/health
   ```

### Alternativa sin Make

```bash
./scripts/setup.sh
cargo run
```

---

## Arquitectura de carpetas

- `src/main.rs` — Punto de entrada: pool de conexiones, configuración y arranque de **HttpServer**.
- `src/lib.rs` — Módulos públicos del crate.
- `src/config/` — Carga de variables de entorno (`AppConfig`).
- `src/db/` — Creación del pool de PostgreSQL (`search_path` → `game_world`).
- `src/auth/` — Extractores `AuthenticatedUser` y `OptionalAuthenticatedUser` (JWT).
- `src/handlers/` — Controladores HTTP (por ahora solo `health`).
- `src/services/` — Lógica de negocio (por ahora solo utilidades JWT en `auth`).
- `src/models/` — Structs de request/response y filas de BD *(vacío, listo para completar)*.
- `src/routes/` — Registro de servicios Actix (`configure`).
- `src/error/` — Errores de API unificados (`ApiError`).
- `db/` — Esquema SQL y datos semilla *(pendiente de definir)*.
- `scripts/` — `setup.sh` y `reset-db.sh`.
- `api/` — Colección [Bruno](https://www.usebruno.com/) para probar endpoints.
- `docker-compose.yml` — PostgreSQL 16 en contenedor.
- `build.rs` — Carga `.env` al compilar; activa **SQLx offline** si existe `.sqlx/`.

---

## Próximos pasos sugeridos

1. Definir el esquema SQL en `db/` (schema `game_world`).
2. Agregar modelos en `src/models/` y servicios en `src/services/`.
3. Registrar handlers y rutas en `src/handlers/` y `src/routes/`.
4. Regenerar caché SQLx con `make sqlx-prepare` cuando uses macros `query!`.
