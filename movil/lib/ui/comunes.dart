
import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

// ---------------------------------------------------------------------------
// Color
//
// Un solo acento, el jade, que significa "vas bien": lo que te queda, lo que
// ahorras, lo que ya liquidaste. Los estados van aparte y no compiten con el:
// rojo lo que urge, ambar lo que esta en proceso, violeta lo que ganaste
// negociando. El gris no es neutro puro, lleva una pizca de verde para que
// pertenezca a la misma familia y no se sienta prestado.
// ---------------------------------------------------------------------------

class Tinta {
  const Tinta({
    required this.fondo,
    required this.superficie,
    required this.borde,
    required this.texto,
    required this.texto2,
    required this.texto3,
    required this.acento,
    required this.urgente,
    required this.proceso,
    required this.ganado,
  });

  final Color fondo, superficie, borde, texto, texto2, texto3;
  final Color acento, urgente, proceso, ganado;

  static const claro = Tinta(
    fondo: Color(0xFFEDF1F0),
    superficie: Color(0xFFFFFFFF),
    borde: Color(0xFFDCE3E1),
    texto: Color(0xFF0E1513),
    texto2: Color(0xFF47534F),
    texto3: Color(0xFF75817D),
    acento: Color(0xFF0A7D6B),
    urgente: Color(0xFFB23A30),
    proceso: Color(0xFFA0640B),
    ganado: Color(0xFF574BC9),
  );

  static const oscuro = Tinta(
    fondo: Color(0xFF080D0C),
    superficie: Color(0xFF121A18),
    borde: Color(0xFF202A27),
    texto: Color(0xFFE7ECEA),
    texto2: Color(0xFFA3AEAB),
    texto3: Color(0xFF76817E),
    acento: Color(0xFF3FD9BC),
    urgente: Color(0xFFF08074),
    proceso: Color(0xFFE0A63C),
    ganado: Color(0xFFA79BFF),
  );
}

/// El tema vivo, para que las pantallas pidan color sin preguntar el brillo.
Tinta tinta(BuildContext c) =>
    Theme.of(c).brightness == Brightness.dark ? Tinta.oscuro : Tinta.claro;

// Nombres que ya usaban las pantallas. Se resuelven contra el tema claro y se
// corrigen por contexto donde importa.
const azul = Color(0xFF0A7D6B);
const ambar = Color(0xFFA0640B);
const violeta = Color(0xFF574BC9);
const verde = Color(0xFF0A7D6B);
const rojo = Color(0xFFB23A30);

// ---------------------------------------------------------------------------
// Dinero
// ---------------------------------------------------------------------------

final _pesos = NumberFormat.currency(locale: 'es_MX', symbol: '\$', decimalDigits: 0);
final _pesosCentavos = NumberFormat.currency(locale: 'es_MX', symbol: '\$', decimalDigits: 2);

/// Recibe centavos y devuelve pesos legibles.
String dinero(int centavos, {bool conCentavos = false}) =>
    conCentavos ? _pesosCentavos.format(centavos / 100) : _pesos.format(centavos / 100);

/// Las cifras siempre con figuras de ancho fijo: asi una columna de montos
/// alinea sus unidades aunque cambien los digitos.
const _tabular = [FontFeature.tabularFigures()];

// ---------------------------------------------------------------------------
// Tipografia
//
// Bricolage Grotesque para lo que se lee de un vistazo: titulos y cifras. Es
// estrecha y de contrastes marcados, asi que un numero grande cabe entero en
// la pantalla del telefono sin encogerse. Public Sans para todo lo demas, que
// es lo que se lee de cerca.
// ---------------------------------------------------------------------------

const _display = 'Bricolage';
const _texto = 'PublicSans';

