import 'package:flutter/material.dart';

import '../datos/repositorio.dart';
import '../modelo/datos.dart';
import '../modelo/rutas.dart';
import 'comunes.dart';

class PantallaRutas extends StatefulWidget {
  const PantallaRutas(this.repo, {super.key});
  final Repositorio repo;

  @override
  State<PantallaRutas> createState() => _PantallaRutasState();
}

class _PantallaRutasState extends State<PantallaRutas> {
  Estado get e => widget.repo.estado;
  Viaje get v => e.viaje;

  Ruta? get rutaActual {
    if (e.rutas.isEmpty) return null;
    return e.rutas.firstWhere((r) => r.id == v.rutaBase, orElse: () => e.rutas.first);
  }

  /// Lo que se deja de gastar al mes por no pedir la aplicación.
  int ahorroMes(Ruta r) {
    final ahorro = v.uberMes - r.costoMes(v.diasPresenciales);
    return ahorro < 0 ? 0 : ahorro;
  }

  int aporteEfectivo(Ruta r) =>
      v.aporteMensual > 0 ? v.aporteMensual : ahorroMes(r);

  @override
  Widget build(BuildContext context) {
    final r = rutaActual;
    if (r == null) return _SinRutas(widget.repo, alCambiar: () => setState(() {}));

    final filas = acumular(
      aporteMensual: aporteEfectivo(r),
      meses: v.mesesAhorro,
      rendimientoAnual: v.rendimientoAnual,
    );

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        _selector(r),
        const SizedBox(height: 18),
        _camino(r),
        const SizedBox(height: 18),
        _cuesta(r),
        const SizedBox(height: 18),
        _contraLaApp(r),
        const SizedBox(height: 18),
        _acumulacion(r, filas),
      ],
    );
  }

  Widget _selector(Ruta r) => Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Encabezado('A dónde vas', nota: 'Toca para cambiar'),
          const SizedBox(height: 10),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              for (final x in e.rutas)
                ChoiceChip(
                  label: Text(x.nombre),
                  selected: x.id == r.id,
                  selectedColor: azul.withOpacity(0.22),
                  onSelected: (_) =>
                      widget.repo.cambiar((_) => v.rutaBase = x.id),
                ),
            ],
          ),
        ],
      );

  Widget _camino(Ruta r) => Card(
        child: Padding(
          padding: const EdgeInsets.all(14),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Rotulo('El camino, tramo por tramo'),
              const SizedBox(height: 10),
              for (final t in r.tramos) ...[
                _tramo(r, t),
                const Divider(height: 18),
              ],
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text('Un trayecto', style: Theme.of(context).textTheme.bodyMedium),
                  Text(dinero(r.costoTrayecto),
                      style: const TextStyle(fontWeight: FontWeight.w700)),
                ],
              ),
              const SizedBox(height: 6),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  const Text('Ida y vuelta', style: TextStyle(fontWeight: FontWeight.w600)),
                  Text(dinero(r.costoDia),
                      style: const TextStyle(fontWeight: FontWeight.w800, color: azul)),
                ],
              ),
              if (r.minutos > 0) ...[
                const SizedBox(height: 10),
                Text('${r.minutos} min por trayecto · ${(r.minutosDia / 60).toStringAsFixed(1)} h al día',
                    style: TextStyle(fontSize: 12.5, color: Theme.of(context).colorScheme.outline)),
              ],
              if (r.nota.isNotEmpty) ...[
                const SizedBox(height: 12),
                Text(r.nota, style: const TextStyle(fontSize: 12.5, height: 1.45)),
              ],
              if (r.planB.isNotEmpty) ...[
                const SizedBox(height: 10),
                Container(
                  width: double.infinity,
                  padding: const EdgeInsets.all(10),
                  decoration: BoxDecoration(
                    color: ambar.withOpacity(0.12),
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Text(r.planB, style: const TextStyle(fontSize: 12.5, height: 1.45)),
                ),
              ],
              const SizedBox(height: 10),
              Align(
                alignment: Alignment.centerRight,
                child: TextButton.icon(
                  onPressed: () => _agregarTramo(r),
                  icon: const Icon(Icons.add, size: 18),
                  label: const Text('Agregar tramo'),
                ),
              ),
            ],
          ),
        ),
      );

  Widget _tramo(Ruta r, Tramo t) => Row(
        children: [
          Expanded(
            child: InkWell(
              onTap: () async {
                final texto = await pedirTexto(context, 'Tramo', inicial: t.descripcion);
                if (texto != null) {
                  widget.repo.cambiar((_) => t.descripcion = texto);
                }
              },
              child: Text(t.descripcion, style: const TextStyle(fontSize: 13.5, height: 1.4)),
            ),
          ),
          const SizedBox(width: 10),
          InkWell(
            onTap: () async {
              final monto = await pedirMonto(context, 'Tarifa del tramo', inicial: t.tarifa);
              if (monto != null) widget.repo.cambiar((_) => t.tarifa = monto);
            },
            child: Text(dinero(t.tarifa),
                style: const TextStyle(fontWeight: FontWeight.w700, fontFeatures: [])),
          ),
          IconButton(
            visualDensity: VisualDensity.compact,
            icon: const Icon(Icons.close, size: 16),
            onPressed: () => widget.repo.cambiar((_) => r.tramos.remove(t)),
          ),
        ],
      );

  Future<void> _agregarTramo(Ruta r) async {
    final texto = await pedirTexto(context, 'Nuevo tramo');
    if (texto == null || texto.isEmpty) return;
    if (!mounted) return;
    final tarifa = await pedirMonto(context, 'Tarifa del tramo');
    widget.repo.cambiar((_) => r.tramos.add(Tramo(
          id: 't${DateTime.now().microsecondsSinceEpoch}',
          descripcion: texto,
          tarifa: tarifa ?? 0,
        )));
  }

  Widget _cuesta(Ruta r) => Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Encabezado('Lo que cuesta al mes',
              nota: 'Según los días que vas'),
          const SizedBox(height: 10),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(14),
              child: Column(
                children: [
                  _barra('Días en la oficina al mes', v.diasPresenciales.toDouble(), 0, 23, 23,
                      (x) => widget.repo.cambiar((_) => v.diasPresenciales = x.round()),
                      etiqueta: '${v.diasPresenciales}'),
                  const SizedBox(height: 14),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      const Text('Transporte al mes', style: TextStyle(fontWeight: FontWeight.w600)),
                      Text(dinero(r.costoMes(v.diasPresenciales)),
                          style: const TextStyle(
                              fontWeight: FontWeight.w800, fontSize: 20, color: azul)),
                    ],
                  ),
                  const SizedBox(height: 6),
                  Align(
                    alignment: Alignment.centerLeft,
                    child: Text(
                      '${(r.minutosMes(v.diasPresenciales) / 60).round()} horas al mes en el camino',
                      style: TextStyle(fontSize: 12.5, color: Theme.of(context).colorScheme.outline),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      );

  Widget _contraLaApp(Ruta r) => Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Encabezado('Contra pedir aplicación',
              nota: 'Lo que te ahorras por no pedirla'),
          const SizedBox(height: 10),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(14),
              child: Column(
                children: [
                  _montoEditable('Costo de ida', v.uberIda,
                      (x) => widget.repo.cambiar((_) => v.uberIda = x)),
                  const Divider(height: 18),
                  _montoEditable('Costo de regreso', v.uberRegreso,
                      (x) => widget.repo.cambiar((_) => v.uberRegreso = x)),
                  const Divider(height: 18),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      const Text('Costaría al mes'),
                      Text(dinero(v.uberMes),
                          style: const TextStyle(fontWeight: FontWeight.w700, color: rojo)),
                    ],
                  ),
                  const SizedBox(height: 12),
                  Container(
                    width: double.infinity,
                    padding: const EdgeInsets.all(12),
                    decoration: BoxDecoration(
                      color: verde.withOpacity(0.12),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Cifra(
                          rotulo: 'Te ahorras al mes',
                          valor: dinero(ahorroMes(r)),
                          nota: '${dinero(ahorroMes(r) * 12)} al año',
                          color: verde,
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      );

  Widget _acumulacion(Ruta r, List<MesAhorro> filas) {
    final fin = filas.isEmpty ? null : filas.last;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Encabezado('Si lo apartas',
            nota: 'El ahorro solo existe si lo guardas'),
        const SizedBox(height: 10),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(14),
            child: Column(
              children: [
                _barra('Apartas al mes', aporteEfectivo(r) / 100, 0, 10000, 200,
                    (x) => widget.repo.cambiar((_) => v.aporteMensual = (x * 100).round()),
                    etiqueta: dinero(aporteEfectivo(r))),
                const SizedBox(height: 12),
                _barra('Durante', v.mesesAhorro.toDouble(), 1, 120, 119,
                    (x) => widget.repo.cambiar((_) => v.mesesAhorro = x.round()),
                    etiqueta: '${v.mesesAhorro} meses'),
                const SizedBox(height: 12),
                _barra('Rendimiento anual', v.rendimientoAnual, 0, 20, 80,
                    (x) => widget.repo.cambiar((_) => v.rendimientoAnual = x),
                    etiqueta: '${v.rendimientoAnual.toStringAsFixed(1)}%'),
              ],
            ),
          ),
        ),
        const SizedBox(height: 12),
        if (fin == null)
          const Card(
            child: Padding(
              padding: EdgeInsets.all(14),
              child: Text('Pon un monto a apartar para ver cuánto se junta.'),
            ),
          )
        else ...[
          Row(
            children: [
              Expanded(
                child: Card(
                  child: Padding(
                    padding: const EdgeInsets.all(14),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Cifra(rotulo: 'Juntas', valor: dinero(fin.saldo), color: verde),
                      ],
                    ),
                  ),
                ),
              ),
              const SizedBox(width: 10),
              Expanded(
                child: Card(
                  child: Padding(
                    padding: const EdgeInsets.all(14),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Cifra(
                            rotulo: 'De eso, rendimiento',
                            valor: dinero(fin.rendimiento),
                            color: violeta),
                      ],
                    ),
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(14),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Rotulo('Cómo va creciendo'),
                  const SizedBox(height: 10),
                  for (final f in _hitos(filas))
                    Padding(
                      padding: const EdgeInsets.only(bottom: 8),
                      child: Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Text(_plazo(f.mes),
                              style: const TextStyle(fontSize: 13.5)),
                          Text(dinero(f.saldo),
                              style: const TextStyle(fontWeight: FontWeight.w700)),
                        ],
                      ),
                    ),
                ],
              ),
            ),
          ),
        ],
        const SizedBox(height: 12),
        Container(
          width: double.infinity,
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: rojo.withOpacity(0.10),
            borderRadius: BorderRadius.circular(8),
          ),
          child: const Text(
            'Mientras debas a tasas más altas que este rendimiento, el mejor '
            'lugar para este dinero es pagar deuda, no ahorrarlo. Ahorrar al '
            '10% debiendo al 128% es perder 118% al año.',
            style: TextStyle(fontSize: 12.5, height: 1.45),
          ),
        ),
      ],
    );
  }

  /// Unos pocos cortes en vez de la tabla mes a mes, que en el teléfono no se
  /// lee.
  List<MesAhorro> _hitos(List<MesAhorro> filas) {
    const cortes = [6, 12, 24, 36, 60, 120];
    final salida = <MesAhorro>[];
    for (final c in cortes) {
      if (c <= filas.length) salida.add(filas[c - 1]);
    }
    if (salida.isEmpty || salida.last.mes != filas.last.mes) salida.add(filas.last);
    return salida;
  }

  String _plazo(int meses) {
    if (meses < 12) return 'A $meses meses';
    final anios = meses ~/ 12;
    final resto = meses % 12;
    final base = anios == 1 ? 'A 1 año' : 'A $anios años';
    return resto == 0 ? base : '$base y $resto meses';
  }

  Widget _montoEditable(String etiqueta, int monto, void Function(int) alCambiar) => Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(etiqueta),
          TextButton(
            onPressed: () async {
              final x = await pedirMonto(context, etiqueta, inicial: monto);
              if (x != null) alCambiar(x);
            },
            child: Text(dinero(monto), style: const TextStyle(fontWeight: FontWeight.w700)),
          ),
        ],
      );

  Widget _barra(String rotulo, double valor, double min, double max, int pasos,
      ValueChanged<double> alCambiar,
      {required String etiqueta}) {
    final v = valor.clamp(min, max);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(child: Rotulo(rotulo)),
            Text(etiqueta,
                style: const TextStyle(fontWeight: FontWeight.w700, fontSize: 15)),
          ],
        ),
        Slider(value: v, min: min, max: max, divisions: pasos, onChanged: alCambiar),
      ],
    );
  }
}

