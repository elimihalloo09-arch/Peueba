import '../modelo/datos.dart';

/// Versión de los datos que entiende esta compilación.
///
/// Los datos viven en el teléfono, así que cuando descubrimos que una cifra
/// era falsa no basta con corregir la semilla: lo que ya está guardado hay
/// que corregirlo también, sin borrar lo que se haya capturado encima.
/// Cada corrección sube este número y agrega un paso en [migrar].
const int revisionActual = 2;

/// Lleva un estado guardado desde la revisión [desde] hasta [revisionActual]
/// y devuelve la revisión en la que quedó.
int migrar(Estado e, int desde) {
  var r = desde;
  if (r < 2) {
    _quitarGasolinaDelCupra(e);
    r = 2;
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
