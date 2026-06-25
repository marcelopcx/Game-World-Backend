# Game World — Referencia de API

Base URL local: `http://127.0.0.1:8080`

Autenticación JWT: header `Authorization: Bearer <token>`

Formato de errores:

```json
{
  "error": "descripción del error"
}
```

**Leyenda:** ✅ implementado

---

## Infraestructura

### ✅ Health Check

```http
GET /health
```

**Auth:** no

**Response `200`:** texto plano

```
Game World informa: servicios en línea.
```

---

## Autenticación y perfil

### ✅ Registro

```http
POST /auth/register
```

**Auth:** no

**Body (`application/json`):**

```json
{
  "username": "jugador1",
  "email": "jugador1@mail.com",
  "password": "mi_clave_segura"
}
```

| Campo | Tipo | Requerido | Notas |
|-------|------|-----------|-------|
| `username` | string | sí | único, máx. 50 |
| `email` | string | sí | único, máx. 100 |
| `password` | string | sí | se hashea con bcrypt |
| `nombre` | string | no | máx. 50 |
| `apellido` | string | no | máx. 50 |

> No hace falta enviar `url_avatar`. Se asigna automáticamente la imagen predeterminada (`DEFAULT_AVATAR_URL`). Para cambiarla: subir con `POST /auth/me/avatar` y persistir con `PATCH /auth/me`.

**Response `201`:**

```json
{
  "user": {
    "id_usuario": 1,
    "username": "jugador1",
    "email": "jugador1@mail.com",
    "url_avatar": "https://res.cloudinary.com/mpc-uru/image/upload/game-world/avatars/default.png",
    "es_critico": false
  }
}
```

**Errores:** `409` username/email duplicado · `400` datos inválidos

---

### ✅ Login

```http
POST /auth/login
```

**Auth:** no

**Body (`application/json`):**

```json
{
  "username": "jugador1",
  "password": "mi_clave_segura"
}
```

> También puede aceptar `email` en lugar de `username` según implementación.

**Response `200`:**

```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "user": {
    "id_usuario": 1,
    "username": "jugador1",
    "email": "jugador1@mail.com",
    "url_avatar": "https://res.cloudinary.com/mpc-uru/image/upload/game-world/avatars/default.png",
    "es_critico": false
  }
}
```

**Errores:** `401` credenciales inválidas

---

### ✅ Perfil propio

```http
GET /auth/me
```

**Auth:** sí

**Response `200`:**

```json
{
  "id_usuario": 1,
  "username": "jugador1",
  "email": "jugador1@mail.com",
  "url_avatar": "https://res.cloudinary.com/.../avatar.jpg",
  "es_critico": false,
  "fecha_registro": "2026-06-18T14:30:00Z",
  "nombre": null,
  "apellido": null
}
```

---

### ✅ Actualizar perfil propio

```http
PATCH /auth/me
```

**Auth:** sí

**Body (`application/json`):** todos los campos opcionales

```json
{
  "username": "nuevo_username",
  "email": "nuevo@mail.com",
  "password": "nueva_clave",
  "url_avatar": "https://res.cloudinary.com/.../avatar.jpg",
  "nombre": "Ana",
  "apellido": "López"
}
```

**Response `200`:** mismo shape que `GET /auth/me`

**Errores:** `409` username/email duplicado · `400` datos inválidos

---

### ✅ Eliminar cuenta propia

```http
DELETE /auth/me
```

**Auth:** sí

**Response `204`:** sin body

---

### ✅ Opiniones del usuario autenticado

```http
GET /auth/me/opiniones
```

**Auth:** sí

**Query params (opcionales):**

| Param | Tipo | Descripción |
|-------|------|-------------|
| `page` | int | página (default `1`) |
| `limit` | int | ítems por página (default `20`) |

**Response `200`:**

```json
{
  "data": [
    {
      "id_opinion": 3,
      "puntaje": 4,
      "comentario": "Muy buen juego",
      "fecha_publicacion": "2026-06-10T18:00:00Z",
      "id_videojuego": 5,
      "titulo_videojuego": "The Legend of Zelda: TOTK",
      "id_plataforma": 6,
      "nombre_plataforma": "Nintendo Switch"
    }
  ],
  "page": 1,
  "limit": 20,
  "total": 1
}
```

