-- Migración: nombre y apellido opcionales en usuarios
SET search_path TO game_world;

ALTER TABLE usuarios ALTER COLUMN nombre DROP NOT NULL;
ALTER TABLE usuarios ALTER COLUMN apellido DROP NOT NULL;
