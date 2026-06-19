-- =============================================================================
-- Game World — Datos semilla iniciales
-- =============================================================================
-- Catálogo base de géneros y plataformas para filtros locales.
-- Los videojuegos se sincronizan desde IGDB cuando el usuario los consulta.
-- =============================================================================

SET search_path TO game_world;

INSERT INTO generos (nombre) VALUES
    ('Acción'),
    ('Aventura'),
    ('RPG'),
    ('Estrategia'),
    ('Simulación'),
    ('Deportes'),
    ('Carreras'),
    ('Puzzle'),
    ('Plataformas'),
    ('Shooter'),
    ('Lucha'),
    ('Terror'),
    ('Indie'),
    ('MMO'),
    ('Música')
ON CONFLICT (nombre) DO NOTHING;

INSERT INTO plataformas (nombre) VALUES
    ('PC'),
    ('PlayStation 5'),
    ('PlayStation 4'),
    ('Xbox Series X|S'),
    ('Xbox One'),
    ('Nintendo Switch'),
    ('iOS'),
    ('Android'),
    ('Mac'),
    ('Linux')
ON CONFLICT (nombre) DO NOTHING;
