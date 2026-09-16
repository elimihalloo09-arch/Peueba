import 'package:flutter/material.dart';

import '../datos/repositorio.dart';
import '../modelo/datos.dart';
import 'comunes.dart';

/// La lista es para barrer con la vista: nombre, estado y cuánto. Todo lo que
/// se edita vive en la hoja que se abre al tocar, porque catorce acreedores
/// con seis fichas cada uno no se leen, se sufren.
class PantallaAcreedores extends StatelessWidget {
  const PantallaAcreedores(this.repo, {super.key});
  final Repositorio repo;

  @override
  Widget build(BuildContext context) {
    final lista = repo.estado.acreedores;
    final t = tinta(context);

    if (lista.isEmpty) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Text('Todavía no hay acreedores cargados.',
              textAlign: TextAlign.center, style: Theme.of(context).textTheme.bodyMedium),
        ),
      );
    }

    // Lo urgente primero, luego lo caro, y lo liquidado al final.
    final ordenada = [...lista]..sort((a, b) {
        if (a.etapa.liquidada != b.etapa.liquidada) return a.etapa.liquidada ? 1 : -1;
        if (a.urgente != b.urgente) return a.urgente ? -1 : 1;
        return b.tasaAnual.compareTo(a.tasaAnual);
      });

    final vivos = ordenada.where((a) => !a.etapa.liquidada).length;

    return ListView(
      padding: const EdgeInsets.fromLTRB(16, 4, 16, 28),
      children: [
        Encabezado('$vivos por cerrar', nota: 'Toca uno para trabajarlo'),
        Panel(
          padding: 0,
          hijos: [for (final a in ordenada) _Renglon(repo: repo, acreedor: a)],
        ),
        const SizedBox(height: 14),
        Text(
          'Ordenados por urgencia y por tasa: arriba lo que más cuesta dejar vivo.',
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.bodySmall?.copyWith(color: t.texto3),
        ),
      ],
    );
  }
}

/// Color y palabra del estado. El color hace el trabajo de un vistazo.
({Color color, String texto}) _estado(BuildContext c, Acreedor a) {
  final t = tinta(c);
  if (a.etapa.liquidada) return (color: t.acento, texto: a.etapa.texto);
  if (a.urgente) return (color: t.urgente, texto: 'Urgente');
  return switch (a.etapa) {
    Etapa.conOferta => (color: t.proceso, texto: 'Con oferta'),
    Etapa.convenio => (color: t.ganado, texto: 'Convenio'),
    Etapa.contactado => (color: t.proceso, texto: 'Contactado'),
    _ => (color: t.texto3, texto: a.porNomina ? 'Por nómina' : 'Sin contactar'),
  };
}

class _Renglon extends StatelessWidget {
  const _Renglon({required this.repo, required this.acreedor});
  final Repositorio repo;
  final Acreedor acreedor;

  @override
  Widget build(BuildContext context) {
    final a = acreedor;
    final t = tinta(context);
    final tt = Theme.of(context).textTheme;
    final e = _estado(context, a);
    final hayQuita = a.ahorro > 0;

    return InkWell(
      onTap: () => _abrirDetalle(context, repo, a),
      child: Padding(
        padding: const EdgeInsets.fromLTRB(14, 13, 14, 13),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Franja de color: dice el estado sin ocupar una palabra.
            Container(
              width: 3,
              height: 38,
              margin: const EdgeInsets.only(right: 12, top: 2),
              decoration: BoxDecoration(
                  color: e.color, borderRadius: BorderRadius.circular(2)),
            ),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(a.nombre,
                      style: tt.titleMedium?.copyWith(
                          decoration: a.etapa.liquidada ? TextDecoration.lineThrough : null,
                          color: a.etapa.liquidada ? t.texto3 : null)),
                  const SizedBox(height: 6),
                  Wrap(
                    spacing: 6,
                    runSpacing: 4,
                    crossAxisAlignment: WrapCrossAlignment.center,
                    children: [
                      Pastilla(e.texto, color: e.color),
                      if (a.tasaAnual > 0)
                        Text('${a.tasaAnual.toStringAsFixed(a.tasaAnual % 1 == 0 ? 0 : 1)}% anual',
                            style: tt.bodySmall?.copyWith(
                                color: a.tasaAnual >= 90 ? t.urgente : t.texto3,
                                fontWeight: a.tasaAnual >= 90 ? FontWeight.w700 : null)),
                    ],
                  ),
                ],
              ),
            ),
            const SizedBox(width: 10),
            Column(
              crossAxisAlignment: CrossAxisAlignment.end,
              children: [
                Text(dinero(a.montoAPagar),
                    style: tt.titleLarge?.copyWith(
                        fontSize: 17,
                        color: a.etapa.liquidada ? t.texto3 : t.texto)),
                if (hayQuita) ...[
                  const SizedBox(height: 3),
                  Text(dinero(a.saldoOriginal),
                      style: tt.bodySmall?.copyWith(
                          decoration: TextDecoration.lineThrough, color: t.texto3)),
                ],
              ],
            ),
          ],
        ),
      ),
    );
  }
}

/// La hoja donde de verdad se trabaja el acreedor.
Future<void> _abrirDetalle(BuildContext context, Repositorio repo, Acreedor a) {
  return showModalBottomSheet<void>(
    context: context,
    isScrollControlled: true,
    showDragHandle: true,
    backgroundColor: tinta(context).superficie,
    shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20))),
    builder: (c) => DraggableScrollableSheet(
      expand: false,
      initialChildSize: 0.8,
      maxChildSize: 0.95,
      builder: (c, control) => _Detalle(repo: repo, acreedor: a, control: control),
    ),
  );
}