TextTheme _tipografia(Tinta t) => TextTheme(
      displayLarge: TextStyle(
          fontFamily: _display, fontSize: 40, fontWeight: FontWeight.w800,
          letterSpacing: -1.6, height: 1.05, color: t.texto, fontFeatures: _tabular),
      headlineMedium: TextStyle(
          fontFamily: _display, fontSize: 27, fontWeight: FontWeight.w700,
          letterSpacing: -0.9, height: 1.1, color: t.texto, fontFeatures: _tabular),
      titleLarge: TextStyle(
          fontFamily: _display, fontSize: 19, fontWeight: FontWeight.w700,
          letterSpacing: -0.4, color: t.texto),
      titleMedium: TextStyle(
          fontFamily: _texto, fontSize: 15, fontWeight: FontWeight.w600,
          letterSpacing: -0.1, color: t.texto),
      bodyLarge: TextStyle(
          fontFamily: _texto, fontSize: 15, height: 1.5, color: t.texto),
      bodyMedium: TextStyle(
          fontFamily: _texto, fontSize: 13.5, height: 1.5, color: t.texto2),
      bodySmall: TextStyle(
          fontFamily: _texto, fontSize: 12, height: 1.45, color: t.texto3),
      labelLarge: TextStyle(
          fontFamily: _texto, fontSize: 14, fontWeight: FontWeight.w600, color: t.texto),
      labelSmall: TextStyle(
          fontFamily: _texto, fontSize: 10.5, fontWeight: FontWeight.w600,
          letterSpacing: 1.1, color: t.texto3),
    );

ThemeData tema(Brightness brillo) {
  final t = brillo == Brightness.dark ? Tinta.oscuro : Tinta.claro;
  final esquema = ColorScheme(
    brightness: brillo,
    primary: t.acento,
    onPrimary: brillo == Brightness.dark ? const Color(0xFF00201B) : Colors.white,
    secondary: t.ganado,
    onSecondary: Colors.white,
    error: t.urgente,
    onError: Colors.white,
    surface: t.superficie,
    onSurface: t.texto,
    outline: t.texto3,
    outlineVariant: t.borde,
  );

  return ThemeData(
    useMaterial3: true,
    brightness: brillo,
    colorScheme: esquema,
    scaffoldBackgroundColor: t.fondo,
    canvasColor: t.fondo,
    fontFamily: _texto,
    textTheme: _tipografia(t),
    splashFactory: InkSparkle.splashFactory,

    appBarTheme: AppBarTheme(
      backgroundColor: t.fondo,
      surfaceTintColor: Colors.transparent,
      elevation: 0,
      scrolledUnderElevation: 0,
      centerTitle: false,
      titleTextStyle: TextStyle(
          fontFamily: _display, fontSize: 21, fontWeight: FontWeight.w800,
          letterSpacing: -0.6, color: t.texto),
      iconTheme: IconThemeData(color: t.texto2),
    ),

    // Las tarjetas dejan de ser cajas con sombra: son superficies con un
    // borde de un pelo, que separan sin gritar.
    cardTheme: CardTheme(
      elevation: 0,
      margin: EdgeInsets.zero,
      color: t.superficie,
      surfaceTintColor: Colors.transparent,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(14),
        side: BorderSide(color: t.borde),
      ),
    ),

    dividerTheme: DividerThemeData(color: t.borde, thickness: 1, space: 1),

    navigationBarTheme: NavigationBarThemeData(
      backgroundColor: t.superficie,
      surfaceTintColor: Colors.transparent,
      indicatorColor: t.acento.withOpacity(brillo == Brightness.dark ? 0.22 : 0.14),
      elevation: 0,
      height: 66,
      labelBehavior: NavigationDestinationLabelBehavior.alwaysShow,
      labelTextStyle: WidgetStateProperty.resolveWith((e) => TextStyle(
            fontFamily: _texto,
            fontSize: 10.5,
            fontWeight: e.contains(WidgetState.selected) ? FontWeight.w700 : FontWeight.w500,
            color: e.contains(WidgetState.selected) ? t.texto : t.texto3,
          )),
      iconTheme: WidgetStateProperty.resolveWith((e) => IconThemeData(
            size: 22,
            color: e.contains(WidgetState.selected) ? t.acento : t.texto3,
          )),
    ),

    chipTheme: ChipThemeData(
      backgroundColor: Colors.transparent,
      selectedColor: t.acento.withOpacity(brillo == Brightness.dark ? 0.22 : 0.13),
      side: BorderSide(color: t.borde),
      // Sin color explicito, la ficha sin seleccionar heredaba tinta oscura
      // sobre fondo oscuro y no se leia.
      labelStyle: TextStyle(
          fontFamily: _texto, fontSize: 12.5, fontWeight: FontWeight.w600, color: t.texto2),
      secondaryLabelStyle: TextStyle(
          fontFamily: _texto, fontSize: 12.5, fontWeight: FontWeight.w700, color: t.texto),
      checkmarkColor: t.acento,
      shape: const StadiumBorder(),
      padding: const EdgeInsets.symmetric(horizontal: 11, vertical: 7),
      showCheckmark: false,
    ),

    sliderTheme: SliderThemeData(
      activeTrackColor: t.acento,
      inactiveTrackColor: t.borde,
      thumbColor: t.acento,
      overlayColor: t.acento.withOpacity(0.12),
      trackHeight: 3,
      activeTickMarkColor: Colors.transparent,
      inactiveTickMarkColor: Colors.transparent,
    ),

    filledButtonTheme: FilledButtonThemeData(
      style: FilledButton.styleFrom(
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(11)),
        textStyle: const TextStyle(fontFamily: _texto, fontSize: 15, fontWeight: FontWeight.w600),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        side: BorderSide(color: t.borde),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(11)),
        textStyle: const TextStyle(fontFamily: _texto, fontSize: 15, fontWeight: FontWeight.w600),
      ),
    ),
    textButtonTheme: TextButtonThemeData(
      style: TextButton.styleFrom(
        foregroundColor: t.acento,
        textStyle: const TextStyle(fontFamily: _texto, fontSize: 14, fontWeight: FontWeight.w600),
      ),
    ),

    inputDecorationTheme: InputDecorationTheme(
      isDense: true,
      filled: true,
      fillColor: t.fondo,
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(10),
        borderSide: BorderSide(color: t.borde),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(10),
        borderSide: BorderSide(color: t.borde),
      ),
    ),

    dialogTheme: DialogTheme(
      backgroundColor: t.superficie,
      surfaceTintColor: Colors.transparent,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
    ),
    snackBarTheme: SnackBarThemeData(
      behavior: SnackBarBehavior.floating,
      backgroundColor: t.texto,
      contentTextStyle: TextStyle(fontFamily: _texto, color: t.fondo, fontSize: 13.5),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
    ),
  );
}

