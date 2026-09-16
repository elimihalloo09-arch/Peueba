import 'dart:convert';

/// Todo el dinero se guarda en CENTAVOS. Nunca en punto flotante.

enum Etapa { sinContactar, contactado, conOferta, convenio, pagado, finiquito }

extension EtapaTexto on Etapa {
  String get texto => switch (this) {
        Etapa.sinContactar => 'Sin contactar',
        Etapa.contactado => 'Contactado',
        Etapa.conOferta => 'Con oferta',
        Etapa.convenio => 'Convenio',
        Etapa.pagado => 'Pagado',
        Etapa.finiquito => 'Finiquito',
      };

  bool get liquidada => this == Etapa.pagado || this == Etapa.finiquito;
}

class Acreedor {
  Acreedor({
    required this.id,
    required this.nombre,
    this.saldoOriginal = 0,
    this.montoAPagar = 0,
    this.tasaAnual = 0,
    this.pagoMensual = 0,
    this.etapa = Etapa.sinContactar,
    this.admiteQuita = true,
    this.porNomina = false,
    this.urgente = false,
    this.tieneConvenio = false,
    this.tieneComprobante = false,
    this.tieneFiniquito = false,
    this.nota = '',
  });

  final String id;
  String nombre;
  int saldoOriginal;
  int montoAPagar;
  double tasaAnual;
  int pagoMensual;
  Etapa etapa;
  bool admiteQuita;
  bool porNomina;
  bool urgente;
  bool tieneConvenio;
  bool tieneComprobante;
  bool tieneFiniquito;
  String nota;

  int get ahorro =>
      saldoOriginal > montoAPagar ? saldoOriginal - montoAPagar : 0;

  int get quitaPorcentaje =>
      saldoOriginal <= 0 ? 0 : ((1 - montoAPagar / saldoOriginal) * 100).round();

  Map<String, dynamic> aJson() => {
        'id': id,
        'nombre': nombre,
        'saldoOriginal': saldoOriginal,
        'montoAPagar': montoAPagar,
        'tasaAnual': tasaAnual,
        'pagoMensual': pagoMensual,
        'etapa': etapa.name,
        'admiteQuita': admiteQuita,
        'porNomina': porNomina,
        'urgente': urgente,
        'tieneConvenio': tieneConvenio,
        'tieneComprobante': tieneComprobante,
        'tieneFiniquito': tieneFiniquito,
        'nota': nota,
      };

  static Acreedor deJson(Map<String, dynamic> j) => Acreedor(
        id: j['id'] as String,
        nombre: j['nombre'] as String? ?? 'Sin nombre',
        saldoOriginal: (j['saldoOriginal'] as num?)?.toInt() ?? 0,
        montoAPagar: (j['montoAPagar'] as num?)?.toInt() ?? 0,
        tasaAnual: (j['tasaAnual'] as num?)?.toDouble() ?? 0,
        pagoMensual: (j['pagoMensual'] as num?)?.toInt() ?? 0,
        etapa: Etapa.values.firstWhere(
          (e) => e.name == j['etapa'],
          orElse: () => Etapa.sinContactar,
        ),
        admiteQuita: j['admiteQuita'] as bool? ?? true,
        porNomina: j['porNomina'] as bool? ?? false,
        urgente: j['urgente'] as bool? ?? false,
        tieneConvenio: j['tieneConvenio'] as bool? ?? false,
        tieneComprobante: j['tieneComprobante'] as bool? ?? false,
        tieneFiniquito: j['tieneFiniquito'] as bool? ?? false,
        nota: j['nota'] as String? ?? '',
      );
}

class Pendiente {
  Pendiente({
    required this.id,
    required this.texto,
    this.vence,
    this.hecho = false,
  });

  final String id;
  String texto;
  DateTime? vence;
  bool hecho;

  /// Días que faltan. Negativo si ya venció.
  int? get diasPara {
    if (vence == null) return null;
    final hoy = DateTime.now();
    final base = DateTime(hoy.year, hoy.month, hoy.day);
    return vence!.difference(base).inDays;
  }

  Map<String, dynamic> aJson() => {
        'id': id,
        'texto': texto,
        'vence': vence?.toIso8601String(),
        'hecho': hecho,
      };

  static Pendiente deJson(Map<String, dynamic> j) => Pendiente(
        id: j['id'] as String,
        texto: j['texto'] as String? ?? '',
        vence: j['vence'] == null ? null : DateTime.tryParse(j['vence'] as String),
        hecho: j['hecho'] as bool? ?? false,
      );
}

