import 'package:finanzas/modelo/simulador.dart';
import 'package:flutter_test/flutter_test.dart';

DeudaSimulada deuda(String nombre, int saldo, {int pago = 0, double tasa = 0, bool urgente = false}) =>
    DeudaSimulada(nombre: nombre, saldo: saldo, pagoMensual: pago, tasaAnual: tasa, urgente: urgente);

void main() {
  test('sin intereses se liquida en los meses justos', () {
    final s = simular(deudas: [deuda('una', 1000000)], capacidad: 250000);
    expect(s.meses, 4);
    expect(s.intereses, 0);
    expect(s.inalcanzable, isFalse);
  });

  test('la avalancha paga menos intereses que la bola de nieve', () {
    final deudas = [
      deuda('cara y grande', 3000000, pago: 50000, tasa: 90),
      deuda('barata y chica', 500000, pago: 20000, tasa: 20),
    ];
    final a = simular(deudas: deudas, capacidad: 300000);
    final b = simular(deudas: deudas, capacidad: 300000, metodo: Metodo.bolaDeNieve);
    expect(a.intereses, lessThanOrEqualTo(b.intereses));
  });

  test('la bola de nieve cierra antes la deuda chica', () {
    final deudas = [deuda('grande', 3000000, tasa: 50), deuda('chica', 200000, tasa: 10)];
    final b = simular(deudas: deudas, capacidad: 300000, metodo: Metodo.bolaDeNieve);
    expect(b.orden.first.nombre, 'chica');
  });

  test('lo urgente se paga primero aunque no cobre intereses', () {
    final deudas = [
      deuda('tarjeta', 2000000, tasa: 90),
      deuda('tenencia', 1000000, urgente: true),
    ];
    final s = simular(deudas: deudas, capacidad: 1000000);
    expect(s.orden.first.nombre, 'tenencia');
  });

  test('el apoyo acorta el plazo y baja los intereses', () {
    final deudas = [deuda('tarjeta', 6000000, pago: 100000, tasa: 70)];
    final sin = simular(deudas: deudas, capacidad: 300000);
    final con = simular(deudas: deudas, capacidad: 300000, apoyoMensual: 1000000, mesesApoyo: 6);
    expect(con.meses, lessThan(sin.meses));
    expect(con.intereses, lessThan(sin.intereses));
  });

  test('si el abono no cubre intereses se marca inalcanzable', () {
    final s = simular(deudas: [deuda('imposible', 10000000, pago: 10000, tasa: 90)], capacidad: 0);
    expect(s.inalcanzable, isTrue);
    expect(s.atoradas, hasLength(1));
  });

  test('el pago liberado rueda a la siguiente deuda', () {
    final deudas = [deuda('chica', 100000, pago: 50000), deuda('grande', 1000000, pago: 50000)];
    final s = simular(deudas: deudas, capacidad: 100000);
    expect(s.meses, lessThan(20));
    expect(s.orden.first.nombre, 'chica');
  });

  test('sin capacidad no se paga nada y se avisa', () {
    final s = simular(deudas: [deuda('tarjeta', 1000000, pago: 50000)], capacidad: 0);
    expect(s.inalcanzable, isTrue);
    expect(s.totalPagado, 0);
  });
}
