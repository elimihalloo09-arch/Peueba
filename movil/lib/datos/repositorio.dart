import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../modelo/datos.dart';
import 'semilla.dart';

/// Guarda todo el estado como un solo JSON en el aparato. Sin servidor: la app
/// funciona completa sin internet, y las cifras no salen de aquí.
class Repositorio extends ChangeNotifier {
  Repositorio._(this._prefs, this.estado, this.arrancado);

  static const _clave = 'estado_finanzas_v1';
  static const _claveArrancado = 'arrancado_v1';

  final SharedPreferences _prefs;
  Estado estado;

  /// Falso solo hasta que se carga un respaldo o se decide empezar de cero.
  /// Distingue "todavía no dice qué quiere" de "empezó vacío a propósito".
  bool arrancado;
  bool guardando = false;

  static Future<Repositorio> abrir() async {
    final prefs = await SharedPreferences.getInstance();
    final texto = prefs.getString(_clave);
    Estado estado;
    if (texto == null) {
      estado = vacio();
    } else {
      try {
        estado = Estado.deTexto(texto);
      } catch (_) {
        // Si lo guardado quedó corrupto, mejor arrancar vacío que dejar la
        // app inservible: el respaldo se puede volver a cargar.
        estado = vacio();
      }
    }
    return Repositorio._(prefs, estado, prefs.getBool(_claveArrancado) ?? false);
  }

  Future<void> guardar() async {
    guardando = true;
    notifyListeners();
    await _prefs.setString(_clave, estado.aTexto());
    guardando = false;
    notifyListeners();
  }

  /// Cambia algo y guarda. Es el único camino para modificar el estado.
  Future<void> cambiar(void Function(Estado e) accion) async {
    accion(estado);
    await guardar();
  }

  Future<void> arrancar() async {
    arrancado = true;
    await _prefs.setBool(_claveArrancado, true);
    await guardar();
  }

  Future<void> restablecer() async {
    estado = vacio();
    arrancado = false;
    await _prefs.setBool(_claveArrancado, false);
    await guardar();
  }

  String exportar() => estado.aTexto();

  Future<bool> importar(String texto) async {
    try {
      estado = Estado.deTexto(texto);
      await arrancar();
      return true;
    } catch (_) {
      return false;
    }
  }
}
