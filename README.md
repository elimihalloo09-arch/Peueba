# Proyector de gastos y ahorro de traslados

Herramienta de una sola página para proyectar cuánto cuesta ir a trabajar y cuánto
se ahorra con cada día de trabajo desde casa, comparando dos destinos (por defecto
**CPN** y **Bancen**).

## Qué hace

- Calcula el costo de **un día presencial** en cada destino: transporte público,
  vehículo propio o moto (combustible, peajes, parqueadero, mantenimiento por km),
  taxi/app, o bici/a pie — más la comida fuera.
- Proyecta el **gasto mensual y anual** según los días que trabajes desde casa,
  descontando el gasto extra de estar en casa (luz, internet, almuerzo).
- Muestra el **ahorro mensual y anual** y las **horas de trayecto recuperadas**.
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
