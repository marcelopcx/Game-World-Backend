-- =============================================================================
-- Game World — Esquema de base de datos (PostgreSQL)
-- =============================================================================
-- Reglas de negocio reflejadas en el DDL:
--   • Username y email únicos por usuario.
--   • Un usuario solo puede dejar una opinión por videojuego.
--   • Puntaje de opinión estrictamente entre 1 y 5.
--   • Sincronización con IGDB mediante id_externa única.
--   • Promedios separados para usuarios normales y críticos (es_critico).
--   • Búsqueda y ordenamiento por título, fecha, género y puntaje.
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS game_world;
SET search_path TO game_world;

-- -----------------------------------------------------------------------------
-- 1. Usuarios (credenciales, perfil, roles y avatar)
-- -----------------------------------------------------------------------------
CREATE TABLE usuarios (
    id_usuario SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    nombre VARCHAR(50),
    apellido VARCHAR(50),
    url_avatar VARCHAR(255),
    fecha_registro TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    es_critico BOOLEAN NOT NULL DEFAULT FALSE
);

-- -----------------------------------------------------------------------------
-- 2. Catálogo: géneros y plataformas
-- -----------------------------------------------------------------------------
CREATE TABLE generos (
    id_genero SERIAL PRIMARY KEY,
    nombre VARCHAR(50) NOT NULL UNIQUE
);

CREATE TABLE plataformas (
    id_plataforma SERIAL PRIMARY KEY,
    nombre VARCHAR(50) NOT NULL UNIQUE
);

-- -----------------------------------------------------------------------------
-- 3. Videojuegos (datos locales + sincronización IGDB)
-- -----------------------------------------------------------------------------
CREATE TABLE videojuegos (
    id_videojuego SERIAL PRIMARY KEY,
    id_externa VARCHAR(100) UNIQUE,
    titulo VARCHAR(150) NOT NULL,
    sinopsis TEXT,
    fecha_lanzamiento DATE,
    url_caratula VARCHAR(255),
    desarrollador VARCHAR(100),
    editor VARCHAR(100),
    promedio_puntaje_usuarios DECIMAL(3, 2) CHECK (
        promedio_puntaje_usuarios IS NULL
        OR (promedio_puntaje_usuarios >= 1 AND promedio_puntaje_usuarios <= 5)
    ),
    promedio_puntaje_criticos DECIMAL(3, 2) CHECK (
        promedio_puntaje_criticos IS NULL
        OR (promedio_puntaje_criticos >= 1 AND promedio_puntaje_criticos <= 5)
    ),
    cantidad_opiniones_usuarios INT NOT NULL DEFAULT 0 CHECK (cantidad_opiniones_usuarios >= 0),
    cantidad_opiniones_criticos INT NOT NULL DEFAULT 0 CHECK (cantidad_opiniones_criticos >= 0)
);

-- -----------------------------------------------------------------------------
-- 4. Relaciones N:M
-- -----------------------------------------------------------------------------
CREATE TABLE videojuegos_generos (
    id_videojuegos_genero SERIAL PRIMARY KEY,
    id_videojuego INT NOT NULL,
    id_genero INT NOT NULL,
    CONSTRAINT fk_vg_videojuego FOREIGN KEY (id_videojuego)
        REFERENCES videojuegos(id_videojuego) ON DELETE CASCADE,
    CONSTRAINT fk_vg_genero FOREIGN KEY (id_genero)
        REFERENCES generos(id_genero) ON DELETE CASCADE,
    CONSTRAINT uq_videojuego_genero UNIQUE (id_videojuego, id_genero)
);

CREATE TABLE videojuegos_plataformas (
    id_videojuegos_plataforma SERIAL PRIMARY KEY,
    id_videojuego INT NOT NULL,
    id_plataforma INT NOT NULL,
    CONSTRAINT fk_vp_videojuego FOREIGN KEY (id_videojuego)
        REFERENCES videojuegos(id_videojuego) ON DELETE CASCADE,
    CONSTRAINT fk_vp_plataforma FOREIGN KEY (id_plataforma)
        REFERENCES plataformas(id_plataforma) ON DELETE CASCADE,
    CONSTRAINT uq_videojuego_plataforma UNIQUE (id_videojuego, id_plataforma)
);

