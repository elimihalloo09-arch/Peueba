// Las rutas de casa al trabajo, lo que cuestan y lo que se acumula con lo
// que no se gasta. Dart puro, sin Flutter, para poder probarlo.

/// Un tramo de un trayecto: un Mexibús, un Metro, una combi.
class Tramo {
  Tramo({required this.id, required this.descripcion, required this.tarifa});

  final String id;
  String descripcion;

  /// En centavos, como todo el dinero de la app.
  int tarifa;

  Map<String, dynamic> aJson() =>
      {'id': id, 'descripcion': descripcion, 'tarifa': tarifa};

  static Tramo deJson(Map<String, dynamic> j) => Tramo(
        id: j['id'] as String,
        descripcion: j['descripcion'] as String? ?? '',
        tarifa: (j['tarifa'] as num?)?.toInt() ?? 0,
      );
}

/// Un destino con su camino. Los tramos describen la ida; el regreso se cobra
/// igual salvo que se diga otra cosa.
class Ruta {
  Ruta({
    required this.id,
    required this.nombre,
    List<Tramo>? tramos,
    this.minutos = 0,
    this.nota = '',
    this.planB = '',
  }) : tramos = List.of(tramos ?? const []);

  final String id;
  String nombre;
  List<Tramo> tramos;

  /// Minutos de un trayecto, puerta a puerta.
  int minutos;
  String nota;
  String planB;

  int get costoTrayecto => tramos.fold(0, (t, x) => t + x.tarifa);
  int get costoDia => costoTrayecto * 2;
  int get minutosDia => minutos * 2;

  int costoMes(int diasPresenciales) => costoDia * diasPresenciales;
  int minutosMes(int diasPresenciales) => minutosDia * diasPresenciales;

  Map<String, dynamic> aJson() => {
        'id': id,
        'nombre': nombre,
        'tramos': tramos.map((t) => t.aJson()).toList(),
        'minutos': minutos,
        'nota': nota,
        'planB': planB,
      };

  static Ruta deJson(Map<String, dynamic> j) => Ruta(
        id: j['id'] as String,
        nombre: j['nombre'] as String? ?? '',
        tramos: ((j['tramos'] as List?) ?? const [])
            .map((e) => Tramo.deJson(e as Map<String, dynamic>))
            .toList(),
        minutos: (j['minutos'] as num?)?.toInt() ?? 0,
        nota: j['nota'] as String? ?? '',
        planB: j['planB'] as String? ?? '',
      );
}

/// Los supuestos del traslado: cuantos dias se va a la oficina y cuanto
/// costaria el mismo camino en aplicacion.
class Viaje {
  Viaje({
    this.diasPresenciales = 16,
    this.uberIda = 0,
    this.uberRegreso = 0,
    this.rutaBase = '',
    this.aporteMensual = 0,
    this.mesesAhorro = 24,
    this.rendimientoAnual = 10,
  });

  int diasPresenciales;
  int uberIda;
  int uberRegreso;

  /// La ruta que se toma como referencia al comparar y al proyectar.
  String rutaBase;

  /// Lo que se aparta al mes. Cero significa "usa el ahorro contra la app".
  int aporteMensual;
  int mesesAhorro;
  double rendimientoAnual;

  int get uberDia => uberIda + uberRegreso;
  int get uberMes => uberDia * diasPresenciales;

  Map<String, dynamic> aJson() => {
        'diasPresenciales': diasPresenciales,
        'uberIda': uberIda,
        'uberRegreso': uberRegreso,
        'rutaBase': rutaBase,
        'aporteMensual': aporteMensual,
        'mesesAhorro': mesesAhorro,
        'rendimientoAnual': rendimientoAnual,
      };

  static Viaje deJson(Map<String, dynamic> j) => Viaje(
        diasPresenciales: (j['diasPresenciales'] as num?)?.toInt() ?? 16,
        uberIda: (j['uberIda'] as num?)?.toInt() ?? 0,
        uberRegreso: (j['uberRegreso'] as num?)?.toInt() ?? 0,
        rutaBase: j['rutaBase'] as String? ?? '',
        aporteMensual: (j['aporteMensual'] as num?)?.toInt() ?? 0,
        mesesAhorro: (j['mesesAhorro'] as num?)?.toInt() ?? 24,
        rendimientoAnual: (j['rendimientoAnual'] as num?)?.toDouble() ?? 10,
      );
}

/// Un mes de la proyeccion de ahorro.
class MesAhorro {
  const MesAhorro({
    required this.mes,
    required this.aportado,
    required this.rendimiento,
    required this.saldo,
  });

  final int mes;

  /// Acumulados desde el principio, no del mes suelto.
  final int aportado;
  final int rendimiento;
  final int saldo;
}

const int _topeMeses = 600;

/// Proyecta cuanto se junta apartando [aporteMensual] durante [meses] a una
/// tasa anual de [rendimientoAnual] por ciento.
///
/// El aporte entra al final de cada mes, que es lo prudente: supone que el
/// dinero empieza a rendir despues de depositarlo, no antes.
List<MesAhorro> acumular({
  required int aporteMensual,
  required int meses,
  double rendimientoAnual = 0,
}) {
  final total = meses.clamp(0, _topeMeses);
  if (total == 0 || aporteMensual <= 0) return const [];

  final mensual = rendimientoAnual / 100 / 12;
  final filas = <MesAhorro>[];
  var saldo = 0.0;
  var aportado = 0;

  for (var m = 1; m <= total; m++) {
    saldo *= 1 + mensual;
    saldo += aporteMensual;
    aportado += aporteMensual;
    filas.add(MesAhorro(
      mes: m,
      aportado: aportado,
      rendimiento: saldo.round() - aportado,
      saldo: saldo.round(),
    ));
  }
  return filas;
}
