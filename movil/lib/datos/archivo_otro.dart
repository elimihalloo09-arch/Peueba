/// En Android y iOS no hay descarga ni selector de archivos del navegador.
/// La app sigue compilando; la pantalla ofrece copiar y pegar en su lugar.
bool get hayArchivos => false;

void descargar(String nombre, String contenido) {}

Future<String?> abrirArchivo() async => null;
