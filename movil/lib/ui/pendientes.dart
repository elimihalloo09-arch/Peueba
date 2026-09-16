import 'package:flutter/material.dart';

import '../datos/repositorio.dart';
import '../modelo/datos.dart';
import 'comunes.dart';

class PantallaPendientes extends StatelessWidget {
  const PantallaPendientes(this.repo, {super.key});
  final Repositorio repo;

  String _etiqueta(Pendiente p) {
    final d = p.diasPara;
    if (d == null) return '';
    if (p.hecho) return '';
    if (d < 0) return 'vencido';
    if (d == 0) return 'hoy';
    if (d == 1) return 'mañana';
    return 'en $d d';
  }

  Color? _color(Pendiente p) {
    final d = p.diasPara;
    if (d == null || p.hecho) return null;
    if (d <= 1) return rojo;
    if (d <= 7) return ambar;
    return null;
  }

  @override
  Widget build(BuildContext context) {
    final lista = repo.estado.pendientes;
    final faltan = lista.where((p) => !p.hecho).length;

    return Scaffold(
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Encabezado('Pendientes', nota: '$faltan sin hacer'),
          for (final p in lista)
            Card(
              child: ListTile(
                leading: Checkbox(
                  value: p.hecho,
                  onChanged: (_) => repo.cambiar((_) => p.hecho = !p.hecho),
                ),
                title: Text(
                  p.texto,
                  style: TextStyle(
                    fontSize: 14.5,
                    decoration: p.hecho ? TextDecoration.lineThrough : null,
                    color: p.hecho ? Theme.of(context).colorScheme.outline : null,
                  ),
                ),
                trailing: _etiqueta(p).isEmpty
                    ? null
                    : Text(
                        _etiqueta(p),
                        style: TextStyle(
                          fontSize: 12,
                          fontWeight: FontWeight.w600,
                          color: _color(p),
                        ),
                      ),
                onLongPress: () => repo.cambiar((e) => e.pendientes.remove(p)),
              ),
            ),
          const SizedBox(height: 8),
          Text('Mantén presionado para borrar un pendiente',
              style: TextStyle(fontSize: 12, color: Theme.of(context).colorScheme.outline)),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => _agregar(context),
        icon: const Icon(Icons.add),
        label: const Text('Pendiente'),
      ),
    );
  }

  Future<void> _agregar(BuildContext context) async {
    final texto = await pedirTexto(context, 'Qué falta hacer');
    if (texto == null || texto.isEmpty || !context.mounted) return;
    final fecha = await showDatePicker(
      context: context,
      initialDate: DateTime.now(),
      firstDate: DateTime.now().subtract(const Duration(days: 365)),
      lastDate: DateTime.now().add(const Duration(days: 365 * 3)),
      helpText: '¿Cuándo vence? Puedes cancelar si no tiene fecha',
    );
    await repo.cambiar((e) => e.pendientes.add(Pendiente(
          id: DateTime.now().microsecondsSinceEpoch.toString(),
          texto: texto,
          vence: fecha,
        )));
  }
}
