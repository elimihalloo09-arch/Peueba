-- Todo el dinero se guarda en CENTAVOS (bigint). Nunca en punto flotante.

CREATE TABLE IF NOT EXISTS acreedores (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nombre          TEXT        NOT NULL,
    saldo_original  BIGINT      NOT NULL DEFAULT 0,
    monto_a_pagar   BIGINT      NOT NULL DEFAULT 0,
    tasa_anual      DOUBLE PRECISION NOT NULL DEFAULT 0,
    pago_mensual    BIGINT      NOT NULL DEFAULT 0,
    etapa           TEXT        NOT NULL DEFAULT 'sin_contactar',
    admite_quita    BOOLEAN     NOT NULL DEFAULT TRUE,
    por_nomina      BOOLEAN     NOT NULL DEFAULT FALSE,
    tiene_convenio     BOOLEAN  NOT NULL DEFAULT FALSE,
    tiene_comprobante  BOOLEAN  NOT NULL DEFAULT FALSE,
    tiene_finiquito    BOOLEAN  NOT NULL DEFAULT FALSE,
    nota            TEXT        NOT NULL DEFAULT '',
    creado_en       TIMESTAMPTZ NOT NULL DEFAULT now(),
    actualizado_en  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT etapa_valida CHECK (etapa IN
        ('sin_contactar','contactado','con_oferta','convenio','pagado','finiquito'))
);

CREATE TABLE IF NOT EXISTS pagos (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    acreedor_id   UUID REFERENCES acreedores(id) ON DELETE SET NULL,
    concepto      TEXT   NOT NULL,
    monto         BIGINT NOT NULL,
    fecha         DATE   NOT NULL DEFAULT CURRENT_DATE,
    creado_en     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS pendientes (
    id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    texto     TEXT    NOT NULL,
    vence     DATE,
    hecho     BOOLEAN NOT NULL DEFAULT FALSE,
    creado_en TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Préstamos de personas: entregas que me hacen y pagos que les hago.
CREATE TABLE IF NOT EXISTS movimientos_personales (
    id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    persona   TEXT   NOT NULL,
    concepto  TEXT   NOT NULL,
    monto     BIGINT NOT NULL,
    tipo      TEXT   NOT NULL,
    fecha     DATE   NOT NULL DEFAULT CURRENT_DATE,
    CONSTRAINT tipo_valido CHECK (tipo IN ('entrega','pago'))
);

CREATE TABLE IF NOT EXISTS ingresos (
    id       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    concepto TEXT   NOT NULL,
    neto_mensual BIGINT NOT NULL,
    activo   BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS gastos (
    id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    concepto  TEXT   NOT NULL,
    categoria TEXT   NOT NULL DEFAULT 'Otros',
    mensual   BIGINT NOT NULL,
    recortable BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_pagos_acreedor ON pagos(acreedor_id);
CREATE INDEX IF NOT EXISTS idx_mov_persona ON movimientos_personales(persona);
