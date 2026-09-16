import 'package:flutter/material.dart';

import '../datos/repositorio.dart';
import '../modelo/datos.dart';
import 'comunes.dart';

class PantallaAcreedores extends StatelessWidget {
  const PantallaAcreedores(this.repo, {super.key});
  final Repositorio repo;

  @override
  Widget build(BuildContext context) {
    final lista = repo.estado.acreedores;
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Encabezado('Acreedores', nota: '${lista.length} · toca la etapa'),
        for (final a in lista) _Tarjeta(repo: repo, acreedor: a),
      ],
    );
  }
}

class _Tarjeta extends StatelessWidget {
  const _Tarjeta({required this.repo, required this.acreedor});
  final Repositorio repo;
  final Acreedor acreedor;

  Color _colorEtapa(Etapa e) => switch (e) {
        Etapa.conOferta => ambar,
        Etapa.convenio => violeta,
        Etapa.pagado || Etapa.finiquito => verde,
        _ => Colors.grey,
      };

  String _subtitulo() {
    final a = acreedor;
    if (a.porNomina) return 'Se descuenta de nómina';
    if (!a.admiteQuita) return 'Sin quita';
    if (a.ahorro <= 0) return 'Falta negociar la quita';
    final firme = a.etapa == Etapa.convenio || a.etapa.liquidada;
    return firme
        ? 'Quita de ${dinero(a.ahorro)}'
        : 'Quita estimada de ${dinero(a.ahorro)}, por negociar';
  }

  @override
  Widget build(BuildContext context) {
    final a = acreedor;
    return Card(
      child: Padding(
        padding: const EdgeInsets.fromLTRB(14, 14, 14, 10),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(a.nombre,
                          style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w700, letterSpacing: -0.2)),
                      const SizedBox(height: 3),
                      Text(_subtitulo(),
                          style: TextStyle(fontSize: 11.5, color: Theme.of(context).colorScheme.outline)),
                    ],
                  ),
                ),
                Column(
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    Text(
                      dinero(a.montoAPagar),
                      style: TextStyle(
                        fontSize: 18,
                        fontWeight: FontWeight.w800,
                        color: a.etapa.liquidada ? verde : null,
                      ),
                    ),
                    if (a.ahorro > 0)
                      Text(
                        dinero(a.saldoOriginal),
                        style: TextStyle(
                          fontSize: 11,
                          decoration: TextDecoration.lineThrough,
                          color: Theme.of(context).colorScheme.outline,
                        ),
                      ),
                  ],
                ),
              ],
            ),
            const SizedBox(height: 12),
            Wrap(
              spacing: 6,
              runSpacing: 6,
              children: [
                for (final e in Etapa.values)
                  ChoiceChip(
                    label: Text(e.texto, style: const TextStyle(fontSize: 11.5)),
                    selected: a.etapa == e,
                    selectedColor: _colorEtapa(e).withOpacity(0.22),
                    onSelected: (_) => repo.cambiar((_) => a.etapa = e),
                  ),
              ],
            ),
            const SizedBox(height: 10),
            const Rotulo('Papeles que ya tienes'),
            const SizedBox(height: 6),
            Wrap(
              spacing: 6,
              children: [
                _doc(context, 'Convenio', a.tieneConvenio, () {
                  repo.cambiar((_) {
                    a.tieneConvenio = !a.tieneConvenio;
                    _cerrarSiCompleto(a);
                  });
                }),
                _doc(context, 'Comprobante', a.tieneComprobante, () {
                  repo.cambiar((_) {
                    a.tieneComprobante = !a.tieneComprobante;
                    _cerrarSiCompleto(a);
                  });
                }),
                _doc(context, 'Finiquito', a.tieneFiniquito, () {
                  repo.cambiar((_) {
                    a.tieneFiniquito = !a.tieneFiniquito;
                    _cerrarSiCompleto(a);
                  });
                }),
              ],
            ),
            const SizedBox(height: 6),
            Row(
              children: [
                Expanded(
                  child: TextButton(
                    onPressed: () async {
                      final v = await pedirMonto(context, 'Saldo original', inicial: a.saldoOriginal);
                      if (v != null) repo.cambiar((_) => a.saldoOriginal = v);
                    },
                    child: Text('Saldo ${dinero(a.saldoOriginal)}', style: const TextStyle(fontSize: 12.5)),
                  ),
                ),
                Expanded(
                  child: TextButton(
                    onPressed: () async {
                      final v = await pedirMonto(context, 'Monto a pagar', inicial: a.montoAPagar);
                      if (v != null) repo.cambiar((_) => a.montoAPagar = v);
                    },
                    child: Text('Pagar ${dinero(a.montoAPagar)}', style: const TextStyle(fontSize: 12.5)),
                  ),
                ),
              ],
            ),
            if (a.admiteQuita)
              Row(
                children: [
                  const Text('Quita', style: TextStyle(fontSize: 12.5)),
                  Expanded(
                    child: Slider(
                      value: a.quitaPorcentaje.clamp(0, 100).toDouble(),
                      max: 100,
                      divisions: 20,
                      label: '${a.quitaPorcentaje}%',
                      onChanged: (v) => repo.cambiar(
                        (_) => a.montoAPagar = (a.saldoOriginal * (1 - v / 100)).round(),
                      ),
                    ),
                  ),
                  SizedBox(
                    width: 42,
                    child: Text('${a.quitaPorcentaje}%',
                        textAlign: TextAlign.end, style: const TextStyle(fontSize: 12.5)),
                  ),
                ],
              ),
            if (a.nota.isNotEmpty)
              Padding(
                padding: const EdgeInsets.only(top: 4),
                child: Text(a.nota,
                    style: TextStyle(fontSize: 12.5, color: Theme.of(context).colorScheme.outline)),
              ),
            Align(
              alignment: Alignment.centerRight,
              child: TextButton.icon(
                icon: const Icon(Icons.edit_note, size: 18),
                label: const Text('Nota', style: TextStyle(fontSize: 12.5)),
                onPressed: () async {
                  final t = await pedirTexto(context, 'Nota de ${a.nombre}', inicial: a.nota);
                  if (t != null) repo.cambiar((_) => a.nota = t);
                },
              ),
            ),
          ],
        ),
      ),
    );
  }

  /// Con los tres documentos en mano, la cuenta está cerrada de verdad.
  void _cerrarSiCompleto(Acreedor a) {
    if (a.tieneConvenio && a.tieneComprobante && a.tieneFiniquito) {
      a.etapa = Etapa.finiquito;
    }
  }

  Widget _doc(BuildContext context, String texto, bool activo, VoidCallback alTocar) {
    return FilterChip(
      label: Text(texto, style: const TextStyle(fontSize: 11.5)),
      selected: activo,
      showCheckmark: true,
      selectedColor: verde.withOpacity(0.2),
      onSelected: (_) => alTocar(),
    );
  }
}
