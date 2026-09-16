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
  e.pendientes.removeWhere(
      (p) => p.id == 'p9' || p.id == 'p10' || p.id == 'p11' || p.id == 'p12');
  e.acreedores.removeWhere(
      (a) => a.id == 'personal10500' || a.id == 'personal5000');
  for (final a in e.acreedores.where((a) => a.id == 'coppel')) {
    a.nota = 'Más de 12 meses de atraso: el mayor margen de quita.';
  }
  for (final a in e.acreedores.where((a) => a.id == 'bancoppel')) {
    a.saldoOriginal = 481200;
    a.montoAPagar = 481200;
    a.tasaAnual = 70;
    a.pagoMensual = 90100;
    a.nota = 'Atraso corto, de 1 a 29 días. Probablemente sin quita.';
  }
  return e;
}

Acreedor banCoppel(Estado e) =>
    e.acreedores.firstWhere((a) => a.id == 'bancoppel');

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

  test('BanCoppel toma las cifras de su estado de cuenta', () {
    final e = estadoViejo();
    migrar(e, 1);

    final b = banCoppel(e);
    expect(b.montoAPagar, 513269);
    expect(b.pagoMensual, 130633);
    expect(b.tasaAnual, 69.4);
    expect(b.nota, contains('20-ago-2026'));
  });

  test('si BanCoppel ya se edito a mano, la migracion no lo pisa', () {
    final e = estadoViejo();
    final b = banCoppel(e);
    b.montoAPagar = 600000;
    b.pagoMensual = 150000;

    migrar(e, 1);

    expect(banCoppel(e).montoAPagar, 600000);
    expect(banCoppel(e).pagoMensual, 150000);
  });

  test('entran los dos préstamos semanales', () {
    final e = estadoViejo();
    final deudaAntes = e.deudaPorPagar;

    // Desde la 3, para que la correccion de BanCoppel no mueva el total y la
    // cuenta mida solo lo que agrega este paso.
    migrar(e, 3);

    final uno = e.acreedores.firstWhere((a) => a.id == 'personal10500');
    final dos = e.acreedores.firstWhere((a) => a.id == 'personal5000');
    expect(uno.montoAPagar, 1033000);
    expect(dos.montoAPagar, 504100);
    expect(e.deudaPorPagar, deudaAntes + 1033000 + 504100);
  });

  test('son las tasas más altas de la lista', () {
    final e = estadoViejo();
    migrar(e, 1);

    final mayor = e.vivos.map((a) => a.tasaAnual).reduce((a, b) => a > b ? a : b);
    expect(mayor, 137);
  });

  test('Coppel queda marcado como posible duplicado', () {
    final e = estadoViejo();
    migrar(e, 1);
    final c = e.acreedores.firstWhere((a) => a.id == 'coppel');
    expect(c.nota, contains('POR VERIFICAR'));
  });

  test('migrar dos veces no duplica los préstamos ni la marca de Coppel', () {
    final e = estadoViejo();
    migrar(e, 1);
    final acreedores = e.acreedores.length;
    final nota = e.acreedores.firstWhere((a) => a.id == 'coppel').nota;

    migrar(e, 1);

    expect(e.acreedores.length, acreedores);
    expect(e.acreedores.firstWhere((a) => a.id == 'coppel').nota, nota);
  });

  test('un estado ya al día no se toca', () {
    final e = semilla();
    final antes = e.aTexto();
    expect(migrar(e, revisionActual), revisionActual);
    expect(e.aTexto(), antes);
  });
}
