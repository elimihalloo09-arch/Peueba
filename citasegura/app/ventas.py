"""Modo ventas (MODO=ventas): el mismo bot, en otro numero, vende CitaSegura a dentistas.

Solo contesta a quien escribio primero (anuncio, volante con QR, recomendacion): nunca escribe
en frio, porque WhatsApp lo prohibe y bloquearia el numero. El dentista prueba la demo en su
propio WhatsApp, recibe el calculo de lo que pierde por faltas y, si le interesa, el bot avisa
al vendedor (TELEFONO_HUMANO) y se hace a un lado.
"""
import os
from datetime import datetime, timedelta

from . import agenda, db, whatsapp
from .clinica import INFO, NOMBRE

MENSUALIDAD_BASICO = 690


def activo() -> bool:
    return os.getenv("MODO") == "ventas"


def vendedor() -> str:
    return os.getenv("VENDEDOR_NOMBRE", "un asesor de CitaSegura")


def system_prompt() -> str:
    v = vendedor()
    return f"""Eres el asistente de ventas de CitaSegura y contestas WhatsApp de dentistas o personal
de consultorios que nos escribieron (por un anuncio, un volante o una recomendacion).
CitaSegura es un asistente de WhatsApp para consultorios dentales: agenda citas solo, manda
recordatorios con botones (Confirmo / Reprogramar / Cancelar), manda como llegar, avisa al
consultorio, rellena cancelaciones con una lista de espera, entiende notas de voz y cada lunes
manda al doctor un reporte con sus faltas. Funciona en el mismo WhatsApp del consultorio.

Tu objetivo: que prueben la demo aqui mismo y acepten un piloto gratis de 30 dias.
Hablas en espanol mexicano, amable, de "usted", breve (maximo 4 lineas), sin presionar.

Como llevar la platica (adaptate, no es un guion rigido):
1. Saluda y pregunta cuantos pacientes les faltan a la semana sin avisar y cuanto cobran
   una consulta en promedio.
2. Con esos dos numeros usa calcular_perdida y explicalo en 2 o 3 lineas. Nunca hagas las
   cuentas tu solo.
3. Invita a probar: "Escribame como si fuera un paciente que quiere una cita". Entonces
   actuas como la recepcionista virtual de {NOMBRE} (clinica de ejemplo) con las
   herramientas de agenda. Avisa el cambio de papel: "(Demo: ahora soy la recepcionista de
   una clinica de ejemplo)". Al terminar de agendar, el sistema le manda solo como llegar;
   luego usa demo_recordatorio para que vea el recordatorio con botones.
4. Precios: plan Basico $690 MXN al mes, plan Completo $990 MXN al mes (agrega lista de
   espera, notas de voz y recordatorio de regreso a los 6 meses). Instalacion $1,500 MXN una
   vez. Sin contrato anual. Piloto de 30 dias gratis, sin compromiso.
5. Si quiere el piloto, una llamada, una visita o hablar con una persona: pide su nombre,
   el nombre del consultorio y la zona (si aun no los tienes), usa registrar_interesado y
   dile que {v} le escribe hoy mismo. Despues de eso ya no sigas vendiendo.

Respuestas a dudas comunes:
- "Mi recepcionista ya hace eso": le quita lo repetitivo y contesta de noche y en fin de semana.
- "¿Y si contesta algo indebido?": nunca da diagnosticos ni consejos medicos; ante dolor,
  urgencia o enojo pasa la platica al consultorio y deja de contestar.
- "¿Cambio mi numero?": no, se conecta a su mismo numero de WhatsApp Business.
- "Esta caro": con 2 citas recuperadas al mes ya se pago; el piloto es gratis.
- "Ya tengo Doctoralia": Doctoralia trae pacientes nuevos; esto cuida a los que ya tiene.
- Datos de pacientes: servidor privado, respaldo diario cifrado, solo nombre, telefono y cita.

Reglas:
- No prometas un porcentaje exacto de reduccion de faltas ni funciones que no estan arriba.
- Si preguntan algo tecnico o de contrato que no sabes, ofrece que {v} le confirma.
- Solo hablas de CitaSegura y de la demo. Nunca pidas datos de sus pacientes.
- Si el usuario esta molesto o pide hablar con una persona, usa registrar_interesado con
  interes "hablar con una persona".

Datos de la clinica de ejemplo para la demo:
{INFO}"""


TOOLS = [
    {
        "name": "calcular_perdida",
        "description": "Calcula cuanto pierde el consultorio al mes por pacientes que faltan y cuanto recuperaria. "
        "Usala en cuanto sepas las faltas por semana y el precio de la consulta.",
        "input_schema": {
            "type": "object",
            "properties": {
                "faltas_semana": {"type": "number"},
                "precio_consulta": {"type": "number", "description": "MXN"},
            },
            "required": ["faltas_semana", "precio_consulta"],
        },
    },
    {
        "name": "demo_recordatorio",
        "description": "Manda al dentista, como ejemplo, el recordatorio con botones Confirmo/Reprogramar/Cancelar "
        "de la cita de prueba que acaba de agendar.",
        "input_schema": {"type": "object", "properties": {}},
    },
    {
        "name": "registrar_interesado",
        "description": "Registra a un dentista interesado y avisa al vendedor para que le escriba hoy. "
        "Despues de usarla el bot deja de contestar en esta platica.",
        "input_schema": {
            "type": "object",
            "properties": {
                "nombre": {"type": "string"},
                "consultorio": {"type": "string"},
                "zona": {"type": "string"},
                "faltas_semana": {"type": "number"},
                "precio_consulta": {"type": "number"},
                "whatsapp_business": {"type": "string", "description": "si / no / no sabe"},
                "interes": {"type": "string", "description": "piloto, llamada, visita, hablar con una persona"},
                "notas": {"type": "string", "description": "dudas u objeciones que menciono"},
            },
            "required": ["nombre", "interes"],
        },
    },
]
NOMBRES = {t["name"] for t in TOOLS}


