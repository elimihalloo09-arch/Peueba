import '../modelo/datos.dart';

/// Versión de los datos que entiende esta compilación.
///
/// Los datos viven en el teléfono, así que cuando descubrimos que una cifra
/// era falsa no basta con corregir la semilla: lo que ya está guardado hay
/// que corregirlo también, sin borrar lo que se haya capturado encima.
/// Cada corrección sube este número y agrega un paso en [migrar].
const int revisionActual = 5;

/// Lleva un estado guardado desde la revisión [desde] hasta [revisionActual]
/// y devuelve la revisión en la que quedó.
int migrar(Estado e, int desde) {
  var r = desde;
  if (r < 2) {
    _quitarGasolinaDelCupra(e);
    r = 2;
  }
  if (r < 3) {
    _corregirBanCoppelConSuEstadoDeCuenta(e);
    r = 3;
  }
  if (r < 4) {
    _agregarLosPrestamosSemanales(e);
    r = 4;
  }
  if (r < 5) {
    _losSemanalesEranDeBancoAzteca(e);
    r = 5;
  }
  return r;
}

/// El cargo "SERV sin identificar", $4,348 al mes, era la gasolina del CUPRA.
/// El CUPRA se entregó en dación en pago el 8 de septiembre de 2026, así que
/// ese gasto ya no existe. Queda el pendiente de confirmarlo contra el estado
/// de cuenta de octubre, que es el primer mes completo sin coche.
void _quitarGasolinaDelCupra(Estado e) {
  e.gastos.removeWhere((g) => g.id == 'g2');
  if (!e.pendientes.any((p) => p.id == 'p9')) {
    e.pendientes.add(
      Pendiente(
        id: 'p9',
        texto: 'Confirmar en el estado de cuenta de octubre que ya no hay gasolina',
        vence: DateTime(2026, 11, 5),
      ),
    );
  }
}

/// BanCoppel estaba cargado con lo que reportaba el Buró: \$4,812 de saldo y
/// \$901 de pago. El estado de cuenta del 20 de agosto de 2026 dice \$5,132.69,
/// mínimo \$1,306.33 y tasa 69.4%, ademas de moratorios al 74.4% porque el
/// pago va vencido.
///
/// Solo corrige si las cifras siguen siendo las del Buró: si ya se editaron a
/// mano, lo escrito por el usuario manda sobre esto.
void _corregirBanCoppelConSuEstadoDeCuenta(Estado e) {
  for (final a in e.acreedores) {
    if (a.id != 'bancoppel') continue;
    if (a.saldoOriginal == 481200 && a.pagoMensual == 90100) {
      a.saldoOriginal = 513269;
      a.montoAPagar = 513269;
      a.tasaAnual = 69.4;
      a.pagoMensual = 130633;
      a.nota = 'Estado de cuenta al 20-ago-2026: saldo \$5,132.69, mínimo '
          '\$1,306.33, fecha límite INMEDIATO. Ya cobra moratorios al '
          '74.4% sobre \$472.50. Pagando el mínimo se liquida en 5 meses '
          'con \$606 de intereses. CAT 98.7%.';
    }
  }
  if (!e.pendientes.any((p) => p.id == 'p10')) {
    e.pendientes.add(
      Pendiente(
        id: 'p10',
        texto: 'Pagar el mínimo de BanCoppel, \$1,306, antes del corte del 20',
        vence: DateTime(2026, 9, 20),
      ),
    );
  }
}

/// Dos préstamos personales que se pagan por semana y que no estaban en la
/// lista. Son, con mucho, lo más caro que se debe: 128% y 137% anual contra
/// el 90% de Fintopia, que hasta ahora encabezaba.
///
/// Lo que los vuelve urgentes no es la tasa sino la diferencia entre seguir
/// pagando y liquidar: \$33,836 contra \$15,371.
void _agregarLosPrestamosSemanales(Estado e) {
  void agregar(Acreedor a) {
    if (e.acreedores.any((x) => x.id == a.id)) return;
    e.acreedores.add(a);
  }

  agregar(
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
  );
  agregar(
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
  );

  for (final a in e.acreedores.where((a) => a.id == 'coppel')) {
    if (!a.nota.contains('POR VERIFICAR')) {
      a.nota = '\${a.nota} POR VERIFICAR: puede ser la misma deuda que los dos '
          'préstamos personales semanales, y entonces estaría contada dos veces.';
    }
  }

  void pendiente(String id, String texto, DateTime vence) {
    if (e.pendientes.any((p) => p.id == id)) return;
    e.pendientes.add(Pendiente(id: id, texto: texto, vence: vence));
  }

  pendiente('p11', 'Liquidar el préstamo de \$10,330 con el primer apoyo de Mario',
      DateTime(2026, 10, 15));
  pendiente('p12',
      'Verificar si los préstamos semanales ya están dentro del saldo de Coppel',
      DateTime(2026, 9, 22));
}

/// Los dos préstamos semanales resultaron ser de Banco Azteca, no de Coppel,
/// y ya estaban dentro del renglón de "tres créditos" que venía del Buró: se
/// estaban contando dos veces.
///
/// Las cuentas cierran por dos lados. El semanal: \$184 + \$95 + \$207 = \$486.
/// Y el total: lo que falta de los dos conocidos, \$33,836, más \$14,495 del
/// tercero, da los \$48,331 del Buró. De paso eso revela que la cifra del Buró
/// no es un saldo sino la suma de las mensualidades que faltan.
void _losSemanalesEranDeBancoAzteca(Estado e) {
  for (final a in e.acreedores) {
    switch (a.id) {
      case 'personal10500':
        if (a.nombre.startsWith('Préstamo')) {
          a.nombre = 'Banco Azteca · préstamo de \$10,500';
        }
      case 'personal5000':
        if (a.nombre.startsWith('Préstamo')) {
          a.nombre = 'Banco Azteca · préstamo de \$5,000';
        }
      case 'azteca':
        // Solo si sigue con lo que reportaba el Buró para los tres juntos.
        if (a.saldoOriginal == 4833100 && a.pagoMensual == 211100) {
          a.nombre = 'Banco Azteca · tercer crédito';
          a.saldoOriginal = 1449500;
          a.montoAPagar = 1449500;
          a.tasaAnual = 128;
          a.pagoMensual = 89700;
          a.nota = 'El tercero de los tres de Azteca, el único sin pantalla. '
              'Sale de restar: el semanal de \$486 menos los \$184 y \$95 de '
              'los otros dos deja \$207, y los \$48,331 del Buró menos lo que '
              'falta de esos dos dejan \$14,495. Eso son PAGOS POR VENIR, no '
              'saldo: liquidarlo hoy debe costar bastante menos, como en los '
              'otros dos. Falta su pantalla de "si liquidas hoy". La tasa es '
              'la de sus hermanos, estimada.';
        }
      case 'coppel':
        a.nota = 'Más de 12 meses de atraso: el mayor margen de quita. Es la '
            'grande, y no tiene nada que ver con BanCoppel ni con Azteca.';
    }
  }

  // Ya se resolvió de quién eran.
  e.pendientes.removeWhere((p) => p.id == 'p12');
  if (!e.pendientes.any((p) => p.id == 'p13')) {
    e.pendientes.add(
      Pendiente(
        id: 'p13',
        texto: 'Sacar la pantalla de "si liquidas hoy" del tercer crédito de Azteca',
        vence: DateTime(2026, 9, 22),
      ),
    );
  }
}
