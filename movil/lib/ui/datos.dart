import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../datos/archivo.dart';
import '../datos/repositorio.dart';
import 'comunes.dart';

/// Respaldar y restaurar. Los datos viven en el navegador de cada aparato, no
/// en la nube, asi que esta es la unica forma de moverlos entre el telefono y
/// la computadora, y de no perderlos si se limpia el navegador.
Future<void> exportar(BuildContext context, Repositorio repo) async {
  final texto = repo.exportar();
  final hoy = DateTime.now();
  final nombre = 'finanzas-${hoy.year}-${dosDigitos(hoy.month)}-${dosDigitos(hoy.day)}.json';

  if (hayArchivos) {
    descargar(nombre, texto);
    if (context.mounted) avisar(context, 'Se descargó $nombre');
    return;
  }
  await Clipboard.setData(ClipboardData(text: texto));
  if (context.mounted) avisar(context, 'Copiado. Pégalo donde lo quieras guardar.');
}

Future<void> importar(BuildContext context, Repositorio repo) async {
  String? texto;

  if (hayArchivos) {
    texto = await abrirArchivo();
  } else {
    if (!context.mounted) return;
    texto = await pedirTexto(context, 'Pega aquí el respaldo');
  }
  if (texto == null || texto.trim().isEmpty) return;

  final ok = await repo.importar(texto);
  if (!context.mounted) return;
  avisar(context, ok ? 'Datos cargados' : 'Ese archivo no se pudo leer');
}

String dosDigitos(int n) => n.toString().padLeft(2, '0');

void avisar(BuildContext context, String mensaje) {
  ScaffoldMessenger.of(context)
    ..hideCurrentSnackBar()
    ..showSnackBar(SnackBar(content: Text(mensaje)));
}

/// Lo primero que se ve en una instalacion nueva. Sin esto la app abriria en
/// ceros y no diria que hacer.
class PantallaBienvenida extends StatelessWidget {
  const PantallaBienvenida(this.repo, {super.key});
  final Repositorio repo;

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context);
    return Scaffold(
      body: SafeArea(
        child: Center(
          child: SingleChildScrollView(
            padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 32),
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 420),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Mis Finanzas',
                      style: t.textTheme.headlineMedium
                          ?.copyWith(fontWeight: FontWeight.w800, letterSpacing: -0.8)),
                  const SizedBox(height: 14),
                  Text(
                    'Esta app no trae datos de nadie. Tus cifras viven solo en '
                    'este aparato: no se suben a ningún servidor y nadie más '
                    'las ve.',
                    style: t.textTheme.bodyLarge?.copyWith(height: 1.5),
                  ),
                  const SizedBox(height: 28),
                  FilledButton.icon(
                    onPressed: () => importar(context, repo),
                    icon: const Icon(Icons.file_open_outlined),
                    label: const Text('Cargar mi respaldo'),
                    style: FilledButton.styleFrom(
                      minimumSize: const Size.fromHeight(52),
                      textStyle: const TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
                    ),
                  ),
                  const SizedBox(height: 12),
                  OutlinedButton(
                    onPressed: repo.arrancar,
                    style: OutlinedButton.styleFrom(minimumSize: const Size.fromHeight(48)),
                    child: const Text('Empezar de cero'),
                  ),
                  const SizedBox(height: 28),
                  const Rotulo('Cómo respaldar'),
                  const SizedBox(height: 8),
                  Text(
                    'Desde el menú de los tres puntos, Exportar te descarga un '
                    'archivo. Guárdalo. Es lo que te devuelve todo si limpias '
                    'el navegador o cambias de aparato.',
                    style: t.textTheme.bodyMedium?.copyWith(height: 1.5, color: t.colorScheme.outline),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