-- -----------------------------------------------------------------------------
-- 5. Opiniones (comentarios + puntaje)
-- -----------------------------------------------------------------------------
CREATE TABLE opinion (
    id_opinion SERIAL PRIMARY KEY,
    puntaje INT NOT NULL,
    comentario TEXT NOT NULL,
    fecha_publicacion TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    id_usuario INT NOT NULL,
    id_videojuego INT NOT NULL,
    id_plataforma INT,
    CONSTRAINT chk_puntaje CHECK (puntaje BETWEEN 1 AND 5),
    CONSTRAINT fk_opinion_usuario FOREIGN KEY (id_usuario)
        REFERENCES usuarios(id_usuario) ON DELETE CASCADE,
    CONSTRAINT fk_opinion_videojuego FOREIGN KEY (id_videojuego)
        REFERENCES videojuegos(id_videojuego) ON DELETE CASCADE,
    CONSTRAINT fk_opinion_plataforma FOREIGN KEY (id_plataforma)
        REFERENCES plataformas(id_plataforma) ON DELETE SET NULL,
    CONSTRAINT uq_usuario_videojuego_review UNIQUE (id_usuario, id_videojuego)
);

-- -----------------------------------------------------------------------------
-- 6. Índices (búsqueda, filtros y ordenamiento)
-- -----------------------------------------------------------------------------
CREATE INDEX idx_usuario_username_lower ON usuarios (LOWER(username));
CREATE INDEX idx_usuario_nombre_lower ON usuarios (LOWER(nombre));
CREATE INDEX idx_usuario_apellido_lower ON usuarios (LOWER(apellido));
CREATE INDEX idx_usuario_email_lower ON usuarios (LOWER(email));
CREATE INDEX idx_usuario_es_critico ON usuarios (es_critico);

CREATE INDEX idx_genero_nombre_lower ON generos (LOWER(nombre));
CREATE INDEX idx_plataforma_nombre_lower ON plataformas (LOWER(nombre));

CREATE INDEX idx_videojuego_titulo_lower ON videojuegos (LOWER(titulo));
CREATE INDEX idx_videojuego_fecha_lanzamiento ON videojuegos (fecha_lanzamiento);
CREATE INDEX idx_videojuego_id_externa ON videojuegos (id_externa);
CREATE INDEX idx_videojuego_promedio_usuarios ON videojuegos (promedio_puntaje_usuarios);
CREATE INDEX idx_videojuego_promedio_criticos ON videojuegos (promedio_puntaje_criticos);
CREATE INDEX idx_videojuego_desarrollador_lower ON videojuegos (LOWER(desarrollador));

CREATE INDEX idx_vg_genero_genero ON videojuegos_generos (id_genero);
CREATE INDEX idx_vg_genero_videojuego ON videojuegos_generos (id_videojuego);
CREATE INDEX idx_vp_plataforma_plataforma ON videojuegos_plataformas (id_plataforma);
CREATE INDEX idx_vp_plataforma_videojuego ON videojuegos_plataformas (id_videojuego);

CREATE INDEX idx_opinion_videojuego ON opinion (id_videojuego);
CREATE INDEX idx_opinion_usuario ON opinion (id_usuario);
CREATE INDEX idx_opinion_fecha_publicacion ON opinion (fecha_publicacion);
CREATE INDEX idx_opinion_puntaje ON opinion (puntaje);

-- -----------------------------------------------------------------------------
-- 7. Funciones y triggers
-- -----------------------------------------------------------------------------

-- Mantiene promedios separados (usuarios normales vs críticos) por videojuego.
CREATE OR REPLACE FUNCTION fn_actualizar_promedios_videojuego()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_id_videojuego INT;
BEGIN
    v_id_videojuego := COALESCE(NEW.id_videojuego, OLD.id_videojuego);

    UPDATE videojuegos
    SET
        promedio_puntaje_usuarios = sub.avg_usuarios,
        promedio_puntaje_criticos = sub.avg_criticos,
        cantidad_opiniones_usuarios = sub.count_usuarios,
        cantidad_opiniones_criticos = sub.count_criticos
    FROM (
        SELECT
            ROUND(AVG(o.puntaje) FILTER (WHERE u.es_critico = FALSE)::numeric, 2) AS avg_usuarios,
            ROUND(AVG(o.puntaje) FILTER (WHERE u.es_critico = TRUE)::numeric, 2) AS avg_criticos,
            COUNT(*) FILTER (WHERE u.es_critico = FALSE) AS count_usuarios,
            COUNT(*) FILTER (WHERE u.es_critico = TRUE) AS count_criticos
        FROM opinion o
        INNER JOIN usuarios u ON u.id_usuario = o.id_usuario
        WHERE o.id_videojuego = v_id_videojuego
    ) AS sub
    WHERE videojuegos.id_videojuego = v_id_videojuego;

    RETURN COALESCE(NEW, OLD);
END;
$$;

CREATE TRIGGER trg_opinion_actualiza_promedios
    AFTER INSERT OR UPDATE OR DELETE ON opinion
    FOR EACH ROW
    EXECUTE PROCEDURE fn_actualizar_promedios_videojuego();
