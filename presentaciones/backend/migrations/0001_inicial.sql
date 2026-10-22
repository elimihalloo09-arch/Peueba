-- Un corte es una presentación: la fecha a la que se reportan los avances.
-- Cada equipo manda su texto; se lee, se revisa y se suma al corte.

CREATE TABLE IF NOT EXISTS proyectos (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nombre     TEXT    NOT NULL,
    equipo     TEXT    NOT NULL DEFAULT '',
    orden      INTEGER NOT NULL DEFAULT 0,
    activo     BOOLEAN NOT NULL DEFAULT TRUE,
    creado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS proyectos_nombre_unico ON proyectos (lower(nombre));

CREATE TABLE IF NOT EXISTS cortes (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    fecha      DATE NOT NULL,
    titulo     TEXT NOT NULL DEFAULT 'Avance de proyectos',
    presenta   TEXT NOT NULL DEFAULT '',
    siguiente  DATE,
    creado_en  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS avances (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    corte_id     UUID NOT NULL REFERENCES cortes(id) ON DELETE CASCADE,
    proyecto_id  UUID NOT NULL REFERENCES proyectos(id) ON DELETE CASCADE,
    texto        TEXT NOT NULL,
    estado       TEXT,
    orden        INTEGER NOT NULL DEFAULT 0,
    creado_en    TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT estado_valido CHECK (estado IS NULL OR estado IN ('concluido','en_proceso','pendiente'))
);
CREATE INDEX IF NOT EXISTS avances_por_corte ON avances (corte_id, proyecto_id, orden);

CREATE TABLE IF NOT EXISTS pasos (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    corte_id     UUID NOT NULL REFERENCES cortes(id) ON DELETE CASCADE,
    proyecto_id  UUID REFERENCES proyectos(id) ON DELETE SET NULL,
    texto        TEXT NOT NULL,
    orden        INTEGER NOT NULL DEFAULT 0,
    creado_en    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS pasos_por_corte ON pasos (corte_id, orden);
