# CitaSegura – Bot de citas por WhatsApp para consultorios dentales

## Que hace
- Contesta WhatsApp con Claude (precios, horarios, ubicacion).
- Agenda, lista y cancela citas (SQLite).
- Manda recordatorios 48 h y 2 h antes (plantilla aprobada por Meta).
- Pasa la conversacion a una persona si hay dolor, urgencia o queja.

## Estructura
```
app/
  main.py          servidor FastAPI + webhook de WhatsApp
  ia.py            Claude con herramientas de agenda
  agenda.py        horarios libres, agendar, cancelar
  recordatorios.py tarea cada 10 min para recordatorios
  whatsapp.py      envio de texto y plantillas (Cloud API de Meta)
  db.py            base de datos SQLite
  clinica.py       datos de la clinica y prompt del bot  <-- edita aqui
```

## Paso 1. Preparar tu compu
1. Instala Python 3.12, Git y Cursor.
2. En la carpeta del proyecto:
```
python -m venv .venv
# Windows: .venv\Scripts\activate    Mac/Linux: source .venv/bin/activate
pip install -r requirements.txt
cp .env.example .env     # y llena tus llaves
```

## Paso 2. Llaves
- **Claude:** crea tu API key en console.anthropic.com -> `ANTHROPIC_API_KEY`.
- **WhatsApp:** en developers.facebook.com crea una App tipo "Business",
  agrega el producto WhatsApp. Meta te da un **numero de prueba** gratis
  (puedes agregar hasta 5 numeros destino para probar).
  Copia `Phone number ID` y el token temporal a `.env`.
  Revisa en el panel de Meta la version actual de la Graph API y ajusta `GRAPH_VERSION`.

## Paso 3. Correr el servidor
```
uvicorn app.main:app --reload --port 8000
```
Prueba: abre http://localhost:8000/salud

## Paso 4. Exponerlo a internet (para que Meta te mande los mensajes)
Instala Cloudflare Tunnel (gratis) y ejecuta:
```
cloudflared tunnel --url http://localhost:8000
```
Copia la URL https que te da y en Meta -> WhatsApp -> Configuracion:
- Callback URL: `https://TU-URL/webhook`
- Verify token: el mismo de `WA_VERIFY_TOKEN`
- Suscribete al campo `messages`.

Escribele al numero de prueba desde tu celular. Debe contestar Claude.

## Paso 5. Plantilla de recordatorio (para clientes reales)
En WhatsApp Manager crea una plantilla categoria **Utilidad**, idioma es_MX,
nombre `recordatorio_cita`, con 3 variables y botones de respuesta rapida:

> Hola {{1}}, te recordamos tu cita en {{2}} el {{3}}. ¿Nos confirmas?
> [Confirmo] [Reprogramar] [Cancelar]

Y una segunda plantilla **Utilidad**, es_MX, nombre `recordatorio_2h`, con 5 variables
(se manda 2 h antes):

> Hola {{1}}, te recordamos que tu cita en {{2}} es hoy a las {{3}}.
> Direccion: {{4}}
> Como llegar: {{5}}
> Si ya no puedes venir, respondenos este mensaje para darle tu lugar a otro paciente. ¡Te esperamos!

Reglas de Meta que evitan el rechazo: ninguna variable al inicio ni al final del texto, y
las variables van en orden ({{1}}, {{2}}…). Al mandar, el codigo quita saltos de linea de
las variables porque Meta tampoco los acepta ahi.

Cuando Meta la apruebe pon `WA_TEMPLATE_RECORDATORIO_2H=recordatorio_2h` en `.env`.
Mientras tanto se usa la primera. Llena `CLINICA_DIRECCION` y `CLINICA_MAPS` (en Google
Maps: busca el consultorio -> Compartir -> Copiar vinculo).

## Botones del recordatorio
**Confirmo** / **Cancelar** se resuelven directo en el codigo (sin gastar Claude) sobre la
proxima cita del paciente. **Reprogramar** lo atiende Claude con `reprogramar_cita`.

## Comandos del consultorio
Desde el telefono `TELEFONO_HUMANO`, escribiendole al numero del bot:

| Comando | Que hace |
|---|---|
| `#reporte` | Resumen de la semana: citas, confirmadas, canceladas, faltas, % de inasistencia |
| `#hoy` | Citas de hoy con su numero |
| `#falta 12` | Marca que la cita 12 no llego (`#asistio 12` lo corrige) |
| `#bot 5215512345678` | Regresa al asistente una conversacion que estaba con una persona |

Cada **lunes a las 9:00** el bot manda el reporte solo. Ese mensaje (y el aviso de "Atender a…")
es texto libre: WhatsApp solo lo entrega si `TELEFONO_HUMANO` le escribio al bot en las ultimas
24 h. Si no llego, `#reporte` lo trae en cualquier momento.

**Las faltas las marca recepcion** con `#falta`: el bot no puede saber si alguien llego. Sin
eso, el reporte cuenta a todos como asistencias. Lo practico: al cerrar el dia, `#hoy` y
marcar quien no vino.

## Pruebas
```
pip install -r requirements-dev.txt
pytest
```
No usan Claude ni Meta: cada prueba crea su propia base de datos temporal.

## Produccion (cuando tengas clientes que pagan)
- No dejes el bot en tu compu de casa: si se va la luz o el internet, no salen los recordatorios.
- Usa un VPS barato con Ubuntu, configura `TZ=America/Mexico_City` y corre con Docker o systemd.
- Respalda `citasegura.db` diario. Con 5+ clientes, migra a PostgreSQL.
- Configura `WA_APP_SECRET` para validar que los mensajes vienen de Meta.
