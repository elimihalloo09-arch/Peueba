-- Datos reales de Carlos, en centavos. Solo se insertan si la tabla está vacía,
-- así volver a correr las migraciones nunca duplica nada.

INSERT INTO acreedores (nombre, saldo_original, monto_a_pagar, tasa_anual, pago_mensual, etapa, admite_quita, por_nomina, nota)
SELECT * FROM (VALUES
  ('Tenencia y refrendo Morelos', 1000000::bigint, 1000000::bigint, 0::double precision, 0::bigint, 'sin_contactar', false, false,
   'Vence el 23 de septiembre. Sin esto no se puede dar de baja las placas del CUPRA entregado.'),
  ('HeyBanco · tarjeta', 1371900, 411600, 70, 107200, 'con_oferta', true, false,
   'Nancy Nelly Ávila Almaguer, nancy.avila@hey.inc. Ofreció por escrito quita del 70% el 15 de septiembre. Carta finiquito a 30 días hábiles del pago.'),
  ('Klar · tarjeta', 1342400, 469800, 70, 0, 'contactado', true, false,
   '207 días de atraso, saldo vencido de $14,610.88. Avisaron cesión a cobranza externa: negociar antes.'),
  ('NU · tarjeta', 2350500, 822700, 70, 0, 'sin_contactar', true, false,
   'En cobranza, atraso de 150 días a 12 meses. Pedir quita del 80%.'),
  ('Coppel · préstamo', 1825300, 638900, 70, 0, 'sin_contactar', true, false,
   'Más de 12 meses de atraso: el mayor margen de quita. Es la grande, y no tiene nada que ver con BanCoppel ni con Azteca.'),
  ('Fintopia · préstamo', 1071500, 375000, 90, 491000, 'sin_contactar', true, false,
   'Más de 12 meses de atraso y la tasa más alta de todas.'),
  ('NU · préstamo personal', 118400, 41400, 80, 59000, 'sin_contactar', true, false,
   'El más chico. Se cierra rápido.'),
  ('BanCoppel · tarjeta', 513269, 513269, 69.4, 130633, 'sin_contactar', false, false,
   'Estado de cuenta al 20-ago-2026: saldo $5,132.69, mínimo $1,306.33, fecha límite INMEDIATO. Ya cobra moratorios al 74.4% sobre $472.50. Pagando el mínimo se liquida en 5 meses con $606 de intereses. CAT 98.7%.'),
  ('Telcel · línea', 60800, 60800, 0, 60800, 'sin_contactar', false, false,
   'Lo más chico de todo. Pagarlo y olvidarlo.'),
  ('Banco Azteca · préstamo de $10,500', 1033000, 1033000, 128, 79733, 'sin_contactar', false, false,
   'Semanal de $184 en la app ($195 en sucursal). Dispuesto el 4-feb-2026 a 154 semanas; llevas 30. Liquidarlo hoy cuesta $10,330 contra $22,816 de seguir pagando: ahorras $12,486.'),
  ('Banco Azteca · préstamo de $5,000', 504100, 504100, 137, 41167, 'sin_contactar', false, false,
   'Semanal de $95 en la app ($100 en sucursal). Dispuesto el 10-jun-2026 a 128 semanas; llevas 12. Liquidarlo hoy cuesta $5,041 contra $11,020 de seguir pagando: ahorras $5,979.'),
  ('Banco Azteca · tercer crédito', 1449500, 1449500, 128, 89700, 'sin_contactar', false, false,
   'El tercero de los tres de Azteca, el único sin pantalla. Sale de restar: el semanal de $486 menos los $184 y $95 de los otros dos deja $207, y los $48,331 del Buró menos lo que falta de esos dos dejan $14,495. Eso son PAGOS POR VENIR, no saldo. Falta su pantalla de "si liquidas hoy". La tasa es la de sus hermanos, estimada.'),
  ('Fonacot', 6823700, 6823700, 20, 282300, 'sin_contactar', false, true,
   'Se descuenta de nómina: no compite por el efectivo.'),
  ('Préstamo ISSSTE', 6253700, 6253700, 12, 271900, 'sin_contactar', false, true,
   '46 quincenas restantes de $1,359.50. Va por nómina y no aparece en buró.')
) AS v
WHERE NOT EXISTS (SELECT 1 FROM acreedores);

INSERT INTO ingresos (concepto, neto_mensual)
SELECT * FROM (VALUES
  ('Nómina SAT (2 quincenas netas)', 1758300::bigint),
  ('Nómina de Wendy (ISSEMyM, sin retroactivos)', 1012500::bigint)
) AS v
WHERE NOT EXISTS (SELECT 1 FROM ingresos);

INSERT INTO gastos (concepto, categoria, mensual, recortable)
SELECT * FROM (VALUES
  ('Comida fuera', 'Comida', 698700::bigint, true),
  ('Efectivo retirado', 'Otros', 360000::bigint, true),
  ('Perfumería, PayPal y compras', 'Gustos', 278400::bigint, true),
  ('Compras MercadoPago y comercios', 'Gustos', 260000::bigint, true),
  ('Despensa y abarrotes', 'Comida', 193900::bigint, false),
  ('OXXO y tiendita', 'Comida', 188400::bigint, true),
  ('Suscripciones digitales', 'Apps', 142900::bigint, true),
  ('Salud y laboratorio', 'Salud', 80500::bigint, false),
  ('Celular y recargas', 'Servicios', 52500::bigint, false),
  ('Transporte al trabajo (Mexibús y Metro)', 'Transporte', 57600::bigint, false)
) AS v
WHERE NOT EXISTS (SELECT 1 FROM gastos);

INSERT INTO movimientos_personales (persona, concepto, monto, tipo, fecha)
SELECT 'Mario', 'Préstamo recibido', 4300000, 'entrega', DATE '2026-08-07'
WHERE NOT EXISTS (SELECT 1 FROM movimientos_personales);

INSERT INTO pendientes (texto, vence)
SELECT * FROM (VALUES
  ('Contestar a HeyBanco aceptando la Opción 1', DATE '2026-09-16'),
  ('Contestar a Klar antes de que cedan la cuenta', DATE '2026-09-16'),
  ('Presentarle el plan a Mario', DATE '2026-09-19'),
  ('Pagar tenencia de Morelos', DATE '2026-09-23'),
  ('Pedirle la nivelación a Víctor', DATE '2026-09-25'),
  ('Tramitar baja de placas del CUPRA', DATE '2026-09-30'),
  ('Pedir a VW la carta finiquito del CUPRA', DATE '2026-09-30'),
  ('Conseguir el saldo del Consupago de Wendy', NULL)
) AS v
WHERE NOT EXISTS (SELECT 1 FROM pendientes);