---

## Subida de archivos (Cloudinary)

### ✅ Subir avatar

```http
POST /auth/me/avatar
```

**Auth:** sí

**Body:** `multipart/form-data`

| Campo | Tipo | Requerido |
|-------|------|-----------|
| `file` | archivo imagen | sí |

**Límites:** máx. 10 MB · formatos imagen habituales (jpg, png, webp, etc.)

**Response `200`:**

```json
{
  "secure_url": "https://res.cloudinary.com/mpc-uru/image/upload/v.../game-world/avatars/foto.jpg"
}
```

> Persistir la URL con `PATCH /auth/me` en el campo `url_avatar`.

**Errores:** `400` archivo vacío o muy grande · `401` no autenticado · `500` error Cloudinary

---

### ✅ Subir carátula de videojuego

```http
POST /videojuegos/imagen
```

**Auth:** sí

**Body:** `multipart/form-data`

| Campo | Tipo | Requerido |
|-------|------|-----------|
| `file` | archivo imagen | sí |

**Response `200`:**

```json
{
  "secure_url": "https://res.cloudinary.com/mpc-uru/image/upload/v.../game-world/caratulas/caratula.jpg"
}
```

> Persistir la URL al crear/actualizar un videojuego en `url_caratula`.

---

## Usuarios (CRUD)

### ✅ Listar usuarios

```http
GET /usuarios
```

**Auth:** sí

**Query params (opcionales):**

| Param | Tipo | Descripción |
|-------|------|-------------|
| `q` | string | búsqueda por username |
| `es_critico` | bool | filtrar críticos (`true`/`false`) |
| `page` | int | página |
| `limit` | int | ítems por página |
| `sort` | string | `username` · `fecha_registro` |
| `order` | string | `asc` · `desc` |

**Response `200`:**

```json
{
  "data": [
    {
      "id_usuario": 2,
      "username": "critico1",
      "email": "critico@mail.com",
      "url_avatar": "https://res.cloudinary.com/mpc-uru/image/upload/game-world/avatars/default.png",
      "es_critico": true,
      "fecha_registro": "2026-06-01T10:00:00Z",
      "nombre": "Carlos",
      "apellido": "Reviewer"
    }
  ],
  "page": 1,
  "limit": 20,
  "total": 1
}
```

---

### ✅ Detalle de usuario

```http
GET /usuarios/{id}
```

**Auth:** no (perfil público)

**Response `200`:**

```json
{
  "id_usuario": 2,
  "username": "critico1",
  "url_avatar": "https://res.cloudinary.com/.../avatar.jpg",
  "es_critico": true,
  "nombre": "Carlos",
  "apellido": "Reviewer"
}
```

**Errores:** `404` no encontrado

---

### ✅ Crear usuario

```http
POST /usuarios
```

**Auth:** sí

**Body:** igual que `POST /auth/register`, con campos opcionales adicionales:

```json
{
  "username": "critico1",
  "email": "critico@mail.com",
  "password": "clave123",
  "es_critico": true,
  "url_avatar": null
}
```

**Response `201`:** objeto usuario creado

---

### ✅ Actualizar usuario

```http
PATCH /usuarios/{id}
```

**Auth:** sí

**Body:** mismos campos opcionales que `PATCH /auth/me` + `es_critico`

```json
{
  "es_critico": true,
  "url_avatar": "https://res.cloudinary.com/.../avatar.jpg"
}
```

**Response `200`:** usuario actualizado

**Errores:** `403` sin permiso · `404` no encontrado · `409` conflicto unique

---

### ✅ Eliminar usuario

```http
DELETE /usuarios/{id}
```

**Auth:** sí

**Response `204`:** sin body

---

## Catálogos

### ✅ Listar géneros

```http
GET /generos
```

**Auth:** no

**Response `200`:**

```json
[
  { "id_genero": 1, "nombre": "Acción" },
  { "id_genero": 2, "nombre": "Aventura" },
  { "id_genero": 3, "nombre": "RPG" }
]
```

---

### ✅ Listar plataformas

```http
GET /plataformas
```

**Auth:** no

**Response `200`:**

