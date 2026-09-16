import 'package:flutter/material.dart';

import 'datos/repositorio.dart';
import 'ui/app.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final repo = await Repositorio.abrir();
  runApp(AppFinanzas(repo));
}
