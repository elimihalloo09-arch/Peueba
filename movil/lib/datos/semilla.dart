import '../modelo/datos.dart';

/// Datos reales de Carlos, en centavos. Es el punto de partida; desde la app
/// se edita todo.
Estado semilla() => Estado(
      acreedores: [
        Acreedor(
          id: 'tenencia',
          nombre: 'Tenencia y refrendo Morelos',
          saldoOriginal: 1000000,
          montoAPagar: 1000000,
          admiteQuita: false,
          urgente: true,
          nota: 'Vence el 23 de septiembre. Sin esto no se puede dar de baja '
              'las placas del CUPRA entregado.',
        ),
        Acreedor(
          id: 'heybanco',
          nombre: 'HeyBanco · tarjeta',
          saldoOriginal: 1371900,
          montoAPagar: 411600,
          tasaAnual: 70,
          pagoMensual: 107200,
          etapa: Etapa.conOferta,
          nota: 'Nancy Nelly Ávila Almaguer, nancy.avila@hey.inc. Ofreció por '
              'escrito quita del 70% el 15 de septiembre. Carta finiquito a 30 '
              'días hábiles del pago.',
        ),
        Acreedor(
          id: 'klar',
          nombre: 'Klar · tarjeta',
          saldoOriginal: 1342400,
          montoAPagar: 469800,
          tasaAnual: 70,
          etapa: Etapa.contactado,
          nota: '207 días de atraso, saldo vencido de \$14,610.88. Avisaron '
              'cesión a cobranza externa: negociar antes.',
        ),
        Acreedor(
          id: 'nu-tarjeta',
          nombre: 'NU · tarjeta',
          saldoOriginal: 2350500,
          montoAPagar: 822700,
          tasaAnual: 70,
          nota: 'En cobranza, atraso de 150 días a 12 meses. Pedir quita del 80%.',
        ),
        Acreedor(
          id: 'coppel',
          nombre: 'Coppel · préstamo',
          saldoOriginal: 1825300,
          montoAPagar: 638900,
          tasaAnual: 70,
          nota: 'Más de 12 meses de atraso: el mayor margen de quita. '
              'POR VERIFICAR: puede ser la misma deuda que los dos préstamos '
              'personales semanales, y entonces estaría contada dos veces.',
        ),
        Acreedor(
          id: 'fintopia',
          nombre: 'Fintopia · préstamo',
          saldoOriginal: 1071500,
          montoAPagar: 375000,
          tasaAnual: 90,
          pagoMensual: 491000,
          nota: 'Más de 12 meses de atraso y la tasa más alta de todas.',
        ),
        Acreedor(
          id: 'nu-prestamo',
          nombre: 'NU · préstamo personal',
          saldoOriginal: 118400,
          montoAPagar: 41400,
          tasaAnual: 80,
          pagoMensual: 59000,
          nota: 'El más chico. Se cierra rápido.',
        ),
        Acreedor(
          id: 'bancoppel',
          nombre: 'BanCoppel · tarjeta',
          saldoOriginal: 513269,
          montoAPagar: 513269,
          tasaAnual: 69.4,
          pagoMensual: 130633,
          admiteQuita: false,
          nota: 'Estado de cuenta al 20-ago-2026: saldo \$5,132.69, mínimo '
              '\$1,306.33, fecha límite INMEDIATO. Ya cobra moratorios al '
              '74.4% sobre \$472.50. Pagando el mínimo se liquida en 5 meses '
              'con \$606 de intereses. CAT 98.7%.',
        ),
        Acreedor(
          id: 'personal10500',
          nombre: 'Préstamo personal de \$10,500',
          saldoOriginal: 1033000,
          montoAPagar: 1033000,
          tasaAnual: 128,
          pagoMensual: 79733,
          admiteQuita: false,
          nota: 'Semanal de \$184 en la app (\$195 en sucursal). Dispuesto el '
              '4-feb-2026 a 154 semanas; llevas 30. Liquidarlo hoy cuesta '
              '\$10,330 contra \$22,816 de seguir pagando: ahorras \$12,486.',
        ),
        Acreedor(
          id: 'personal5000',
          nombre: 'Préstamo personal de \$5,000',
          saldoOriginal: 504100,
          montoAPagar: 504100,
          tasaAnual: 137,
          pagoMensual: 41167,
          admiteQuita: false,
          nota: 'Semanal de \$95 en la app (\$100 en sucursal). Dispuesto el '
              '10-jun-2026 a 128 semanas; llevas 12. Liquidarlo hoy cuesta '
              '\$5,041 contra \$11,020 de seguir pagando: ahorras \$5,979.',
        ),
        Acreedor(
          id: 'telcel',
          nombre: 'Telcel · línea',
          saldoOriginal: 60800,
          montoAPagar: 60800,
          pagoMensual: 60800,
          admiteQuita: false,
          nota: 'Lo más chico de todo. Pagarlo y olvidarlo.',
        ),
        Acreedor(
          id: 'azteca',
          nombre: 'Banco Azteca · tres créditos',
          saldoOriginal: 4833100,
          montoAPagar: 4833100,
          tasaAnual: 75,
          pagoMensual: 211100,
          admiteQuita: false,
          nota: 'Al corriente, pago semanal de \$486. Sin atraso no hay quita.',
        ),
        Acreedor(
          id: 'fonacot',
          nombre: 'Fonacot',
          saldoOriginal: 6823700,
          montoAPagar: 6823700,
          tasaAnual: 20,
          pagoMensual: 282300,
          admiteQuita: false,
          porNomina: true,
          nota: 'Se descuenta de nómina: no compite por el efectivo.',
        ),
        Acreedor(
          id: 'issste',
          nombre: 'Préstamo ISSSTE',
          saldoOriginal: 6253700,
          montoAPagar: 6253700,
          tasaAnual: 12,
          pagoMensual: 271900,
          admiteQuita: false,
          porNomina: true,
          nota: '46 quincenas restantes de \$1,359.50. Va por nómina y no '
              'aparece en buró.',
        ),
      ],
      pendientes: [
        Pendiente(id: 'p1', texto: 'Contestar a HeyBanco aceptando la Opción 1', vence: DateTime(2026, 9, 16)),
        Pendiente(id: 'p2', texto: 'Contestar a Klar antes de que cedan la cuenta', vence: DateTime(2026, 9, 16)),
        Pendiente(id: 'p3', texto: 'Presentarle el plan a Mario', vence: DateTime(2026, 9, 19)),
        Pendiente(id: 'p4', texto: 'Pagar tenencia de Morelos', vence: DateTime(2026, 9, 23)),
        Pendiente(id: 'p5', texto: 'Pedirle la nivelación a Víctor', vence: DateTime(2026, 9, 25)),
        Pendiente(id: 'p6', texto: 'Tramitar baja de placas del CUPRA', vence: DateTime(2026, 9, 30)),
        Pendiente(id: 'p7', texto: 'Pedir a VW la carta finiquito del CUPRA', vence: DateTime(2026, 9, 30)),
        Pendiente(id: 'p8', texto: 'Conseguir el saldo del Consupago de Wendy'),
        Pendiente(
          id: 'p9',
          texto: 'Confirmar en el estado de cuenta de octubre que ya no hay gasolina',
          vence: DateTime(2026, 11, 5),
        ),
        Pendiente(
          id: 'p10',
          texto: 'Pagar el mínimo de BanCoppel, \$1,306, antes del corte del 20',
          vence: DateTime(2026, 9, 20),
        ),
        Pendiente(
          id: 'p11',
          texto: 'Liquidar el préstamo de \$10,330 con el primer apoyo de Mario',
          vence: DateTime(2026, 10, 15),
        ),
        Pendiente(
          id: 'p12',
          texto: 'Verificar si los préstamos semanales ya están dentro del saldo de Coppel',
          vence: DateTime(2026, 9, 22),
        ),
      ],
      movimientos: [
        Movimiento(
          id: 'm1',
          persona: 'Mario',
          concepto: 'Préstamo recibido',
          monto: 4300000,
          esEntrega: true,
          fecha: DateTime(2026, 8, 7),
        ),
      ],
      ingresos: [
        Renglon(id: 'i1', concepto: 'Nómina SAT (2 quincenas netas)', monto: 1758300),
        Renglon(id: 'i2', concepto: 'Nómina de Wendy (ISSEMyM)', monto: 1012500),
      ],
      gastos: [
        Renglon(id: 'g1', concepto: 'Comida fuera', monto: 698700, recortable: true),
        Renglon(id: 'g3', concepto: 'Efectivo retirado', monto: 360000, recortable: true),
        Renglon(id: 'g4', concepto: 'Perfumería, PayPal y compras', monto: 278400, recortable: true),
        Renglon(id: 'g5', concepto: 'Compras MercadoPago', monto: 260000, recortable: true),
        Renglon(id: 'g6', concepto: 'Despensa y abarrotes', monto: 193900),
        Renglon(id: 'g7', concepto: 'OXXO y tiendita', monto: 188400, recortable: true),
        Renglon(id: 'g8', concepto: 'Suscripciones digitales', monto: 142900, recortable: true),
        Renglon(id: 'g9', concepto: 'Salud y laboratorio', monto: 80500),
        Renglon(id: 'g10', concepto: 'Celular y recargas', monto: 52500),
        Renglon(id: 'g11', concepto: 'Transporte al trabajo', monto: 57600),
      ],
    );
