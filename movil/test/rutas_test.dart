import 'package:finanzas/modelo/rutas.dart';
import 'package:flutter_test/flutter_test.dart';

Ruta conTramos(List<int> tarifas, {int minutos = 0}) => Ruta(
      id: 'r',
      nombre: 'Ruta',
      minutos: minutos,
      tramos: [
        for (var i = 0; i < tarifas.length; i++)
          Tramo(id: 't$i', descripcion: 'Tramo $i', tarifa: tarifas[i]),
      ],
    );

void main() {
  group('lo que cuesta la ruta', () {
    test('el trayecto es la suma de sus tramos', () {
      expect(conTramos([1000, 500, 300]).costoTrayecto, 1800);
    });

    test('el dia es ida y vuelta', () {
      final r = conTramos([1000, 500, 300], minutos: 95);
      expect(r.costoDia, 3600);
      expect(r.minutosDia, 190);
    });

    test('el mes multiplica por los dias que se va a la oficina', () {
      // 18 pesos el trayecto, 36 al dia, 16 dias: 576 al mes.
      expect(conTramos([1000, 500, 300]).costoMes(16), 57600);
    });

    test('una ruta sin tramos no cuesta nada', () {
      expect(Ruta(id: 'x', nombre: 'Vacia').costoMes(20), 0);
    });
  });

  group('lo que costaria en aplicacion', () {
    test('el mes son ida y vuelta por los dias presenciales', () {
      final v = Viaje(diasPresenciales: 16, uberIda: 24000, uberRegreso: 25900);
      expect(v.uberDia, 49900);
      expect(v.uberMes, 798400);
    });
  });

  group('acumular lo que no se gasta', () {
    test('sin rendimiento solo se suman los aportes', () {
      final filas = acumular(aporteMensual: 100000, meses: 12);
      expect(filas.length, 12);
      expect(filas.last.saldo, 1200000);
      expect(filas.last.rendimiento, 0);
    });

    test('con rendimiento junta mas de lo aportado', () {
      final filas = acumular(aporteMensual: 100000, meses: 24, rendimientoAnual: 10);
      expect(filas.last.aportado, 2400000);
      expect(filas.last.saldo, greaterThan(filas.last.aportado));
      expect(filas.last.rendimiento, filas.last.saldo - filas.last.aportado);
    });

    test('el aporte entra al final del mes, asi que el primero no rinde', () {
      final filas = acumular(aporteMensual: 100000, meses: 1, rendimientoAnual: 100);
      expect(filas.single.saldo, 100000);
      expect(filas.single.rendimiento, 0);
    });

    test('el saldo nunca baja', () {
      final filas = acumular(aporteMensual: 57600, meses: 60, rendimientoAnual: 10);
      for (var i = 1; i < filas.length; i++) {
        expect(filas[i].saldo, greaterThanOrEqualTo(filas[i - 1].saldo));
      }
    });

    test('sin aporte no hay nada que acumular', () {
      expect(acumular(aporteMensual: 0, meses: 24, rendimientoAnual: 10), isEmpty);
      expect(acumular(aporteMensual: 100000, meses: 0), isEmpty);
    });

    test('un plazo absurdo se recorta en vez de colgar la app', () {
      expect(acumular(aporteMensual: 1000, meses: 100000).length, 600);
    });
  });
}
