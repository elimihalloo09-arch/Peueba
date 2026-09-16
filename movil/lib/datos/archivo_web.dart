// El aviso de no usar librerias solo-web es correcto en general; este archivo
// existe justamente para la version web y no se compila en las demas.
// ignore_for_file: avoid_web_libraries_in_flutter

import 'dart:html' as html;

/// Descargar y elegir archivos, que es como se respalda y se restaura en el
/// navegador. Solo existe en web; en el telefono nativo no hay equivalente.
bool get hayArchivos => true;

void descargar(String nombre, String contenido) {
  final blob = html.Blob(<String>[contenido], 'application/json');
  final url = html.Url.createObjectUrlFromBlob(blob);
  html.AnchorElement(href: url)
    ..setAttribute('download', nombre)
    ..click();
  html.Url.revokeObjectUrl(url);
}

Future<String?> abrirArchivo() async {
  final entrada = html.FileUploadInputElement()..accept = '.json,application/json';
  entrada.click();
  await entrada.onChange.first;
  final archivos = entrada.files;
  if (archivos == null || archivos.isEmpty) return null;
  final lector = html.FileReader()..readAsText(archivos.first);
  await lector.onLoad.first;
  final texto = lector.result;
  return texto is String ? texto : null;
}
