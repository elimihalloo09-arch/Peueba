import 'package:flutter/material.dart';

import '../datos/repositorio.dart';
import '../modelo/datos.dart';
import 'comunes.dart';

class PantallaResumen extends StatelessWidget {
  const PantallaResumen(this.repo, {super.key});
  final Repositorio repo;

  @override
  Widget build(BuildContext context) {
    final e = repo.estado;
    final t = tinta(context);
    final tt = Theme.of(context).textTheme;
    final queda = e.capacidadMensual - e.pagosMensuales;
    final cierra = queda >= 0;

    return ListView(
      padding: const EdgeInsets.fromLTRB(16, 4, 16, 28),
      children: [
        // Una sola cifra manda la pantalla: si el mes cierra o no. Todo lo
        // demas esta para explicarla.
        _Portada(queda: queda, cierra: cierra, faltaPara: e.gastoRecortable),
        const SizedBox(height: 22),

        const Encabezado('De dónde sale', nota: 'El mes, renglón por renglón'),
        Panel(hijos: [
          Fila('Entra al hogar', dinero(e.ingresoMensual), color: t.acento),
          Fila('Se va en gastos', '−${dinero(e.gastoMensual)}',
              nota: '${dinero(e.gastoRecortable)} de eso es recortable'),
          Fila('Mínimos de deuda', '−${dinero(e.pagosMensuales)}',
              nota: 'sin contar lo que se descuenta de nómina'),
        ]),
        const SizedBox(height: 22),

        const Encabezado('La deuda', nota: 'Con las quitas ya aplicadas'),
        _Barra(
          porPagar: e.deudaPorPagar,
          liquidado: e.deudaLiquidada,
          ahorrado: e.ahorroPorQuitas,
        ),
        const SizedBox(height: 12),
        Panel(hijos: [
          Fila('Por pagar', dinero(e.deudaPorPagar), color: t.urgente),
          if (e.deudaLiquidada > 0)
            Fila('Ya liquidado', dinero(e.deudaLiquidada), color: t.acento),
          Fila('Ahorrado en quitas', dinero(e.ahorroPorQuitas), color: t.ganado,
              nota: 'lo que ya no vas a pagar por negociar'),
        ]),

        if (e.personas.isNotEmpty) ...[
          const SizedBox(height: 22),
          const Encabezado('Préstamos de personas', nota: 'Fuera del buró, pero se pagan'),
          Panel(hijos: [
            for (final p in e.personas)
              Fila(
                p,
                dinero(e.saldoDe(p)),
                color: t.urgente,
                nota: 'te dio ${dinero(e.movimientos.where((m) => m.persona == p && m.esEntrega).fold(0, (a, m) => a + m.monto))}'
                    ' · le has pagado ${dinero(e.movimientos.where((m) => m.persona == p && !m.esEntrega).fold(0, (a, m) => a + m.monto))}',
              ),
          ]),
          const SizedBox(height: 10),
          OutlinedButton.icon(
            onPressed: () => _registrar(context),
            icon: const Icon(Icons.add, size: 18),
            label: const Text('Registrar entrega o pago'),
            style: OutlinedButton.styleFrom(minimumSize: const Size.fromHeight(46)),
          ),
        ] else ...[
          const SizedBox(height: 22),
          OutlinedButton.icon(
            onPressed: () => _registrar(context),
            icon: const Icon(Icons.add, size: 18),
            label: const Text('Registrar un préstamo de alguien'),
            style: OutlinedButton.styleFrom(minimumSize: const Size.fromHeight(46)),
          ),
        ],
        const SizedBox(height: 8),
        Text(
          'Todo vive en este aparato. Respalda desde el menú de arriba.',
          textAlign: TextAlign.center,
          style: tt.bodySmall,
        ),
      ],
    );
  }

