import '../modelo/datos.dart';

/// Versión de los datos que entiende esta compilación.
///
/// Los datos viven en el teléfono, así que cuando descubrimos que una cifra
/// era falsa no basta con corregir la semilla: lo que ya está guardado hay
/// que corregirlo también, sin borrar lo que se haya capturado encima.
/// Cada corrección sube este número y agrega un paso en [migrar].
const int revisionActual = 3;

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
