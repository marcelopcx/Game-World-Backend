-- Migración: eliminar genero y fecha_nacimiento de personas (si aún existe la tabla)
SET search_path TO game_world;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.tables
        WHERE table_schema = 'game_world' AND table_name = 'personas'
    ) THEN
        ALTER TABLE personas DROP COLUMN IF EXISTS genero;
        ALTER TABLE personas DROP COLUMN IF EXISTS fecha_nacimiento;
    END IF;
END $$;
