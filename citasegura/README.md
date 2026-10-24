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

## Paso 3b. Probar sin WhatsApp (solo con la llave de Claude)
```
python probar_chat.py
```
Platicas con el bot en la terminal como si fueras el paciente. Tambien puedes simular botones
(`/boton Confirmo`), escribir como el consultorio (`/staff #reporte`) y correr los
recordatorios (`/recordatorios`). Usa su propia base `prueba_chat.db`.

## Paso 4. Exponerlo a internet (para que Meta te mande los mensajes)
Instala Cloudflare Tunnel (gratis) y ejecuta:
```
cloudflared tunnel --url http://localhost:8000
```
Copia la URL https que te da y en Meta -> WhatsApp -> Configuracion:
- Callback URL: `https://TU-URL/webhook`
- Verify token: el mismo de `WA_VERIFY_TOKEN`
- Suscribete al campo `messages` (y a `smb_message_echoes` si usas el numero del consultorio
  en coexistencia).

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

## Vender en automatico (modo ventas)
El mismo bot, en **otro numero**, vende CitaSegura a dentistas (`MODO=ventas`):
1. El dentista **te escribe primero** (volante con QR, anuncio de Facebook/Instagram con boton de
   WhatsApp, recomendacion). El bot nunca escribe en frio: WhatsApp lo prohibe y bloquea el numero.
2. El bot le pregunta cuantas faltas tiene y cuanto cobra, le calcula lo que pierde
   (`calcular_perdida`, las cuentas las hace el codigo), y le pide que **pruebe la demo ahi mismo**
   escribiendo como paciente: agenda, recibe como llegar y un recordatorio con botones.
3. Contesta dudas con lo del guion de venta y ofrece el piloto gratis de 30 dias.
4. Si quiere el piloto, una llamada o hablar con alguien, lo registra y **te avisa a ti**
   (`TELEFONO_HUMANO`): *"Interesado (piloto): Dr. Ramirez | Sonrisas Ramirez Neza | 5 faltas/semana
   | consulta $600 | ..."*. Desde ahi el bot deja de contestarle y sigues tu.

En modo ventas no hay recordatorios, reportes ni avisos de citas: las citas son de prueba.
`#interesados` (desde tu celular) lista los ultimos.

**Link para el QR o el anuncio** (cambia el numero por el de ventas):
`https://wa.me/5215500000000?text=Hola%2C%20quiero%20ver%20c%C3%B3mo%20funciona%20CitaSegura`

Como encenderlo en el servidor: ver "Bot de ventas" en DESPLIEGUE.md.

## Recordatorio de regreso (opcional)
Trae de vuelta a pacientes que no iban a regresar: *"Hola Ana, ya es momento de tu proxima
limpieza en Clinica Dental Sonrisa. ¿Te agendamos una cita?"*. Se activa con `REGRESO=1`.

- `REGRESO_REGLAS=limpieza:6,revision:12`: si el motivo de la cita trae esa palabra, se
  recuerda despues de esos meses (sin importar acentos ni mayusculas).
- Un solo mensaje por paciente y solo por su cita mas reciente. Nunca si ya tiene otra cita,
  si falto o si cancelo. Al activarlo no se le escribe a todo el historial viejo (solo a lo que
  vencio en los ultimos 30 dias).
- El boton **No por ahora** lo da de baja de estos recordatorios.
- El reporte del lunes cuenta los pacientes que regresaron gracias al recordatorio.

Necesita una plantilla aprobada en `WA_TEMPLATE_REGRESO` (es_MX, nombre `regreso_cita`).
Meta probablemente la clasifique como **marketing** (~USD 0.04 por mensaje); conviene
ofrecerlo en un plan mas alto:

> Hola {{1}}, ya es momento de tu proxima {{2}} en {{3}}. ¿Te agendamos una cita?
> [Agendar] [No por ahora]

"Agendar" lo atiende Claude como cualquier mensaje. Los pacientes deben saber que el
consultorio les puede escribir por WhatsApp (por ejemplo, avisarles al agendar).

## Notas de voz (opcional)
Muchos pacientes mandan audios. Con `VOZ=1` el bot los convierte a texto **en tu propio
servidor** con [faster-whisper](https://github.com/SYSTRAN/faster-whisper) (gratis, codigo
abierto) y contesta como si lo hubieran escrito. El audio no se manda a ningun servicio externo
ni se guarda.

```
pip install -r requirements-voz.txt
python -m app.voz mi_audio.ogg     # prueba: guarda una nota de voz de WhatsApp y transcribela
```

- La primera vez descarga el modelo (~250 MB con `small`) y tarda; despues es mas rapido.
- `VOZ_MODELO=small` entiende mejor; `base` usa menos memoria y es mas rapido (para un VPS de 1 GB).
  Con `small` conviene un servidor de 2 GB.
- Audios de mas de `VOZ_MAX_SEGUNDOS` (120) o que no se entienden: el bot pide que lo escriban.
- Si el audio trae nombre, dia u hora poco claros, el bot los confirma antes de agendar.