```json
[
  { "id_plataforma": 1, "nombre": "PC" },
  { "id_plataforma": 6, "nombre": "Nintendo Switch" }
]
```

---

## Videojuegos

### ✅ Listar videojuegos (filtros + sorting)

```http
GET /videojuegos
```

**Auth:** no

**Query params (opcionales):**

| Param | Tipo | Descripción |
|-------|------|-------------|
| `q` | string | búsqueda por título |
| `genero` | int | id de género |
| `plataforma` | int | id de plataforma |
| `fecha_desde` | date | lanzamiento desde (`YYYY-MM-DD`) |
| `fecha_hasta` | date | lanzamiento hasta |
| `puntaje_min` | float | puntaje mínimo (1–5) |
| `tipo_puntaje` | string | `usuarios` (default) · `criticos` |
| `sort` | string | `titulo` · `fecha` · `puntaje_usuarios` · `puntaje_criticos` |
| `order` | string | `asc` · `desc` |
| `page` | int | página |
| `limit` | int | ítems por página |

**Ejemplo:**

```http
GET /videojuegos?q=zelda&genero=2&sort=puntaje_usuarios&order=desc&page=1&limit=10
```

**Response `200`:**

```json
{
  "data": [
    {
      "id_videojuego": 5,
      "id_externa": "119171",
      "titulo": "The Legend of Zelda: Tears of the Kingdom",
      "sinopsis": "En este sequel...",
      "fecha_lanzamiento": "2023-05-12",
      "url_caratula": "https://images.igdb.com/.../cover_big.jpg",
      "desarrollador": "Nintendo EPD",
      "editor": "Nintendo",
      "promedio_puntaje_usuarios": 4.50,
      "promedio_puntaje_criticos": 4.80,
      "cantidad_opiniones_usuarios": 12,
      "cantidad_opiniones_criticos": 3,
      "generos": [
        { "id_genero": 2, "nombre": "Aventura" }
      ],
      "plataformas": [
        { "id_plataforma": 6, "nombre": "Nintendo Switch" }
      ]
    }
  ],
  "page": 1,
  "limit": 10,
  "total": 1
}
```

---

### ✅ Detalle de videojuego

```http
GET /videojuegos/{id}
```

**Auth:** no

**Response `200`:**

```json
{
  "id_videojuego": 5,
  "id_externa": "119171",
  "titulo": "The Legend of Zelda: Tears of the Kingdom",
  "sinopsis": "En este sequel...",
  "fecha_lanzamiento": "2023-05-12",
  "url_caratula": "https://images.igdb.com/.../cover_big.jpg",
  "desarrollador": "Nintendo EPD",
  "editor": "Nintendo",
  "promedio_puntaje_usuarios": 4.50,
  "promedio_puntaje_criticos": 4.80,
  "cantidad_opiniones_usuarios": 12,
  "cantidad_opiniones_criticos": 3,
  "generos": [
    { "id_genero": 2, "nombre": "Aventura" },
    { "id_genero": 3, "nombre": "RPG" }
  ],
  "plataformas": [
    { "id_plataforma": 6, "nombre": "Nintendo Switch" }
  ]
}
```

**Errores:** `404` no encontrado

---

### ✅ Crear videojuego

```http
POST /videojuegos
```

**Auth:** sí

**Body (`application/json`):**

```json
{
  "id_externa": "119171",
  "titulo": "The Legend of Zelda: Tears of the Kingdom",
  "sinopsis": "En este sequel...",
  "fecha_lanzamiento": "2023-05-12",
  "url_caratula": "https://res.cloudinary.com/.../caratula.jpg",
  "desarrollador": "Nintendo EPD",
  "editor": "Nintendo",
  "generos": [2, 3],
  "plataformas": [6]
}
```

| Campo | Tipo | Requerido |
|-------|------|-----------|
| `titulo` | string | sí |
| `id_externa` | string | no |
| `sinopsis` | string | no |
| `fecha_lanzamiento` | date | no |
| `url_caratula` | string | no |
| `desarrollador` | string | no |
| `editor` | string | no |
| `generos` | int[] | no |
| `plataformas` | int[] | no |

**Response `201`:** objeto videojuego creado

**Errores:** `409` `id_externa` duplicada

