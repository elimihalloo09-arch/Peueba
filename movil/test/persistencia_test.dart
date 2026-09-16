import 'package:finanzas/modelo/datos.dart';
import 'package:flutter_test/flutter_test.dart';

/// Un estado con algo de cada cosa, para que el viaje de ida y vuelta pase por
/// todos los campos.
Estado completo() => Estado(
      acreedores: [
        Acreedor(
          id: 'uno',
          nombre: 'Tarjeta',
          saldoOriginal: 1371900,
          montoAPagar: 411600,
          tasaAnual: 69.4,
          pagoMensual: 107200,
          etapa: Etapa.conOferta,
          admiteQuita: true,
          tieneConvenio: true,
          nota: 'Con acentos: negociación',
        ),
        Acreedor(
          id: 'dos',
          nombre: 'Nómina',
          saldoOriginal: 6823700,
          montoAPagar: 6823700,
          tasaAnual: 20,
          pagoMensual: 282300,
          porNomina: true,
          urgente: true,
          etapa: Etapa.pagado,
        ),
      ],
      pendientes: [
        Pendiente(id: 'p1', texto: 'Con fecha', vence: DateTime(2026, 9, 23)),
        Pendiente(id: 'p2', texto: 'Sin fecha', hecho: true),
      ],
      movimientos: [
        Movimiento(
          id: 'm1',
          persona: 'Alguien',
          concepto: 'Préstamo recibido',
          monto: 4300000,
          esEntrega: true,
          fecha: DateTime(2026, 8, 7),
        ),
      ],
      ingresos: [Renglon(id: 'i1', concepto: 'Sueldo', monto: 1758300)],
      gastos: [Renglon(id: 'g1', concepto: 'Comida', monto: 698700, recortable: true)],
    );

void main() {
  test('lo guardado se puede volver a leer', () {
    final antes = completo();
    final texto = antes.aTexto();

    final despues = Estado.deTexto(texto);

    expect(despues.aTexto(), texto);
  });

  test('sobreviven los campos de cada acreedor', () {
    final d = Estado.deTexto(completo().aTexto());
    final uno = d.acreedores.firstWhere((a) => a.id == 'uno');

    expect(uno.nombre, 'Tarjeta');
    expect(uno.montoAPagar, 411600);
    expect(uno.tasaAnual, 69.4);
    expect(uno.etapa, Etapa.conOferta);
    expect(uno.tieneConvenio, isTrue);
    expect(uno.nota, contains('negociación'));

    final dos = d.acreedores.firstWhere((a) => a.id == 'dos');
    expect(dos.porNomina, isTrue);
    expect(dos.urgente, isTrue);
    expect(dos.etapa, Etapa.pagado);
  });

  test('sobreviven las fechas, las que hay y las que no', () {
    final d = Estado.deTexto(completo().aTexto());
    expect(d.pendientes.firstWhere((p) => p.id == 'p1').vence, DateTime(2026, 9, 23));
    expect(d.pendientes.firstWhere((p) => p.id == 'p2').vence, isNull);
    expect(d.pendientes.firstWhere((p) => p.id == 'p2').hecho, isTrue);
    expect(d.movimientos.single.fecha, DateTime(2026, 8, 7));
  });

  test('las cuentas dan lo mismo antes y despues de guardar', () {
    final antes = completo();
    final despues = Estado.deTexto(antes.aTexto());

    expect(despues.ingresoMensual, antes.ingresoMensual);
    expect(despues.gastoMensual, antes.gastoMensual);
    expect(despues.deudaPorPagar, antes.deudaPorPagar);
    expect(despues.pagosMensuales, antes.pagosMensuales);
    expect(despues.ahorroPorQuitas, antes.ahorroPorQuitas);
  });

  test('un estado vacio tambien va y viene', () {
    final v = Estado(acreedores: [], pendientes: [], movimientos: [], ingresos: [], gastos: []);
    expect(Estado.deTexto(v.aTexto()).acreedores, isEmpty);
  });
}
