/// Simulador de liquidación de deudas.
///
/// Dos métodos: avalancha (primero la tasa más alta, paga menos intereses) y
/// bola de nieve (primero el saldo más chico, da victorias antes). El pago
/// mensual que se libera al saldar una deuda rueda a la siguiente.
///
/// Todo en centavos, con enteros. Lógica pura: se prueba sin necesitar
/// teléfono ni base de datos.
library;

enum Metodo { avalancha, bolaDeNieve }

class DeudaSimulada {
  const DeudaSimulada({
    required this.nombre,
    required this.saldo,
    this.pagoMensual = 0,
    this.tasaAnual = 0,
    this.urgente = false,
  });

  final String nombre;

  /// Lo que de verdad hay que pagar: ya con quita si se negoció.
  final int saldo;
  final int pagoMensual;

  /// Tasa anual en porcentaje: 70 es 70%.
  final double tasaAnual;

  /// Se ataca primero, sin importar tasa ni saldo.
  final bool urgente;
}

class DeudaLiquidada {
  const DeudaLiquidada(this.nombre, this.mes, this.totalPagado);
  final String nombre;
  final int mes;
  final int totalPagado;
}

class PuntoCurva {
  const PuntoCurva(this.mes, this.saldo);
  final int mes;
  final int saldo;
}

class Simulacion {
  const Simulacion({
    required this.meses,
    required this.intereses,
    required this.totalPagado,
    required this.orden,
    required this.curva,
    required this.inalcanzable,
    required this.atoradas,
  });

  final int meses;
  final int intereses;
  final int totalPagado;
  final List<DeudaLiquidada> orden;
  final List<PuntoCurva> curva;

  /// True cuando el abono no alcanza ni para cubrir los intereses.
  final bool inalcanzable;
  final List<String> atoradas;
}

const int _topeMeses = 360;

/// Debajo de un peso damos la deuda por saldada: son residuos de redondeo.
const int _umbralSaldada = 100;

class _EnCurso {
  _EnCurso(this.d) : saldo = d.saldo;
  final DeudaSimulada d;
  int saldo;
  int pagado = 0;
  int? mesFin;
}

/// Simula la liquidación.
///
/// [capacidad] es TODO el dinero que se destina cada mes a deuda, en centavos:
/// de ahí salen primero los mínimos y lo que sobra se concentra en una sola
/// deuda. Si no alcanza ni para los mínimos, el resultado sale `inalcanzable`.
Simulacion simular({
  required List<DeudaSimulada> deudas,
  required int capacidad,
  Metodo metodo = Metodo.avalancha,
  int apoyoMensual = 0,
  int mesesApoyo = 0,
}) {
  final vivas = deudas.where((d) => d.saldo > 0).map(_EnCurso.new).toList();

  int saldoTotal() =>
      vivas.fold(0, (a, d) => a + (d.saldo > 0 ? d.saldo : 0));

  final curva = <PuntoCurva>[PuntoCurva(0, saldoTotal())];
  final orden = <DeudaLiquidada>[];
  var intereses = 0;
  var totalPagado = 0;
  var liberado = 0;
  var mes = 0;

  while (mes < _topeMeses && vivas.any((d) => d.saldo > _umbralSaldada)) {
    mes++;

    for (final d in vivas.where((d) => d.saldo > _umbralSaldada)) {
      final interes = (d.saldo * d.d.tasaAnual / 100 / 12).round();
      d.saldo += interes;
      intereses += interes;
    }

    final apoyo = mes <= mesesApoyo ? apoyoMensual : 0;
    var bolsa = capacidad + apoyo + liberado;

    // Primero los mínimos comprometidos.
    for (final d in vivas.where((d) => d.saldo > _umbralSaldada)) {
      if (bolsa <= 0) break;
      final pago = [d.d.pagoMensual, d.saldo, bolsa].reduce((a, b) => a < b ? a : b);
      if (pago > 0) {
        d.saldo -= pago;
        d.pagado += pago;
        bolsa -= pago;
        totalPagado += pago;
      }
    }

    // Lo que sobra se concentra en una sola deuda.
    final cola = vivas.where((d) => d.saldo > _umbralSaldada).toList()
      ..sort((a, b) {
        // Lo urgente gana sobre cualquier criterio financiero.
        if (a.d.urgente != b.d.urgente) return a.d.urgente ? -1 : 1;
        return metodo == Metodo.avalancha
            ? b.d.tasaAnual.compareTo(a.d.tasaAnual)
            : a.saldo.compareTo(b.saldo);
      });
    for (final d in cola) {
      if (bolsa <= 0) break;
      final golpe = bolsa < d.saldo ? bolsa : d.saldo;
      d.saldo -= golpe;
      d.pagado += golpe;
      bolsa -= golpe;
      totalPagado += golpe;
    }

    for (final d in vivas) {
      if (d.saldo <= _umbralSaldada && d.mesFin == null) {
        d.saldo = 0;
        d.mesFin = mes;
        liberado += d.d.pagoMensual;
        orden.add(DeudaLiquidada(d.d.nombre, mes, d.pagado));
      }
    }

    curva.add(PuntoCurva(mes, saldoTotal()));
  }

  final atoradas = vivas
      .where((d) => d.saldo > _umbralSaldada)
      .map((d) => d.d.nombre)
      .toList();

  return Simulacion(
    meses: mes,
    intereses: intereses,
    totalPagado: totalPagado,
    orden: orden,
    curva: curva,
    inalcanzable: atoradas.isNotEmpty,
    atoradas: atoradas,
  );
}