/// Lo que se ve cuando todavía no hay ninguna ruta cargada.
class _SinRutas extends StatelessWidget {
  const _SinRutas(this.repo, {required this.alCambiar});
  final Repositorio repo;
  final VoidCallback alCambiar;

  @override
  Widget build(BuildContext context) => Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Text('Todavía no hay rutas',
                  style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700)),
              const SizedBox(height: 10),
              Text(
                'Agrega el camino de tu casa al trabajo, tramo por tramo, para '
                'ver lo que cuesta y lo que te ahorras.',
                textAlign: TextAlign.center,
                style: TextStyle(color: Theme.of(context).colorScheme.outline, height: 1.45),
              ),
              const SizedBox(height: 20),
              FilledButton.icon(
                onPressed: () async {
                  final nombre = await pedirTexto(context, 'Nombre del destino');
                  if (nombre == null || nombre.isEmpty) return;
                  final id = 'r${DateTime.now().microsecondsSinceEpoch}';
                  await repo.cambiar((e) {
                    e.rutas.add(Ruta(id: id, nombre: nombre));
                    e.viaje.rutaBase = id;
                  });
                  alCambiar();
                },
                icon: const Icon(Icons.add),
                label: const Text('Agregar destino'),
              ),
            ],
          ),
        ),
      );
}
