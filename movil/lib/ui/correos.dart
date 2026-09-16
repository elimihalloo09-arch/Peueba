import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'comunes.dart';

class _Correo {
  const _Correo(this.titulo, this.para, this.cuerpo);
  final String titulo;
  final String para;
  final String cuerpo;
}

const _correos = <_Correo>[
  _Correo(
    'HeyBanco · aceptar la quita',
    'nancy.avila@hey.inc · Asunto: Aceptación de Opción 1, liquidación con quita',
    '''Estimada Nancy Nelly Ávila Almaguer:

Acuso recibo de su correo del 15 de septiembre de 2026 y le confirmo que ACEPTO LA OPCIÓN 1: Liquidación con 70% de descuento.

Para proceder con el pago, le solicito que me envíe por este mismo medio el convenio de liquidación por escrito, con los siguientes elementos:

1. Monto exacto a pagar y fecha límite de vigencia de la oferta.
2. Manifestación expresa de que ese pago liquida TOTALMENTE el adeudo y cancela el contrato, sin saldos remanentes de ningún tipo.
3. Datos oficiales de pago a nombre de HEY BANCO (CLABE institucional y referencia).
4. Compromiso y plazo de emisión de la carta finiquito.
5. Compromiso de reportar la cuenta ante las sociedades de información crediticia con la clave que corresponda a cuenta liquidada.
6. La constancia de condonación para efectos fiscales, en su caso.

En cuanto reciba el convenio con esos puntos, realizo la transferencia y le envío el comprobante el mismo día.

Quedo atento a su respuesta.

Carlos Hernández Montes''',
  ),
  _Correo(
    'Klar · antes de la cesión',
    'Cobranza Klar · Asunto: Solicitud de convenio de liquidación antes de cesión',
    '''Equipo de Cobranza Klar:

Recibí su aviso sobre los 207 días de atraso, el saldo vencido de \$14,610.88 y la advertencia de cesión a una agencia externa.

Quiero resolver esta cuenta y prefiero hacerlo directamente con ustedes, antes de que sea cedida. Les solicito su mejor propuesta de LIQUIDACIÓN CON QUITA, por escrito.

Con transparencia: estoy regularizando todas mis obligaciones al mismo tiempo y con recursos limitados. Ya cuento con una oferta formal de liquidación con 70% de descuento de otra institución, y voy a priorizar las cuentas que me ofrezcan condiciones de liquidación definitiva.

La propuesta debe incluir: monto exacto, vigencia, manifestación de que liquida totalmente el adeudo y cancela el contrato, datos oficiales de pago a nombre de la institución, plazo de entrega de la carta finiquito, y el compromiso de reportar la cuenta como liquidada ante las sociedades de información crediticia.

En cuanto reciba el convenio, pago y envío comprobante el mismo día.

Quedo atento.

Carlos Hernández Montes''',
  ),
  _Correo(
    'Plantilla · NU, Coppel y Fintopia',
    'Asunto: Solicitud de convenio de liquidación',
    '''A quien corresponda:

Me dirijo a ustedes para resolver de forma definitiva el adeudo de la cuenta [referencia], que reconozco y que se encuentra con atraso.

Mi situación es la siguiente: estoy regularizando todas mis obligaciones al mismo tiempo, con recursos limitados y provenientes de un apoyo familiar por única vez. Eso me permite liquidar de contado, pero no pagar la totalidad de los saldos.

Por ello solicito su propuesta de LIQUIDACIÓN CON QUITA, por escrito. Estoy en posibilidad de pagar en una sola exhibición dentro de los próximos días si las condiciones lo permiten. Ya cuento con ofertas formales de otras instituciones con descuentos del 70%.

El convenio debe indicar: monto exacto, vigencia, que el pago liquida totalmente el adeudo y cancela el contrato, datos oficiales de pago a nombre de la institución, plazo de la carta finiquito, y el reporte de la cuenta como liquidada ante las sociedades de información crediticia.

Quedo atento.

Carlos Hernández Montes''',
  ),
];

class PantallaCorreos extends StatelessWidget {
  const PantallaCorreos({super.key});

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        const Encabezado('Correos listos', nota: 'Toca para copiar'),
        for (final c in _correos)
          Card(
            child: Padding(
              padding: const EdgeInsets.all(14),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(c.titulo, style: const TextStyle(fontSize: 15, fontWeight: FontWeight.w700)),
                  const SizedBox(height: 4),
                  Text(c.para,
                      style: TextStyle(fontSize: 11.5, color: Theme.of(context).colorScheme.outline)),
                  const SizedBox(height: 10),
                  Container(
                    constraints: const BoxConstraints(maxHeight: 170),
                    width: double.infinity,
                    padding: const EdgeInsets.all(11),
                    decoration: BoxDecoration(
                      color: Theme.of(context).colorScheme.surfaceContainerHighest,
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: SingleChildScrollView(
                      child: Text(c.cuerpo, style: const TextStyle(fontSize: 12, height: 1.45)),
                    ),
                  ),
                  const SizedBox(height: 10),
                  SizedBox(
                    width: double.infinity,
                    child: FilledButton.icon(
                      icon: const Icon(Icons.copy, size: 18),
                      label: const Text('Copiar'),
                      onPressed: () async {
                        await Clipboard.setData(ClipboardData(text: c.cuerpo));
                        if (context.mounted) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            const SnackBar(content: Text('Copiado. Pégalo en tu correo.')),
                          );
                        }
                      },
                    ),
                  ),
                ],
              ),
            ),
          ),
        Card(
          color: ambar.withOpacity(0.12),
          child: const Padding(
            padding: EdgeInsets.all(14),
            child: Text(
              'Antes de pagar cualquier convenio: verifica la CLABE por canal oficial del banco, '
              'nunca por el que venga en el correo. Y no pagues sin tener el convenio por escrito.',
              style: TextStyle(fontSize: 13),
            ),
          ),
        ),
      ],
    );
  }
}
