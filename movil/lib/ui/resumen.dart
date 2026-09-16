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
    final queda = e.capacidadMensual - e.pagosMensuales;

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        const Encabezado('El mes', nota: 'Ingreso del hogar contra lo que sale'),
        Row(
          children: [
            Expanded(
              child: Card(
                child: Padding(
                  padding: const EdgeInsets.all(14),
                  child: Cifra(rotulo: 'Entra', valor: dinero(e.ingresoMensual)),
                ),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Card(
                child: Padding(
                  padding: const EdgeInsets.all(14),
                  child: Cifra(
                    rotulo: 'Se va',
                    valor: dinero(e.gastoMensual),
                    nota: '${dinero(e.gastoRecortable)} recortable',
                  ),
                ),
              ),
            ),
          ],
        ),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(14),
            child: Cifra(
              rotulo: 'Te queda después de gastos y mínimos',
              valor: dinero(queda),
              color: queda < 0 ? rojo : verde,
              nota: queda < 0
                  ? 'El mes no cierra: hay que recortar o negociar'
                  : 'Esto es lo que puede atacar deuda',
            ),
          ),
        ),
        const SizedBox(height: 10),
        const Encabezado('La deuda', nota: 'Con las quitas ya aplicadas'),
        Row(
          children: [
            Expanded(
              child: Card(
                child: Padding(
                  padding: const EdgeInsets.all(14),
                  child: Cifra(rotulo: 'Por pagar', valor: dinero(e.deudaPorPagar), color: rojo),
                ),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Card(
                child: Padding(
                  padding: const EdgeInsets.all(14),
                  child: Cifra(rotulo: 'Ya liquidado', valor: dinero(e.deudaLiquidada), color: verde),
                ),
              ),
            ),
          ],
        ),
        Row(
          children: [
            Expanded(
              child: Card(
                child: Padding(
                  padding: const EdgeInsets.all(14),
                  child: Cifra(
                    rotulo: 'Ahorrado en quitas',
                    valor: dinero(e.ahorroPorQuitas),
                    color: violeta,
                  ),
                ),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Card(
                child: Padding(
                  padding: const EdgeInsets.all(14),
                  child: Cifra(
                    rotulo: 'Mínimos al mes',
                    valor: dinero(e.pagosMensuales),
                    nota: 'sin lo de nómina',
                  ),
                ),
              ),
            ),
          ],
        ),
        if (e.personas.isNotEmpty) ...[
          const SizedBox(height: 10),
          const Encabezado('Préstamos de personas'),
          for (final p in e.personas)
            Card(
              child: ListTile(
                title: Text(p, style: const TextStyle(fontWeight: FontWeight.w600)),
                subtitle: Text(
                  'Te dio ${dinero(e.movimientos.where((m) => m.persona == p && m.esEntrega).fold(0, (a, m) => a + m.monto))} · '
                  'le has pagado ${dinero(e.movimientos.where((m) => m.persona == p && !m.esEntrega).fold(0, (a, m) => a + m.monto))}',
                ),
                trailing: Text(
                  dinero(e.saldoDe(p)),
                  style: const TextStyle(fontWeight: FontWeight.w700, color: rojo, fontSize: 16),
                ),
              ),
            ),
          Card(
            child: ListTile(
              leading: const Icon(Icons.add),
              title: const Text('Registrar entrega o pago'),
              onTap: () => _registrar(context),
            ),
          ),
        ],
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