---

### ✅ Actualizar videojuego

```http
PATCH /videojuegos/{id}
```

**Auth:** sí

**Body:** mismos campos que POST, todos opcionales

```json
{
  "titulo": "Zelda TOTK",
  "url_caratula": "https://res.cloudinary.com/.../nueva.jpg",
  "generos": [2, 3, 9],
  "plataformas": [6]
}
```

**Response `200`:** videojuego actualizado

---

### ✅ Eliminar videojuego

```http
DELETE /videojuegos/{id}
```

**Auth:** sí

**Response `204`:** sin body (cascade a opiniones y tablas N:M)

---

## Opiniones (comentarios + puntaje)

### ✅ Listar opiniones de un videojuego

```http
GET /videojuegos/{id}/opiniones
```

**Auth:** no

**Query params (opcionales):**

| Param | Tipo | Descripción |
|-------|------|-------------|
| `es_critico` | bool | filtrar por tipo de usuario |
| `sort` | string | `fecha` · `puntaje` |
| `order` | string | `asc` · `desc` |
| `page` | int | página |
| `limit` | int | ítems por página |

**Response `200`:**

```json
{
  "data": [
    {
      "id_opinion": 10,
      "puntaje": 5,
      "comentario": "Obra maestra absoluta.",
      "fecha_publicacion": "2026-06-15T20:00:00Z",
      "usuario": {
        "id_usuario": 2,
        "username": "critico1",
        "url_avatar": "https://res.cloudinary.com/mpc-uru/image/upload/game-world/avatars/default.png",
        "es_critico": true
      },
      "plataforma": {
        "id_plataforma": 6,
        "nombre": "Nintendo Switch"
      }
    }
  ],
  "page": 1,
  "limit": 20,
  "total": 1
}
```

---

### ✅ Resumen de puntajes de un videojuego

```http
GET /videojuegos/{id}/opiniones/resumen
```

**Auth:** no

**Response `200`:**

```json
{
  "id_videojuego": 5,
  "promedio_puntaje_usuarios": 4.50,
  "promedio_puntaje_criticos": 4.80,
  "cantidad_opiniones_usuarios": 12,
  "cantidad_opiniones_criticos": 3
}
```

> Estos campos también vienen en `GET /videojuegos/{id}`.

---

### ✅ Detalle de opinión

```http
GET /opiniones/{id}
```

**Auth:** no

**Response `200`:**

```json
{
  "id_opinion": 10,
  "puntaje": 5,
  "comentario": "Obra maestra absoluta.",
  "fecha_publicacion": "2026-06-15T20:00:00Z",
  "id_usuario": 2,
  "id_videojuego": 5,
  "id_plataforma": 6
}
```

---

### ✅ Crear opinión

```http
POST /videojuegos/{id}/opiniones
```

**Auth:** sí

**Body (`application/json`):**

```json
{
  "puntaje": 4,
  "comentario": "Muy buen juego, mundo abierto increíble.",
  "id_plataforma": 6
}
```

| Campo | Tipo | Requerido | Notas |
|-------|------|-----------|-------|
| `puntaje` | int | sí | entre 1 y 5 |
| `comentario` | string | sí | |
| `id_plataforma` | int | no | plataforma en la que jugó |

**Response `201`:**

```json
{
  "id_opinion": 11,
  "puntaje": 4,
  "comentario": "Muy buen juego, mundo abierto increíble.",
  "fecha_publicacion": "2026-06-18T22:00:00Z",
  "id_usuario": 1,
  "id_videojuego": 5,
  "id_plataforma": 6
}
```

**Errores:** `409` el usuario ya tiene opinión en ese juego · `400` puntaje fuera de rango

---

### ✅ Actualizar opinión

```http
PATCH /opiniones/{id}
```

**Auth:** sí (solo el autor)

**Body (`application/json`):**

```json
{
  "puntaje": 5,
  "comentario": "Actualicé mi reseña: es un 10/10.",
  "id_plataforma": 6
}
```

**Response `200`:** opinión actualizada

**Errores:** `403` no es el autor · `404` no encontrada

---

### ✅ Eliminar opinión

```http
DELETE /opiniones/{id}
```

**Auth:** sí (solo el autor)

