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

1. En el celular abre el repositorio en GitHub.
2. Entra a **Releases** (o al run mas reciente en **Actions → APK**).
3. Descarga el archivo `finanzas-<numero>.apk`.
4. Abrelo desde la barra de descargas. Android va a pedir permiso para
   **instalar apps de origen desconocido** — daselo al navegador.
5. Instalar. Listo.

Cada vez que cambie el codigo, GitHub compila un APK nuevo y publica otro
release. Instalas encima y se conservan tus datos.

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
