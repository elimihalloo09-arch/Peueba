import 'package:flutter/material.dart';

import '../datos/repositorio.dart';
import 'acreedores.dart';
import 'comunes.dart';
import 'correos.dart';
import 'pendientes.dart';
import 'plan.dart';
import 'resumen.dart';

class AppFinanzas extends StatelessWidget {
  const AppFinanzas(this.repo, {super.key});
  final Repositorio repo;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Finanzas',
      debugShowCheckedModeBanner: false,
      theme: tema(Brightness.light),
      darkTheme: tema(Brightness.dark),
      home: _Marco(repo),
    );
  }
}

class _Marco extends StatefulWidget {
  const _Marco(this.repo);
  final Repositorio repo;

  @override
  State<_Marco> createState() => _MarcoState();
}

class _MarcoState extends State<_Marco> {
  int tab = 0;

  @override
  void initState() {
    super.initState();
    // Cualquier cambio guardado vuelve a pintar la pantalla.
    widget.repo.addListener(_refrescar);
  }

  @override
  void dispose() {
    widget.repo.removeListener(_refrescar);
    super.dispose();
  }

  void _refrescar() {
    if (mounted) setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    final pantallas = [
      PantallaResumen(widget.repo),
      PantallaAcreedores(widget.repo),
      PantallaPlan(widget.repo),
      PantallaPendientes(widget.repo),
      const PantallaCorreos(),
    ];
    final faltan = widget.repo.estado.pendientes.where((p) => !p.hecho).length;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Finanzas', style: TextStyle(fontWeight: FontWeight.w800, letterSpacing: -0.5)),
        actions: [
          if (widget.repo.guardando)
            const Padding(
              padding: EdgeInsets.only(right: 16),
              child: Center(child: SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2))),
            ),
          PopupMenuButton<String>(
            onSelected: (v) async {
              if (v == 'restablecer') {
                final ok = await showDialog<bool>(
                  context: context,
                  builder: (c) => AlertDialog(
                    title: const Text('¿Restablecer todo?'),
                    content: const Text('Se pierde lo que hayas cambiado y vuelven los datos iniciales.'),
                    actions: [
                      TextButton(onPressed: () => Navigator.pop(c, false), child: const Text('No')),
                      FilledButton(onPressed: () => Navigator.pop(c, true), child: const Text('Sí')),
                    ],
                  ),
                );
                if (ok ?? false) await widget.repo.restablecer();
              }
            },
            itemBuilder: (c) => const [
              PopupMenuItem(value: 'restablecer', child: Text('Restablecer datos')),
            ],
          ),
        ],
      ),
      body: IndexedStack(index: tab, children: pantallas),
      bottomNavigationBar: NavigationBar(
        selectedIndex: tab,
        onDestinationSelected: (i) => setState(() => tab = i),
        destinations: [
          const NavigationDestination(icon: Icon(Icons.pie_chart_outline), selectedIcon: Icon(Icons.pie_chart), label: 'Resumen'),
          const NavigationDestination(icon: Icon(Icons.account_balance_outlined), selectedIcon: Icon(Icons.account_balance), label: 'Deudas'),
          const NavigationDestination(icon: Icon(Icons.timeline_outlined), selectedIcon: Icon(Icons.timeline), label: 'Plan'),
          NavigationDestination(
            icon: Badge(
              isLabelVisible: faltan > 0,
              label: Text('$faltan'),
              child: const Icon(Icons.check_box_outlined),
            ),
            selectedIcon: const Icon(Icons.check_box),
            label: 'Hoy',
          ),
          const NavigationDestination(icon: Icon(Icons.mail_outline), selectedIcon: Icon(Icons.mail), label: 'Correos'),
        ],
      ),
    );
  }
}