class _Detalle extends StatefulWidget {
  const _Detalle({required this.repo, required this.acreedor, required this.control});
  final Repositorio repo;
  final Acreedor acreedor;
  final ScrollController control;

  @override
  State<_Detalle> createState() => _DetalleState();
}

class _DetalleState extends State<_Detalle> {
  Acreedor get a => widget.acreedor;

  void _cambiar(void Function() accion) {
    widget.repo.cambiar((_) => accion());
    setState(() {});
  }

  /// Con los tres documentos en mano, la cuenta está cerrada de verdad.
  void _cerrarSiCompleto() {
    if (a.tieneConvenio && a.tieneComprobante && a.tieneFiniquito) {
      a.etapa = Etapa.finiquito;
    }
  }

  @override
  Widget build(BuildContext context) {
    final t = tinta(context);
    final tt = Theme.of(context).textTheme;
    final e = _estado(context, a);

    return ListView(
      controller: widget.control,
      padding: const EdgeInsets.fromLTRB(20, 4, 20, 32),
      children: [
        Text(a.nombre, style: tt.headlineMedium?.copyWith(fontSize: 22)),
        const SizedBox(height: 10),
        Wrap(spacing: 8, runSpacing: 6, children: [
          Pastilla(e.texto, color: e.color),
          if (a.porNomina) Pastilla('Por nómina', color: t.texto3, relleno: false),
          if (!a.admiteQuita) Pastilla('Sin quita', color: t.texto3, relleno: false),
        ]),
        const SizedBox(height: 20),

        Panel(hijos: [
          Fila('A pagar', dinero(a.montoAPagar), color: t.texto,
              alTocar: () async {
            final m = await pedirMonto(context, 'A pagar', inicial: a.montoAPagar);
            if (m != null) _cambiar(() => a.montoAPagar = m);
          }),
          Fila('Saldo sin quita', dinero(a.saldoOriginal), alTocar: () async {
            final m = await pedirMonto(context, 'Saldo sin quita', inicial: a.saldoOriginal);
            if (m != null) _cambiar(() => a.saldoOriginal = m);
          }),
          Fila('Pago mensual', dinero(a.pagoMensual), alTocar: () async {
            final m = await pedirMonto(context, 'Pago mensual', inicial: a.pagoMensual);
            if (m != null) _cambiar(() => a.pagoMensual = m);
          }),
          if (a.ahorro > 0)
            Fila('Te ahorras', dinero(a.ahorro), color: t.ganado,
                nota: 'si la quita se firma'),
        ]),

        if (a.admiteQuita) ...[
          const SizedBox(height: 20),
          const Rotulo('Qué tanto te perdonan'),
          Row(children: [
            Expanded(
              child: Slider(
                value: a.saldoOriginal == 0
                    ? 0
                    : (1 - a.montoAPagar / a.saldoOriginal).clamp(0.0, 0.95),
                max: 0.95,
                divisions: 19,
                onChanged: (v) => _cambiar(
                    () => a.montoAPagar = (a.saldoOriginal * (1 - v)).round()),
              ),
            ),
            SizedBox(
              width: 48,
              child: Text(
                a.saldoOriginal == 0
                    ? '—'
                    : '${((1 - a.montoAPagar / a.saldoOriginal) * 100).round()}%',
                textAlign: TextAlign.right,
                style: tt.titleMedium?.copyWith(fontWeight: FontWeight.w700),
              ),
            ),
          ]),
        ],

        const SizedBox(height: 20),
        const Rotulo('En qué va la negociación'),
        const SizedBox(height: 8),
        Wrap(spacing: 7, runSpacing: 7, children: [
          for (final x in Etapa.values)
            ChoiceChip(
              label: Text(x.texto),
              selected: a.etapa == x,
              onSelected: (_) => _cambiar(() => a.etapa = x),
            ),
        ]),

        const SizedBox(height: 20),
        const Rotulo('Papeles que ya tienes'),
        const SizedBox(height: 8),
        Wrap(spacing: 7, runSpacing: 7, children: [
          _papel('Convenio', a.tieneConvenio,
              () => _cambiar(() { a.tieneConvenio = !a.tieneConvenio; _cerrarSiCompleto(); })),
          _papel('Comprobante', a.tieneComprobante,
              () => _cambiar(() { a.tieneComprobante = !a.tieneComprobante; _cerrarSiCompleto(); })),
          _papel('Carta finiquito', a.tieneFiniquito,
              () => _cambiar(() { a.tieneFiniquito = !a.tieneFiniquito; _cerrarSiCompleto(); })),
        ]),

        const SizedBox(height: 20),
        const Rotulo('Nota'),
        const SizedBox(height: 8),
        InkWell(
          onTap: () async {
            final texto = await pedirTexto(context, 'Nota de ${a.nombre}', inicial: a.nota);
            if (texto != null) _cambiar(() => a.nota = texto);
          },
          child: Container(
            width: double.infinity,
            padding: const EdgeInsets.all(13),
            decoration: BoxDecoration(
              color: t.fondo,
              borderRadius: BorderRadius.circular(11),
              border: Border.all(color: t.borde),
            ),
            child: Text(a.nota.isEmpty ? 'Toca para escribir una nota' : a.nota,
                style: a.nota.isEmpty
                    ? tt.bodySmall
                    : tt.bodyMedium?.copyWith(color: t.texto)),
          ),
        ),
      ],
    );
  }

  Widget _papel(String texto, bool activo, VoidCallback alTocar) => FilterChip(
        label: Text(texto),
        selected: activo,
        showCheckmark: true,
        onSelected: (_) => alTocar(),
      );
}
