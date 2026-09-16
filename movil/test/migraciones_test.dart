import 'package:finanzas/datos/migraciones.dart';
import 'package:finanzas/datos/semilla.dart';
import 'package:finanzas/modelo/datos.dart';
import 'package:flutter_test/flutter_test.dart';

/// Reconstruye el estado como estaba guardado en la revisión 1: con el cargo
/// de gasolina del CUPRA todavía vivo.
Estado estadoViejo() {
  final e = semilla();
  e.gastos.add(
    Renglon(id: 'g2', concepto: 'SERV sin identificar', monto: 434800, recortable: true),
  );
  e.pendientes.removeWhere((p) => p.id == 'p9');
  return e;
}

void main() {
  test('la migración quita la gasolina del CUPRA', () {
    final e = estadoViejo();
    final gastoAntes = e.gastoMensual;

    expect(migrar(e, 1), revisionActual);

    expect(e.gastos.any((g) => g.id == 'g2'), isFalse);
    expect(e.gastoMensual, gastoAntes - 434800);
  });

  test('la migración deja el pendiente de confirmarlo', () {
    final e = estadoViejo();
    migrar(e, 1);
    expect(e.pendientes.where((p) => p.id == 'p9').length, 1);
  });

  test('no borra lo que el usuario haya capturado', () {
    final e = estadoViejo();
    e.gastos.add(Renglon(id: 'mio', concepto: 'Pañales', monto: 90000));
    e.pendientes.add(Pendiente(id: 'mio', texto: 'Cita del bebé'));

    migrar(e, 1);

    expect(e.gastos.any((g) => g.id == 'mio'), isTrue);
    expect(e.pendientes.any((p) => p.id == 'mio'), isTrue);
  });

  test('correr la migración dos veces no duplica nada', () {
    final e = estadoViejo();
    migrar(e, 1);
    final gastos = e.gastoMensual;
    final pendientes = e.pendientes.length;

    migrar(e, 1);

    expect(e.gastoMensual, gastos);
    expect(e.pendientes.length, pendientes);
  });

  test('un estado ya al día no se toca', () {
    final e = semilla();
    final antes = e.aTexto();
    expect(migrar(e, revisionActual), revisionActual);
    expect(e.aTexto(), antes);
  });
}
