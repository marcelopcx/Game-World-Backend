-- Migración: eliminar tabla personas; nombre y apellido pasan a usuarios
SET search_path TO game_world;

ALTER TABLE usuarios ADD COLUMN IF NOT EXISTS nombre VARCHAR(50);
ALTER TABLE usuarios ADD COLUMN IF NOT EXISTS apellido VARCHAR(50);

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'game_world'
          AND table_name = 'usuarios'
          AND column_name = 'id_persona'
    ) AND EXISTS (
        SELECT 1 FROM information_schema.tables
        WHERE table_schema = 'game_world' AND table_name = 'personas'
    ) THEN
        UPDATE usuarios u
        SET
            nombre = p.nombre,
            apellido = p.apellido
        FROM personas p
        WHERE u.id_persona = p.id_persona;
    END IF;
END $$;

DROP TRIGGER IF EXISTS trg_usuario_borra_persona ON usuarios;
DROP FUNCTION IF EXISTS fn_borrar_persona_al_eliminar_usuario();

ALTER TABLE usuarios DROP CONSTRAINT IF EXISTS fk_usuario_persona;
ALTER TABLE usuarios DROP CONSTRAINT IF EXISTS usuarios_id_persona_key;
ALTER TABLE usuarios DROP COLUMN IF EXISTS id_persona;

DROP INDEX IF EXISTS idx_persona_nombre_lower;
DROP INDEX IF EXISTS idx_persona_apellido_lower;
DROP TABLE IF EXISTS personas;

CREATE INDEX IF NOT EXISTS idx_usuario_nombre_lower ON usuarios (LOWER(nombre));
CREATE INDEX IF NOT EXISTS idx_usuario_apellido_lower ON usuarios (LOWER(apellido));
