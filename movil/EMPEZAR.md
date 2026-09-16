# Empezar en Ubuntu con Cursor

Tres pasos. El primero tarda unos minutos, los otros son inmediatos.

## 1. Instalar Flutter

Abre una terminal en esta carpeta y corre:

```bash
./instalar.sh
```

Baja el SDK a `~/flutter`, lo agrega al PATH y trae las dependencias del
proyecto. Te va a pedir tu contraseña una vez, para instalar los paquetes de
sistema.

Cuando termine, **cierra la terminal y abre otra** para que el PATH tome
efecto.

## 2. Verla corriendo

```bash
./correr.sh
```

Al arrancar imprime dos direcciones:

```
  En esta laptop:   http://localhost:8080
  En el iPhone:     http://192.168.x.x:8080
```

En la terminal donde quedó corriendo:

- **R** vuelve a cargar la app con tus cambios
- **q** la cierra

Deja esa terminal abierta mientras trabajas.

### Verla en el iPhone mientras la programas

La segunda dirección es la de tu laptop dentro de tu red. Con el **iPhone en
el mismo WiFi**, escríbela en Safari y ahí está la app corriendo, la misma
que tienes en la laptop.

Editas en Cursor, presionas **R** en la terminal, y el iPhone muestra el
cambio. Es la forma de trabajar la app viéndola en el teléfono de verdad,
del tamaño real y con el dedo, no en una ventana del navegador.

Si la IP no aparece, córrela a mano:

```bash
hostname -I
```

Y usa el primer número que salga.

**Esto funciona solo mientras la laptop está encendida con `./correr.sh`
corriendo y las dos en el mismo WiFi.** Para tenerla siempre, sin depender
de la laptop, usa la versión publicada y agrégala a la pantalla de inicio.

### Por qué no hay una app nativa de iPhone

Compilar para iOS exige Xcode, que solo existe en macOS, y firmar la app con
la membresía de desarrollador de Apple, 99 dólares al año. Desde Ubuntu no
hay manera, y no es una limitación de Flutter sino de Apple. La versión web
agregada a la pantalla de inicio queda con su icono y a pantalla completa, y
se usa igual.

## 3. Trabajarla en Cursor

```bash
cursor .
```

Instálale la extensión **Dart** y la de **Flutter** (busca "Dart-Code") y
tendrás autocompletado y errores en vivo.

Antes de dar por bueno cualquier cambio:

```bash
./probar.sh
```

Corre el analizador y las 19 pruebas. Si algo truena, ahí sale.

## Dónde está cada cosa

| Archivo | Qué hace |
|---|---|
| `lib/datos/semilla.dart` | **Tus cifras.** Acreedores, ingresos, gastos, pendientes. Es lo que vas a tocar más. |
| `lib/datos/migraciones.dart` | Corrige datos ya guardados en un teléfono. Al cambiar la semilla, agrega un paso aquí o el cambio no llega a donde ya está instalada. |
| `lib/modelo/simulador.dart` | El cálculo del plan de pagos. Dart puro, con pruebas. |
| `lib/ui/*.dart` | Las cinco pantallas. |
| `test/` | Las pruebas. |

Los montos se guardan en **centavos enteros**, nunca con decimales: `$184.50`
se escribe `18450`. Así no se pierden pesos por redondeo.

## Publicar los cambios

```bash
git add -A
git commit -m "lo que cambiaste"
git push
```

GitHub compila solo y verifica que las pruebas pasen.

## Ojo con los datos

Lo que capturas en la app vive en el navegador de cada aparato. **La laptop y
el teléfono llevan cuentas separadas.** Para pasar datos de uno a otro usa
Exportar e Importar, en el menú de los tres puntos.
