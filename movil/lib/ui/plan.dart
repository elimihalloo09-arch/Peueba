import 'package:flutter/material.dart';

import '../datos/repositorio.dart';
import '../modelo/simulador.dart';
import 'comunes.dart';

class PantallaPlan extends StatefulWidget {
  const PantallaPlan(this.repo, {super.key});
  final Repositorio repo;

  @override
  State<PantallaPlan> createState() => _PantallaPlanState();
}

class _PantallaPlanState extends State<PantallaPlan> {
  // En pesos: se convierten a centavos al simular.
  double capacidad = 13352;
  double apoyo = 10000;
  double mesesApoyo = 6;
  Metodo metodo = Metodo.avalancha;
  bool sinQuitas = false;

  Simulacion _correr() {
    final deudas = widget.repo.estado.vivos
        .where((a) => !a.porNomina)
        .map((a) => DeudaSimulada(
              nombre: a.nombre,
              saldo: sinQuitas ? a.saldoOriginal : a.montoAPagar,
              pagoMensual: a.pagoMensual,
              tasaAnual: a.tasaAnual,
              urgente: a.urgente,
            ))
        .toList();
    return simular(
      deudas: deudas,
      capacidad: (capacidad * 100).round(),
      metodo: metodo,
      apoyoMensual: (apoyo * 100).round(),
      mesesApoyo: mesesApoyo.round(),
    );
  }

  @override
  Widget build(BuildContext context) {
    final s = _correr();
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        const Encabezado('Plan de liquidación', nota: 'Mueve y vuelve a calcular'),
        Card(
          child: Padding(
            padding: const EdgeInsets.fromLTRB(14, 10, 14, 14),
            child: Column(
              children: [
                _barra('Al mes para deuda', capacidad, 0, 40000, 400,
                    (v) => setState(() => capacidad = v)),
                _barra('Apoyo mensual', apoyo, 0, 30000, 300,
                    (v) => setState(() => apoyo = v)),
                _barra('Meses de apoyo', mesesApoyo, 0, 24, 24,
                    (v) => setState(() => mesesApoyo = v), esDinero: false),
                const SizedBox(height: 4),
                SegmentedButton<Metodo>(
                  segments: const [
                    ButtonSegment(value: Metodo.avalancha, label: Text('Tasa más alta')),
                    ButtonSegment(value: Metodo.bolaDeNieve, label: Text('Saldo más chico')),
                  ],
                  selected: {metodo},
                  onSelectionChanged: (v) => setState(() => metodo = v.first),
                ),
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  title: const Text('Sin quitas, con los saldos completos',
                      style: TextStyle(fontSize: 13.5)),
                  value: sinQuitas,
                  onChanged: (v) => setState(() => sinQuitas = v),
                ),
              ],
            ),
          ),
        ),
        Row(
          children: [
            Expanded(
              child: Card(
                child: Padding(
                  padding: const EdgeInsets.all(14),
                  child: Cifra(
                    rotulo: 'Libre en',
                    valor: s.inalcanzable ? 'no alcanza' : '${s.meses} meses',
                    color: s.inalcanzable ? rojo : verde,
                  ),
                ),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Card(
                child: Padding(
                  padding: const EdgeInsets.all(14),
                  child: Cifra(rotulo: 'Intereses', valor: dinero(s.intereses), color: rojo),
                ),
              ),
            ),
          ],
        ),
        if (s.inalcanzable)
          Card(
            color: rojo.withOpacity(0.12),
            child: Padding(
              padding: const EdgeInsets.all(14),
              child: Text(
                'Con ese abono no se liquidan: ${s.atoradas.join(', ')}. '
                'Los intereses crecen más rápido que el pago.',
                style: const TextStyle(fontSize: 13.5),
              ),
            ),
          ),
        const SizedBox(height: 8),
        const Encabezado('Orden de pago'),
        for (var i = 0; i < s.orden.length; i++)
          Card(
            child: ListTile(
              dense: true,
              leading: CircleAvatar(
                radius: 14,
                backgroundColor: azul.withOpacity(0.15),
                child: Text('${i + 1}', style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w700)),
              ),
              title: Text(s.orden[i].nombre, style: const TextStyle(fontSize: 14)),
              subtitle: Text('Pagas ${dinero(s.orden[i].totalPagado)}'),
              trailing: Text('mes ${s.orden[i].mes}',
                  style: TextStyle(fontSize: 12.5, color: Theme.of(context).colorScheme.outline)),
            ),
          ),
      ],
    );
  }

  Widget _barra(String rotulo, double valor, double min, double max, int divisiones,
      ValueChanged<double> alCambiar,
      {bool esDinero = true}) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(child: Rotulo(rotulo)),
            Text(
              esDinero ? dinero((valor * 100).round()) : valor.round().toString(),
              style: const TextStyle(fontWeight: FontWeight.w700, fontSize: 15),
            ),
          ],
        ),
        Slider(value: valor, min: min, max: max, divisions: divisiones, onChanged: alCambiar),
      ],
    );
  }
}