## Lista de espera
Si el dia que quiere el paciente esta lleno, el bot le ofrece otro dia y tambien anotarlo en
la **lista de espera** de ese dia. Cuando alguien **cancela o mueve** su cita, ese horario se
le ofrece al primero de la lista con dos botones: **Lo quiero** / **No, gracias**.
- Si acepta, queda agendado y le llegan las indicaciones para llegar.
- Si dice que no, o no contesta en `OFERTA_MINUTOS` (90), pasa al siguiente.
- El lugar no se aparta: si otra persona lo agenda primero, se le avisa y sigue en la lista.
- Al consultorio le llega "Nueva cita (lugar liberado, de la lista de espera)" y el reporte
  del lunes cuenta los lugares que se volvieron a llenar.

La oferta casi siempre llega dias despues de que el paciente escribio, fuera de la ventana de
24 h, asi que necesita una plantilla **Utilidad**, es_MX, nombre `lugar_liberado`, con
botones de respuesta rapida, puesta en `WA_TEMPLATE_LISTA_ESPERA`:

> Hola {{1}}, se libero un lugar en {{2}} el {{3}}. ¿Lo quieres? Si no nos contestas pronto se lo ofreceremos a otra persona.
> [Lo quiero] [No, gracias]

Sin plantilla solo se ofrece a quien escribio en las ultimas 24 h.

## Indicaciones para llegar
Copia `llegada.ejemplo.txt` a `llegada.txt` y escribe como llegar al consultorio con tus
palabras (estacionamiento, entrada, piso, numero de consultorio). Con la direccion, Google Maps
y Waze (`CLINICA_WAZE`) el bot arma un mensaje completo y lo manda solo:
- cuando el paciente **agenda o mueve** su cita por chat, y
- cuando toca **Confirmo** en el recordatorio (la primera vez).

En los dos casos el paciente acaba de escribir, asi que va como mensaje normal: **gratis** y
sin plantilla. Si preguntan "¿como llego?", el bot tambien lo sabe. El archivo se lee cada vez:
se puede corregir sin reiniciar. En el servidor va en `datos/llegada.txt`.

## Botones del recordatorio
**Confirmo** / **Cancelar** se resuelven directo en el codigo (sin gastar Claude) sobre la
proxima cita del paciente. **Reprogramar** lo atiende Claude con `reprogramar_cita`.

## Mismo numero del consultorio (coexistencia)
Meta permite que el numero siga en la app **WhatsApp Business** del consultorio y al mismo
tiempo este conectado al bot. Se conecta con el registro de Meta (*Embedded Signup*,
opcion "Connect a WhatsApp Business App"); revisa los requisitos actuales en la
documentacion de Meta antes del primer cliente.

Cuando la recepcion le contesta a un paciente desde su celular, **el bot se pausa solo en
esa conversacion** para no contestarle encima. Vuelve cuando el consultorio manda
`#bot <telefono>` o solo, tras `PAUSA_HUMANO_HORAS` (12 por omision) sin que la recepcion
escriba. Lo que escribio la recepcion queda en el historial para que el bot tenga contexto.

Para que funcione, en Meta -> WhatsApp -> Configuracion suscribete tambien al campo
**smb_message_echoes**. El formato se tomo de la documentacion publica: confirmalo con un
mensaje real de la recepcion y revisa en `docker compose logs bot` que el bot se pause.

## Comandos del consultorio
Desde el telefono `TELEFONO_HUMANO`, escribiendole al numero del bot:

| Comando | Que hace |
|---|---|
| `#reporte` | Resumen de la semana: citas, confirmadas, canceladas, faltas, % de inasistencia |
| `#hoy` | Citas de hoy con su numero |
| `#falta 12` | Marca que la cita 12 no llego (`#asistio 12` lo corrige) |
| `#bot 5215512345678` | Regresa al asistente una conversacion que estaba con una persona |

**Avisos automaticos:** al consultorio le llega un WhatsApp con cada cita nueva, cancelada o
movida, y cuando una conversacion necesita a una persona (dolor, urgencia, queja). Las
confirmaciones no se avisan para no llenar el chat; se ven con `#hoy`. Se apagan con
`AVISAR_CONSULTORIO=0`.

Para que los avisos lleguen siempre, crea una plantilla **Utilidad**, es_MX, nombre
`aviso_consultorio`, y ponla en `WA_TEMPLATE_AVISO`:

> Aviso de tu asistente de citas: {{1}}. Revisa tu agenda del dia con #hoy.

Sin plantilla se mandan como texto libre: no cuestan, pero solo llegan si el consultorio le
escribio al bot en las ultimas 24 h (por ejemplo, mandando `#hoy` cada manana).

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

## Produccion
Guia completa paso a paso en **[DESPLIEGUE.md](DESPLIEGUE.md)**: VPS con Ubuntu, Docker,
HTTPS automatico con Caddy, token permanente de Meta y respaldo diario cifrado.
```
docker compose up -d --build
```
En el contenedor `PRODUCCION=1`: si falta una llave en `.env` no arranca y dice cual.