def calcular_perdida(faltas_semana, precio_consulta) -> str:
    faltas = float(faltas_semana)
    precio = float(precio_consulta)
    if faltas <= 0 or precio <= 0:
        raise ValueError("faltas y precio deben ser mayores a cero")
    faltas_mes = round(faltas * 4.3)
    perdida = faltas_mes * precio
    recupera = perdida / 3  # supuesto conservador: recuperar 1 de cada 3 citas perdidas
    veces = recupera / MENSUALIDAD_BASICO
    return (
        f"Faltas al mes: {faltas_mes}. Pierde al mes: ${perdida:,.0f} MXN. "
        f"Si se recupera 1 de cada 3: ${recupera:,.0f} MXN al mes, "
        f"{veces:.1f} veces la mensualidad del plan Basico (${MENSUALIDAD_BASICO})."
    )


def demo_recordatorio(telefono) -> str:
    cita = agenda.proxima_cita(telefono)
    if not cita:
        return "Todavia no tiene cita de prueba: primero que agende una como paciente."
    d = datetime.strptime(cita["inicio"], "%Y-%m-%d %H:%M")
    texto = (
        "Asi le llega al paciente 2 dias antes:\n\n"
        f"Hola {cita.get('nombre') or ''}, te recordamos tu cita en {NOMBRE} el {d:%d/%m a las %H:%M}. ¿Nos confirmas?"
    )
    whatsapp.enviar_botones(telefono, texto, ["Confirmo", "Reprogramar", "Cancelar"])
    db.guardar_mensaje(telefono, "assistant", texto)
    return "Recordatorio de prueba enviado con botones. Invitalo a tocar uno para ver que pasa."


def registrar_interesado(telefono, args) -> str:
    def num(v):
        try:
            return float(v) if v not in (None, "") else None
        except (TypeError, ValueError):
            return None

    with db.conn() as c:
        c.execute(
            "INSERT INTO interesados (telefono, nombre, consultorio, zona, faltas_semana, precio_consulta, "
            "whatsapp_business, interes, notas) VALUES (?,?,?,?,?,?,?,?,?)",
            (
                telefono, args.get("nombre"), args.get("consultorio"), args.get("zona"),
                num(args.get("faltas_semana")), num(args.get("precio_consulta")),
                args.get("whatsapp_business"), args.get("interes"), args.get("notas"),
            ),
        )
    db.activar_humano(telefono)  # desde aqui sigue el vendedor; main.py le manda el aviso
    return f"Registrado. Dile que {vendedor()} le escribe hoy mismo y despidete."


def ejecutar(nombre, args, telefono) -> str:
    if nombre == "calcular_perdida":
        return calcular_perdida(args["faltas_semana"], args["precio_consulta"])
    if nombre == "demo_recordatorio":
        return demo_recordatorio(telefono)
    if nombre == "registrar_interesado":
        return registrar_interesado(telefono, args)
    raise KeyError(nombre)


def aviso_interesado(telefono) -> str | None:
    """Texto para el vendedor si este telefono se acaba de registrar como interesado."""
    hace_poco = (datetime.now() - timedelta(minutes=10)).strftime("%Y-%m-%d %H:%M:%S")
    with db.conn() as c:
        f = c.execute(
            "SELECT * FROM interesados WHERE telefono=? AND creado >= ? ORDER BY id DESC", (telefono, hace_poco)
        ).fetchone()
    if not f:
        return None
    partes = [f"Interesado ({f['interes']}): {f['nombre']}"]
    if f["consultorio"] or f["zona"]:
        partes.append(" ".join(x for x in (f["consultorio"], f["zona"]) if x))
    if f["faltas_semana"]:
        partes.append(f"{f['faltas_semana']:g} faltas/semana")
    if f["precio_consulta"]:
        partes.append(f"consulta ${f['precio_consulta']:,.0f}")
    if f["whatsapp_business"]:
        partes.append(f"WhatsApp Business: {f['whatsapp_business']}")
    if f["notas"]:
        partes.append(f"notas: {f['notas']}")
    return " | ".join(partes) + f" | Tel {telefono}. El bot ya no le contesta; para regresarlo: #bot {telefono}"


def lista_interesados(limite=10) -> str:
    with db.conn() as c:
        filas = c.execute("SELECT * FROM interesados ORDER BY id DESC LIMIT ?", (limite,)).fetchall()
    if not filas:
        return "Todavia no hay interesados."
    lineas = ["*Ultimos interesados*"]
    for f in filas:
        lineas.append(f"{f['creado'][:16]} {f['nombre']} ({f['interes']}) Tel {f['telefono']}")
    return "\n".join(lineas)
