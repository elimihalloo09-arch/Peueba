# Finanzas

Aplicación para gestionar la salida de deudas: acreedores con su etapa de
negociación, simulador de liquidación, pendientes con fecha y préstamos de
personas.

**Rust + Axum + SQLx** en el backend, **Angular 18** en el frontend,
**PostgreSQL** de base. Todo el dinero se guarda en **centavos** (`bigint`):
nunca en punto flotante.

## Correr

```bash
# 1. Base de datos
docker compose up -d

# 2. Backend (aplica migraciones y siembra los datos al arrancar)
cp .env.example backend/.env
cd backend && cargo run

# 3. Frontend, en otra terminal
cd frontend && npm install && npm start
```

La app queda en <http://localhost:4200> y la API en <http://localhost:3000>.

> El navegador solo puede llamar al backend desde el origen que diga
> `ORIGEN_PERMITIDO`. Si abres la app en `127.0.0.1` en vez de `localhost`,
> CORS la bloquea: son orígenes distintos.

## Estructura

```
backend/
  migrations/           Esquema y datos iniciales. Se aplican solos al arrancar.
  src/dominio/          Simulador de liquidación, con sus pruebas.
  src/rutas/            Un archivo por recurso de la API.
  src/modelos.rs        Structs que viajan a la base y al navegador.
frontend/
  src/app/nucleo/       Cliente de la API, modelos y el pipe de dinero.
  src/app/paginas/      Una página por sección.
```

## La API

| Método | Ruta | Qué hace |
|---|---|---|
| `GET` | `/api/resumen` | Ingreso, gasto, deuda y capacidad del mes |
| `GET` | `/api/plan` | Simula la liquidación completa |
| `GET POST` | `/api/acreedores` | Lista y alta |
| `PUT DELETE` | `/api/acreedores/:id` | Cambia etapa, montos, documentos y notas |
| `GET POST` | `/api/pendientes` | Lista y alta |
| `PUT DELETE` | `/api/pendientes/:id` | Marca hecho o borra |
| `GET POST` | `/api/pagos` | Registro de pagos |
| `GET POST` | `/api/movimientos` | Préstamos de personas, por persona |

Parámetros de `/api/plan`, todos en centavos:

```
capacidad=1335200      todo lo que se destina al mes a deuda, mínimos incluidos
metodo=avalancha       o bola_de_nieve
apoyo_mensual=1000000  dinero prestado que entra los primeros meses
meses_apoyo=6
sin_quitas=false       true compara contra los saldos completos
```

## El simulador

Vive en `backend/src/dominio/simulador.rs` y es donde está la lógica que
importa:

- Cobra el interés del mes sobre lo que sigue vivo.
- Paga los mínimos comprometidos de cada deuda.
- Concentra todo lo que sobra en **una sola** deuda: la de tasa más alta
  (avalancha) o la de saldo más chico (bola de nieve).
- Lo marcado como urgente se paga antes que cualquier criterio financiero.
- El pago mensual que se libera al saldar una deuda **rueda** a la siguiente.
- Si el abono no alcanza ni para los intereses, devuelve `inalcanzable` y la
  lista de las que se quedan atoradas, en vez de correr para siempre.

Ocho pruebas lo cubren:

```bash
cd backend && cargo test
```

## Seguridad

Pensada para correr en tu máquina. Antes de exponerla:

1. Ponle valor a `APP_API_KEY` en `backend/.env` y el mismo en
   `frontend/src/app/nucleo/api.ts`. Sin llave, la API queda abierta y el
   backend te lo avisa en el log al arrancar.
2. Cambia la contraseña de Postgres del `docker-compose.yml`.
3. Ponlo detrás de HTTPS.

Si más adelante la va a usar alguien más, el siguiente paso es cambiar la
llave por usuarios con sesión.

## Qué sigue

- Importar el estado de cuenta del banco en PDF o CSV y categorizar solo.
- Registrar los pagos desde la pantalla de acreedores, no solo por API.
- Historial mensual: guardar el cierre de cada mes para comparar plan contra real.
- Recordatorios de los pendientes que vencen.
