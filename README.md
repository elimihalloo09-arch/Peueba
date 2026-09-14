# Proyector de gastos y ahorro de traslados

Herramienta de una sola página para proyectar cuánto cuesta ir a trabajar y cuánto
se ahorra con cada día de trabajo desde casa, comparando dos destinos.

Viene cargada con las rutas reales desde Calle 27 #136, Nezahualcóyotl (CP 57210)
a las dos sedes del SAT — **CPN / El Reloj** (Coyoacán) y **BanCen / Av. Hidalgo 77**
(Col. Guerrero) — con tarifas oficiales: Mexibús $10, Metro $5, Tren Ligero $3,
Metrobús $6.

## Qué hace

- Desglosa el trayecto **tramo por tramo** (Mexibús → Metro → Tren Ligero) y suma ida y vuelta.
- Compara la ruta económica contra el **Uber** (~$240 ida + ~$259 regreso) con los mismos días presenciales.
- Calcula el costo de **un día presencial** en cada destino: transporte público,
  vehículo propio o moto (combustible, peajes, parqueadero, mantenimiento por km),
  taxi/app, o bici/a pie — más la comida fuera.
- Proyecta el **gasto mensual y anual** según los días que trabajes desde casa,
  descontando el gasto extra de estar en casa (luz, internet, almuerzo).
- Muestra el **ahorro mensual y anual** y las **horas de trayecto recuperadas**.
- **Tu dinero**: con tu sueldo neto mensual calcula cuánto ganas por hora, qué porcentaje
  del sueldo se va en el traslado (y cuánto se iría en Uber), lo que te queda libre y cuánto
  vale el tiempo que pasas en el camino.
- **Mis gastos del mes**: lista editable de gastos (renta, despensa, comida fuera, servicios,
  apps y suscripciones, salud, gustos…) con categoría y frecuencia — día, semana, quincena,
  mes o año — convertida a su equivalente mensual. El transporte de la ruta elegida entra
  solo, calculado. Muestra el total, qué porcentaje del sueldo es, cuánto te sobra y una
  gráfica de barras por categoría.
- **Bola de nieve**: toma el ahorro mensual contra el Uber como aporte y proyecta el
  acumulado a 6, 12, 24 y 60 meses, con un rendimiento anual editable (supuesto, no
  promesa) y la fecha en que alcanzas tu meta. El aporte puede ser el ahorro contra el Uber,
  lo que te sobra del sueldo o un monto fijo.
- Gráfica comparativa del gasto mensual para cada combinación de días desde casa,
  con recorrido punto a punto, y tabla de desglose (presencial completo, escenario
  actual, ahorro, 100% desde casa).

## Uso

`index.html` es autónomo: ábrelo en el navegador, cambia los nombres de los
destinos, la moneda y los valores del ejemplo por los tuyos. Los datos se guardan
en el navegador; publicado como Artifact, también se guardan en el almacenamiento
del propio artifact para recuperarlos desde otro dispositivo.

Fórmula base:

```
gasto del mes = (días laborales − días desde casa) × costo del día presencial
              + días desde casa × gasto extra en casa
```