// ---------------------------------------------------------------------------
// Piezas
// ---------------------------------------------------------------------------

/// Rótulo chico en mayúsculas, para encabezar una cifra.
class Rotulo extends StatelessWidget {
  const Rotulo(this.texto, {super.key, this.color});
  final String texto;
  final Color? color;

  @override
  Widget build(BuildContext context) => Text(
        texto.toUpperCase(),
        style: Theme.of(context).textTheme.labelSmall?.copyWith(color: color),
      );
}

/// Un número con su rótulo. Es la unidad con la que se arma cada tablero.
class Cifra extends StatelessWidget {
  const Cifra({
    super.key,
    required this.rotulo,
    required this.valor,
    this.nota,
    this.color,
    this.grande = false,
  });

  final String rotulo;
  final String valor;
  final String? nota;
  final Color? color;
  final bool grande;

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Rotulo(rotulo),
        const SizedBox(height: 7),
        FittedBox(
          fit: BoxFit.scaleDown,
          alignment: Alignment.centerLeft,
          child: Text(valor,
              maxLines: 1,
              style: (grande ? tt.displayLarge : tt.headlineMedium)?.copyWith(color: color)),
        ),
        if (nota != null) ...[
          const SizedBox(height: 5),
          Text(nota!, style: tt.bodySmall),
        ],
      ],
    );
  }
}

/// Encabezado de sección: el título manda, la nota acompaña.
class Encabezado extends StatelessWidget {
  const Encabezado(this.titulo, {super.key, this.nota});
  final String titulo;
  final String? nota;

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;
    return Padding(
      padding: const EdgeInsets.only(top: 6, bottom: 12),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          // El titulo se queda con la mayor parte del ancho y puede bajar a
          // dos lineas antes que recortarse; la nota cede primero.
          Flexible(
            flex: 3,
            child: Text(titulo, style: tt.titleLarge, maxLines: 2),
          ),
          if (nota != null) ...[
            const SizedBox(width: 12),
            Flexible(
              flex: 2,
              child: Text(nota!,
                  textAlign: TextAlign.right,
                  maxLines: 2,
                  style: tt.bodySmall?.copyWith(height: 1.25)),
            ),
          ],
        ],
      ),
    );
  }
}

