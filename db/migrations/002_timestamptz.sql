SET search_path TO game_world, public;

ALTER TABLE usuarios
    ALTER COLUMN fecha_registro TYPE TIMESTAMPTZ
    USING fecha_registro AT TIME ZONE 'UTC';

ALTER TABLE opinion
    ALTER COLUMN fecha_publicacion TYPE TIMESTAMPTZ
    USING fecha_publicacion AT TIME ZONE 'UTC';