**Response `204`:** sin body

---

## IGDB (API externa)

Credenciales IGDB se configuran en el backend (no expuestas al cliente móvil).

Variables de entorno planificadas:

```env
IGDB_CLIENT_ID=<tu_client_id>
IGDB_CLIENT_SECRET=<tu_client_secret>
```

### ✅ Buscar en IGDB

```http
GET /igdb/buscar
```

**Auth:** no / sí *(según implementación)*

**Query params:**

| Param | Tipo | Requerido | Descripción |
|-------|------|-----------|-------------|
| `q` | string | sí | término de búsqueda |
| `limit` | int | no | resultados (default `10`) |

**Ejemplo:**

```http
GET /igdb/buscar?q=zelda&limit=5
```

**Response `200`:**

```json
{
  "data": [
    {
      "id_externa": "119171",
      "titulo": "The Legend of Zelda: Tears of the Kingdom",
      "sinopsis": "...",
      "fecha_lanzamiento": "2023-05-12",
      "url_caratula": "https://images.igdb.com/igdb/image/upload/...",
      "desarrollador": "Nintendo EPD",
      "editor": "Nintendo",
      "generos": ["Aventura", "RPG"],
      "plataformas": ["Nintendo Switch"]
    }
  ]
}
```

---

### ✅ Detalle desde IGDB

```http
GET /igdb/juegos/{id_externa}
```

**Auth:** no / sí

**Response `200`:** mismo shape que un ítem de búsqueda

**Errores:** `404` juego no encontrado en IGDB

---

### ✅ Sincronizar juego a BD local

```http
POST /igdb/sincronizar/{id_externa}
```

**Auth:** sí

**Body:** vacío o `{}`

**Comportamiento:**

1. Consulta IGDB por `id_externa`
2. Si no existe en BD local → crea `videojuego` + relaciones género/plataforma
3. Si ya existe → devuelve el registro local

**Response `200` / `201`:**

```json
{
  "id_videojuego": 5,
  "id_externa": "119171",
  "titulo": "The Legend of Zelda: Tears of the Kingdom",
  "sinopsis": "...",
  "fecha_lanzamiento": "2023-05-12",
  "url_caratula": "https://images.igdb.com/...",
  "desarrollador": "Nintendo EPD",
  "editor": "Nintendo",
  "promedio_puntaje_usuarios": null,
  "promedio_puntaje_criticos": null,
  "cantidad_opiniones_usuarios": 0,
  "cantidad_opiniones_criticos": 0,
  "generos": [{ "id_genero": 2, "nombre": "Aventura" }],
  "plataformas": [{ "id_plataforma": 6, "nombre": "Nintendo Switch" }],
  "sincronizado": true
}
```

---

## Códigos HTTP comunes

| Código | Significado |
|--------|-------------|
| `200` | OK |
| `201` | Creado |
| `204` | Sin contenido (delete exitoso) |
| `400` | Solicitud inválida |
| `401` | No autenticado / credenciales inválidas |
| `403` | Sin permiso |
| `404` | Recurso no encontrado |
| `409` | Conflicto (unique constraint) |
| `500` | Error interno |

---

## Resumen rápido

| Módulo | Endpoints |
|--------|-----------|
| Infra | `GET /health` ✅ |
| Auth | `POST /auth/register` · `POST /auth/login` · `GET/PATCH/DELETE /auth/me` · `GET /auth/me/opiniones` ✅ |
| Uploads | `POST /auth/me/avatar` · `POST /videojuegos/imagen` ✅ |
| Usuarios | `GET/POST /usuarios` · `GET/PATCH/DELETE /usuarios/{id}` ✅ |
| Catálogos | `GET /generos` · `GET /plataformas` ✅ |
| Videojuegos | `GET/POST /videojuegos` · `GET/PATCH/DELETE /videojuegos/{id}` ✅ |
| Opiniones | `GET/POST /videojuegos/{id}/opiniones` · `GET /videojuegos/{id}/opiniones/resumen` · `GET/PATCH/DELETE /opiniones/{id}` ✅ |
| IGDB | `GET /igdb/buscar` · `GET /igdb/juegos/{id}` · `POST /igdb/sincronizar/{id}` ✅ |
