import '../modelo/datos.dart';

/// Un estado en blanco.
///
/// La app no trae cifras de nadie: se cargan desde un respaldo o se capturan
/// a mano. Así el código se puede publicar sin exponer la situación
/// financiera de quien lo usa.
Estado vacio() => Estado(
      acreedores: [],
      pendientes: [],
      movimientos: [],
      ingresos: [],
      gastos: [],
    );
