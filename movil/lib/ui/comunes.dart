import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

const azul = Color(0xFF0B8F9C);
const ambar = Color(0xFFBE7314);
const violeta = Color(0xFF6257BE);
const verde = Color(0xFF2E7443);
const rojo = Color(0xFFB23A30);

final _pesos = NumberFormat.currency(locale: 'es_MX', symbol: '\$', decimalDigits: 0);
final _pesosCentavos = NumberFormat.currency(locale: 'es_MX', symbol: '\$', decimalDigits: 2);

/// Recibe centavos y devuelve pesos legibles.
String dinero(int centavos, {bool conCentavos = false}) =>
    conCentavos ? _pesosCentavos.format(centavos / 100) : _pesos.format(centavos / 100);

ThemeData tema(Brightness brillo) {
  final base = ColorScheme.fromSeed(seedColor: azul, brightness: brillo);
  return ThemeData(
    colorScheme: base,
    useMaterial3: true,
    scaffoldBackgroundColor: brillo == Brightness.dark
        ? const Color(0xFF0C1214)
        : const Color(0xFFE7EBEC),
    cardTheme: CardTheme(
      elevation: 0,
      margin: const EdgeInsets.only(bottom: 12),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
      color: brillo == Brightness.dark ? const Color(0xFF151E21) : Colors.white,
    ),
    inputDecorationTheme: const InputDecorationTheme(
      isDense: true,
      border: OutlineInputBorder(),
    ),
  );
}

/// Rótulo chico en mayúsculas, para encabezar una cifra.
class Rotulo extends StatelessWidget {
  const Rotulo(this.texto, {super.key});
  final String texto;

  @override
  Widget build(BuildContext context) => Text(
        texto.toUpperCase(),
        style: TextStyle(
          fontSize: 10,
          letterSpacing: 1.4,
          fontWeight: FontWeight.w500,
          color: Theme.of(context).colorScheme.outline,
        ),
      );
}

/// Una cifra grande con su rótulo y, si hace falta, una nota abajo.
class Cifra extends StatelessWidget {
  const Cifra({
    super.key,
    required this.rotulo,
    required this.valor,
    this.nota,
    this.color,
  });

  final String rotulo;
  final String valor;
  final String? nota;
  final Color? color;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Rotulo(rotulo),
        const SizedBox(height: 6),
        Text(
          valor,
          style: TextStyle(
            fontSize: 26,
            fontWeight: FontWeight.w800,
            letterSpacing: -0.8,
            color: color,
          ),
        ),
        if (nota != null) ...[
          const SizedBox(height: 4),
          Text(
            nota!,
            style: TextStyle(fontSize: 12, color: Theme.of(context).colorScheme.outline),
          ),
        ],
      ],
    );
  }
}

/// Encabezado de sección con una línea abajo.
class Encabezado extends StatelessWidget {
  const Encabezado(this.titulo, {super.key, this.nota});
  final String titulo;
  final String? nota;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(top: 8, bottom: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              Expanded(
                child: Text(
                  titulo,
                  style: const TextStyle(fontSize: 17, fontWeight: FontWeight.w700, letterSpacing: -0.3),
                ),
              ),
              if (nota != null)
                Text(nota!, style: TextStyle(fontSize: 12, color: Theme.of(context).colorScheme.outline)),
            ],
          ),
          const SizedBox(height: 8),
          Container(height: 1.5, color: Theme.of(context).colorScheme.onSurface),
        ],
      ),
    );
  }
}

/// Pide un monto en pesos y devuelve centavos.
Future<int?> pedirMonto(BuildContext context, String titulo, {int inicial = 0}) async {
  final control = TextEditingController(
    text: inicial == 0 ? '' : (inicial / 100).toStringAsFixed(0),
  );
  return showDialog<int>(
    context: context,
    builder: (c) => AlertDialog(
      title: Text(titulo),
      content: TextField(
        controller: control,
        autofocus: true,
        keyboardType: const TextInputType.numberWithOptions(decimal: true),
        decoration: const InputDecoration(prefixText: '\$ ', labelText: 'Pesos'),
      ),
      actions: [
        TextButton(onPressed: () => Navigator.pop(c), child: const Text('Cancelar')),
        FilledButton(
          onPressed: () {
            final v = double.tryParse(control.text.replaceAll(',', '')) ?? 0;
            Navigator.pop(c, (v * 100).round());
          },
          child: const Text('Guardar'),
        ),
      ],
    ),
  );
}

/// Pide un texto corto.
Future<String?> pedirTexto(BuildContext context, String titulo, {String inicial = ''}) async {
  final control = TextEditingController(text: inicial);
  return showDialog<String>(
    context: context,
    builder: (c) => AlertDialog(
      title: Text(titulo),
      content: TextField(controller: control, autofocus: true),
      actions: [
        TextButton(onPressed: () => Navigator.pop(c), child: const Text('Cancelar')),
        FilledButton(
          onPressed: () => Navigator.pop(c, control.text.trim()),
          child: const Text('Guardar'),
        ),
      ],
    ),
  );
}
