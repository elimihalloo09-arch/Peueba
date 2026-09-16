import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../modelo/datos.dart';
import 'migraciones.dart';
import 'semilla.dart';

/// Guarda todo el estado como un solo JSON en el teléfono. Sin servidor:
/// la app funciona completa sin internet.
class Repositorio extends ChangeNotifier {
  Repositorio._(this._prefs, this.estado);

  static const _clave = 'estado_finanzas_v1';
  static const _claveRevision = 'revision_finanzas_v1';

  final SharedPreferences _prefs;
  Estado estado;
  bool guardando = false;

  static Future<Repositorio> abrir() async {
    final prefs = await SharedPreferences.getInstance();
    final texto = prefs.getString(_clave);
    Estado estado;
    var revision = revisionActual;
    if (texto == null) {
      estado = semilla();
    } else {
      try {
        estado = Estado.deTexto(texto);
        // Lo guardado antes de que existieran las revisiones es la 1.
        revision = prefs.getInt(_claveRevision) ?? 1;
      } catch (_) {
        // Si lo guardado quedó corrupto, mejor arrancar con la semilla que
        // dejar la app inservible.
        estado = semilla();
      }
    }

    final repo = Repositorio._(prefs, estado);
    if (revision < revisionActual) {
      // Corrige los datos viejos conservando lo capturado, y lo deja guardado
      // para no repetir el trabajo en cada arranque.
      migrar(estado, revision);
      await repo.guardar();
    }
    return repo;
  }

  Future<void> guardar() async {
    guardando = true;
    notifyListeners();
    await _prefs.setString(_clave, estado.aTexto());
    await _prefs.setInt(_claveRevision, revisionActual);
    guardando = false;
    notifyListeners();
  }

  /// Cambia algo y guarda. Es el único camino para modificar el estado.
  Future<void> cambiar(void Function(Estado e) accion) async {
    accion(estado);
    await guardar();
  }

  Future<void> restablecer() async {
    estado = semilla();
    await guardar();
  }

  String exportar() => estado.aTexto();

  Future<bool> importar(String texto) async {
    try {
      estado = Estado.deTexto(texto);
      await guardar();
      return true;
    } catch (_) {
      return false;
    }
  }
}