/// Etiqueta de estado. El color dice qué tan urgente es sin leer la palabra.
class Pastilla extends StatelessWidget {
  const Pastilla(this.texto, {super.key, required this.color, this.relleno = true});
  final String texto;
  final Color color;
  final bool relleno;

  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 9, vertical: 4),
        decoration: BoxDecoration(
          color: relleno ? color.withOpacity(0.13) : null,
          border: relleno ? null : Border.all(color: color.withOpacity(0.5)),
          borderRadius: BorderRadius.circular(999),
        ),
        child: Text(
          texto,
          style: TextStyle(
            fontFamily: _texto,
            fontSize: 11,
            fontWeight: FontWeight.w700,
            letterSpacing: 0.1,
            color: color,
          ),
        ),
      );
}

/// Una superficie agrupada. Reemplaza al montón de tarjetas sueltas: lo que va
/// junto se lee junto, separado apenas por una línea.
class Panel extends StatelessWidget {
  const Panel({super.key, required this.hijos, this.padding = 14});
  final List<Widget> hijos;
  final double padding;

  @override
  Widget build(BuildContext context) {
    final t = tinta(context);
    return Container(
      decoration: BoxDecoration(
        color: t.superficie,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: t.borde),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          for (var i = 0; i < hijos.length; i++) ...[
            if (i > 0) Divider(height: 1, color: t.borde),
            Padding(padding: EdgeInsets.all(padding), child: hijos[i]),
          ],
        ],
      ),
    );
  }
}

/// Renglón de etiqueta y valor, alineado y con cifras de ancho fijo.
class Fila extends StatelessWidget {
  const Fila(this.etiqueta, this.valor, {super.key, this.color, this.nota, this.alTocar});
  final String etiqueta;
  final String valor;
  final Color? color;
  final String? nota;
  final VoidCallback? alTocar;

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;
    final cuerpo = Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(etiqueta, style: tt.titleMedium),
              if (nota != null) ...[
                const SizedBox(height: 3),
                Text(nota!, style: tt.bodySmall),
              ],
            ],
          ),
        ),
        const SizedBox(width: 12),
        Text(valor,
            style: tt.titleMedium?.copyWith(
                fontWeight: FontWeight.w700, color: color, fontFeatures: _tabular)),
      ],
    );
    if (alTocar == null) return cuerpo;
    return InkWell(onTap: alTocar, child: cuerpo);
  }
}

// ---------------------------------------------------------------------------
// Diálogos
// ---------------------------------------------------------------------------

Future<int?> pedirMonto(BuildContext context, String titulo, {int inicial = 0}) async {
  final control = TextEditingController(
      text: inicial == 0 ? '' : (inicial / 100).toStringAsFixed(2));
  final valor = await showDialog<String>(
    context: context,
    builder: (c) => AlertDialog(
      title: Text(titulo, style: Theme.of(c).textTheme.titleLarge),
      content: TextField(
        controller: control,
        autofocus: true,
        keyboardType: const TextInputType.numberWithOptions(decimal: true),
        decoration: const InputDecoration(prefixText: '\$ '),
        onSubmitted: (v) => Navigator.pop(c, v),
      ),
      actions: [
        TextButton(onPressed: () => Navigator.pop(c), child: const Text('Cancelar')),
        FilledButton(
            onPressed: () => Navigator.pop(c, control.text), child: const Text('Guardar')),
      ],
    ),
  );
  if (valor == null) return null;
  final n = double.tryParse(valor.replaceAll(',', '').trim());
  return n == null ? null : (n * 100).round();
}

Future<String?> pedirTexto(BuildContext context, String titulo, {String inicial = ''}) async {
  final control = TextEditingController(text: inicial);
  return showDialog<String>(
    context: context,
    builder: (c) => AlertDialog(
      title: Text(titulo, style: Theme.of(c).textTheme.titleLarge),
      content: TextField(
        controller: control,
        autofocus: true,
        maxLines: null,
        onSubmitted: (v) => Navigator.pop(c, v),
      ),
      actions: [
        TextButton(onPressed: () => Navigator.pop(c), child: const Text('Cancelar')),
        FilledButton(
            onPressed: () => Navigator.pop(c, control.text), child: const Text('Guardar')),
      ],
    ),
  );
}
