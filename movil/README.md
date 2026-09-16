# Mis Finanzas — app de Android

App que corre **sola en el telefono**. No necesita servidor, ni internet, ni
computadora. Todo se guarda en el propio telefono.

## Que trae

- **Resumen** — lo que entra, lo que sale, lo que queda, y cuanta deuda falta.
- **Acreedores** — los 12 con saldo, tasa, minimo, quita negociada y etapa
  (pendiente / negociando / acordado / pagando / liquidado).
- **Plan** — simulador de pagos: avalancha o bola de nieve, con el apoyo
  mensual de Mario. Dice el mes exacto en que quedas libre y cuanto interes
  pagas de mas segun el metodo.
- **Pendientes** — lo que falta hacer, con lo urgente arriba.
- **Correos** — los textos ya escritos para HeyBanco y Klar, listos para
  copiar y pegar.

Los montos se guardan en **centavos enteros**, nunca en decimales, para que
no se pierdan pesos por redondeo.

## Instalar en el telefono (sin computadora)

### iPhone

iOS no instala APKs, y la version nativa exige una Mac y la membresia de
desarrollador de Apple. La salida es la version web, que en iOS queda con su
propio icono y a pantalla completa:

1. Abre la app en **Safari**.
2. Boton **Compartir** (el cuadro con la flecha hacia arriba).
3. **Agregar a pantalla de inicio**.

Queda como **Mis Finanzas**. Los datos se guardan en el almacenamiento de
Safari, asi que si limpias los datos del navegador se borran: usa
**Exportar** del menu de los tres puntos y guardate ese texto.

### Android

1. En el celular abre el repositorio en GitHub.
2. Entra a **Releases** (o al run mas reciente en **Actions → APK**).
3. Descarga el archivo `finanzas-<numero>.apk`.
4. Abrelo desde la barra de descargas. Android va a pedir permiso para
   **instalar apps de origen desconocido** — daselo al navegador.
5. Instalar. Listo.

Cada vez que cambie el codigo, GitHub compila un APK nuevo y publica otro
release. Instalas encima y se conservan tus datos.

El APK y el paquete web llevan dentro los datos financieros reales, asi que
el workflow **solo los publica si el repositorio es privado**. Mientras sea
publico compila y verifica, pero no sube nada.

## Compilar sin computadora

En el iPhone no se puede compilar: Apple no permite compiladores en iOS. Lo
que si se puede es compilar en la nube y manejarlo desde el telefono. Tres
formas, de la mas comoda a la mas laboriosa:

**1. Pedirselo a Claude Code.** Compila, prueba y publica la version nueva.
Es lo que conviene para cualquier cambio de fondo.

**2. Editar en github.com y dejar que Actions compile.** Abre el archivo en
el navegador del telefono, el icono del lapiz, cambia, **Commit**. El
workflow `.github/workflows/apk.yml` analiza, prueba y compila solo. Sirve
para ajustes chicos: un texto, una cifra, un color.

**3. Un Codespace.** Es una maquina Linux completa dentro del navegador, con
terminal de verdad. En el repositorio: **Code → Codespaces → Create**. El
`.devcontainer/` de este repo instala Flutter solo la primera vez (tarda unos
minutos). Despues:

```bash
cd movil
flutter run -d web-server --web-port 8080 --web-hostname 0.0.0.0
```

GitHub reenvia el puerto 8080 y te da una liga para abrirla en el mismo
telefono. Escribir codigo en el teclado del celular es incomodo, pero
funciona y no cuesta nada dentro de las horas gratis de la cuenta.

**La version nativa de iOS no sale por ninguna de las tres.** Necesita una
Mac o un runner de macOS **y** la membresia de desarrollador de Apple, 99
dolares al ano, porque sin firmar la app el iPhone no la instala. La version
web instalada en la pantalla de inicio evita ese costo.

## Correr en desarrollo

```bash
flutter pub get
flutter test      # 8 pruebas del simulador
flutter analyze
flutter run
```

## Como esta armado

| Archivo | Que hace |
|---|---|
| `lib/modelo/simulador.dart` | La logica de pagos. Dart puro, sin Flutter, con pruebas. |
| `lib/modelo/datos.dart` | Acreedor, Pendiente, Movimiento, Estado y su serializacion. |
| `lib/datos/semilla.dart` | Los datos reales de arranque. |
| `lib/datos/repositorio.dart` | Guardar y leer de `SharedPreferences`. |
| `lib/ui/*.dart` | Las cinco pantallas. |

El simulador es el mismo algoritmo que el del backend en Rust
(`app/backend/src/dominio/simulador.rs`), portado a Dart y con las mismas
ocho pruebas, para que ambos den el mismo resultado.