class Movimiento {
  Movimiento({
    required this.id,
    required this.persona,
    required this.concepto,
    required this.monto,
    required this.esEntrega,
    required this.fecha,
  });

  final String id;
  String persona;
  String concepto;
  int monto;

  /// True si me lo dieron; false si yo lo pagué.
  bool esEntrega;
  DateTime fecha;

  Map<String, dynamic> aJson() => {
        'id': id,
        'persona': persona,
        'concepto': concepto,
        'monto': monto,
        'esEntrega': esEntrega,
        'fecha': fecha.toIso8601String(),
      };

  static Movimiento deJson(Map<String, dynamic> j) => Movimiento(
        id: j['id'] as String,
        persona: j['persona'] as String? ?? '',
        concepto: j['concepto'] as String? ?? '',
        monto: (j['monto'] as num?)?.toInt() ?? 0,
        esEntrega: j['esEntrega'] as bool? ?? true,
        fecha: DateTime.tryParse(j['fecha'] as String? ?? '') ?? DateTime.now(),
      );
}

class Renglon {
  Renglon({required this.id, required this.concepto, required this.monto, this.recortable = false});
  final String id;
  String concepto;
  int monto;
  bool recortable;

  Map<String, dynamic> aJson() =>
      {'id': id, 'concepto': concepto, 'monto': monto, 'recortable': recortable};

  static Renglon deJson(Map<String, dynamic> j) => Renglon(
        id: j['id'] as String,
        concepto: j['concepto'] as String? ?? '',
        monto: (j['monto'] as num?)?.toInt() ?? 0,
        recortable: j['recortable'] as bool? ?? false,
      );
}

/// Todo el estado de la app en un objeto: se guarda como un solo JSON.
class Estado {
  Estado({
    required this.acreedores,
    required this.pendientes,
    required this.movimientos,
    required this.ingresos,
    required this.gastos,
  });

  List<Acreedor> acreedores;
  List<Pendiente> pendientes;
  List<Movimiento> movimientos;
  List<Renglon> ingresos;
  List<Renglon> gastos;

  int get ingresoMensual => ingresos.fold(0, (a, r) => a + r.monto);
  int get gastoMensual => gastos.fold(0, (a, r) => a + r.monto);
  int get gastoRecortable =>
      gastos.where((r) => r.recortable).fold(0, (a, r) => a + r.monto);

  Iterable<Acreedor> get vivos => acreedores.where((a) => !a.etapa.liquidada);

  int get pagosMensuales =>
      vivos.where((a) => !a.porNomina).fold(0, (a, x) => a + x.pagoMensual);

  int get deudaPorPagar => vivos.fold(0, (a, x) => a + x.montoAPagar);
  int get deudaLiquidada => acreedores
      .where((a) => a.etapa.liquidada)
      .fold(0, (a, x) => a + x.montoAPagar);
  int get ahorroPorQuitas => acreedores.fold(0, (a, x) => a + x.ahorro);
  int get capacidadMensual => ingresoMensual - gastoMensual;

  int saldoDe(String persona) {
    var s = 0;
    for (final m in movimientos.where((m) => m.persona == persona)) {
      s += m.esEntrega ? m.monto : -m.monto;
    }
    return s < 0 ? 0 : s;
  }

  List<String> get personas =>
      movimientos.map((m) => m.persona).toSet().toList()..sort();

  String aTexto() => jsonEncode({
        'version': 1,
        'acreedores': acreedores.map((a) => a.aJson()).toList(),
        'pendientes': pendientes.map((p) => p.aJson()).toList(),
        'movimientos': movimientos.map((m) => m.aJson()).toList(),
        'ingresos': ingresos.map((r) => r.aJson()).toList(),
        'gastos': gastos.map((r) => r.aJson()).toList(),
      });

  static Estado deTexto(String texto) {
    final j = jsonDecode(texto) as Map<String, dynamic>;
    List<T> lista<T>(String clave, T Function(Map<String, dynamic>) de) =>
        ((j[clave] as List?) ?? const [])
            .map((e) => de(e as Map<String, dynamic>))
            .toList();
    return Estado(
      acreedores: lista('acreedores', Acreedor.deJson),
      pendientes: lista('pendientes', Pendiente.deJson),
      movimientos: lista('movimientos', Movimiento.deJson),
      ingresos: lista('ingresos', Renglon.deJson),
      gastos: lista('gastos', Renglon.deJson),
    );
  }
}