  Future<void> _registrar(BuildContext context) async {
    final persona = await pedirTexto(context, '¿Con quién?', inicial: '');
    if (persona == null || persona.isEmpty || !context.mounted) return;
    final monto = await pedirMonto(context, 'Monto');
    if (monto == null || monto <= 0 || !context.mounted) return;
    final esEntrega = await showDialog<bool>(
      context: context,
      builder: (c) => AlertDialog(
        title: const Text('¿Qué fue?'),
        actions: [
          TextButton(onPressed: () => Navigator.pop(c, true), child: const Text('Me dio')),
          FilledButton(onPressed: () => Navigator.pop(c, false), child: const Text('Le pagué')),
        ],
      ),
    );
    if (esEntrega == null) return;
    await repo.cambiar((e) => e.movimientos.add(Movimiento(
          id: DateTime.now().microsecondsSinceEpoch.toString(),
          persona: persona,
          concepto: esEntrega ? 'Entrega' : 'Pago',
          monto: monto,
          esEntrega: esEntrega,
          fecha: DateTime.now(),
        )));
  }
}

/// El veredicto del mes, en grande. Es lo primero y lo unico que hay que
/// entender al abrir la app.
class _Portada extends StatelessWidget {
  const _Portada({required this.queda, required this.cierra, required this.faltaPara});
  final int queda;
  final bool cierra;
  final int faltaPara;

  @override
  Widget build(BuildContext context) {
    final t = tinta(context);
    final tt = Theme.of(context).textTheme;
    final color = cierra ? t.acento : t.urgente;

    return Container(
      width: double.infinity,
      padding: const EdgeInsets.fromLTRB(18, 20, 18, 18),
      decoration: BoxDecoration(
        color: t.superficie,
        borderRadius: BorderRadius.circular(18),
        border: Border.all(color: color.withOpacity(0.35)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(width: 7, height: 7,
                  decoration: BoxDecoration(color: color, shape: BoxShape.circle)),
              const SizedBox(width: 8),
              Rotulo(cierra ? 'El mes cierra' : 'El mes no cierra', color: color),
            ],
          ),
          const SizedBox(height: 14),
          FittedBox(
            fit: BoxFit.scaleDown,
            alignment: Alignment.centerLeft,
            child: Text(dinero(queda), maxLines: 1,
                style: tt.displayLarge?.copyWith(color: color)),
          ),
          const SizedBox(height: 10),
          Text(
            cierra
                ? 'Esto es lo que te queda libre cada mes para atacar deuda.'
                : 'Esto falta cada mes después de gastos y mínimos. '
                    'Tienes ${dinero(faltaPara)} de gasto recortable para cubrirlo.',
            style: tt.bodyMedium,
          ),
        ],
      ),
    );
  }
}

/// Cuanto del camino llevas. Sin numeros: la proporcion se ve.
class _Barra extends StatelessWidget {
  const _Barra({required this.porPagar, required this.liquidado, required this.ahorrado});
  final int porPagar, liquidado, ahorrado;

  @override
  Widget build(BuildContext context) {
    final t = tinta(context);
    final total = porPagar + liquidado + ahorrado;
    if (total <= 0) return const SizedBox.shrink();

    // Los pesos van en centavos, asi que el flex seria de millones. Se
    // reparte sobre mil para que el reparto quede en numeros manejables.
    int peso(int centavos) => (centavos * 1000 / total).round();

    Widget trozo(int parte, Color color) => parte <= 0
        ? const SizedBox.shrink()
        : Expanded(flex: peso(parte), child: Container(color: color));

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        ClipRRect(
          borderRadius: BorderRadius.circular(999),
          child: SizedBox(
            width: double.infinity,
            height: 10,
            child: Row(mainAxisSize: MainAxisSize.max, children: [
              trozo(porPagar, t.urgente),
              trozo(liquidado, t.acento),
              trozo(ahorrado, t.ganado),
            ]),
          ),
        ),
        const SizedBox(height: 8),
        Wrap(spacing: 14, runSpacing: 4, alignment: WrapAlignment.start, children: [
          _Punto('Por pagar', t.urgente),
          if (liquidado > 0) _Punto('Liquidado', t.acento),
          _Punto('Quitas', t.ganado),
        ]),
      ],
    );
  }
}

class _Punto extends StatelessWidget {
  const _Punto(this.texto, this.color);
  final String texto;
  final Color color;

  @override
  Widget build(BuildContext context) => Row(mainAxisSize: MainAxisSize.min, children: [
        Container(width: 7, height: 7,
            decoration: BoxDecoration(color: color, shape: BoxShape.circle)),
        const SizedBox(width: 5),
        Text(texto, style: Theme.of(context).textTheme.bodySmall),
      ]);
}
