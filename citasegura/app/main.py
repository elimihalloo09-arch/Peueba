"""Servidor principal: recibe mensajes de WhatsApp y responde con Claude."""
import hashlib
import hmac
import logging
import os
from contextlib import asynccontextmanager
from datetime import datetime

from apscheduler.schedulers.background import BackgroundScheduler
from dotenv import load_dotenv

load_dotenv()

from fastapi import BackgroundTasks, FastAPI, HTTPException, Request  # noqa: E402
from fastapi.responses import PlainTextResponse  # noqa: E402

from . import agenda, avisos, clinica, db, ia, metricas, recordatorios, whatsapp  # noqa: E402

logging.basicConfig(level=logging.INFO)
log = logging.getLogger("citasegura")
scheduler = BackgroundScheduler()
OBLIGATORIAS = ("WA_TOKEN", "WA_PHONE_NUMBER_ID", "WA_VERIFY_TOKEN", "WA_APP_SECRET", "ANTHROPIC_API_KEY")


def revisar_configuracion():
    """En produccion (PRODUCCION=1) no arranca si falta una llave: mejor un error claro al
    iniciar que un bot que acepta mensajes falsos o que no puede contestar."""
    if os.getenv("PRODUCCION") != "1":
        return
    faltan = [v for v in OBLIGATORIAS if not os.getenv(v) or os.getenv(v, "").startswith("pega_aqui")]
    if faltan:
        raise RuntimeError("Faltan en .env: " + ", ".join(faltan))


@asynccontextmanager
async def ciclo(app):
    revisar_configuracion()
    db.init()
    scheduler.add_job(recordatorios.revisar, "interval", minutes=10)
    scheduler.add_job(metricas.enviar_reporte, "cron", day_of_week="mon", hour=9)  # lunes 9:00
    scheduler.start()
    yield
    scheduler.shutdown(wait=False)


app = FastAPI(title="CitaSegura", lifespan=ciclo)


@app.get("/salud")
def salud():
    return {"ok": True}


@app.get("/webhook")
def verificar(request: Request):
    """Meta llama aqui una sola vez para verificar tu webhook."""
    p = request.query_params
    if p.get("hub.mode") == "subscribe" and p.get("hub.verify_token") == os.getenv("WA_VERIFY_TOKEN"):
        return PlainTextResponse(p.get("hub.challenge"))
    raise HTTPException(403)


def _firma_valida(cuerpo: bytes, firma: str | None) -> bool:
    secreto = os.getenv("WA_APP_SECRET")
    if not secreto:
        return True  # en desarrollo; en produccion SIEMPRE configura WA_APP_SECRET
    esperada = "sha256=" + hmac.new(secreto.encode(), cuerpo, hashlib.sha256).hexdigest()
    return hmac.compare_digest(esperada, firma or "")


@app.post("/webhook")
async def recibir(request: Request, tareas: BackgroundTasks):
    cuerpo = await request.body()
    if not _firma_valida(cuerpo, request.headers.get("X-Hub-Signature-256")):
        raise HTTPException(401)
    data = await request.json()
    for entry in data.get("entry", []):
        for change in entry.get("changes", []):
            valor = change.get("value", {})
            for msg in valor.get("messages", []):
                tareas.add_task(procesar, msg)
            # Coexistencia: la recepcion contesto desde la app WhatsApp Business (campo smb_message_echoes)
            for eco in valor.get("message_echoes", []):
                tareas.add_task(eco_de_recepcion, eco)
    return {"ok": True}  # contestar rapido a Meta; el trabajo se hace en segundo plano


def _extraer_texto(msg: dict) -> str | None:
    tipo = msg.get("type")
    if tipo == "text":
        return msg["text"]["body"]
    if tipo == "button":  # boton de una plantilla (ej. "Confirmo")
        return msg["button"]["text"]
    if tipo == "interactive":
        i = msg["interactive"]
        return (i.get("button_reply") or i.get("list_reply") or {}).get("title")
    return None


AYUDA_STAFF = (
    "Comandos:\n"
    "#reporte – resumen de la semana\n"
    "#hoy – citas de hoy con su numero\n"
    "#falta 12 – la cita 12 no llego (#asistio 12 lo corrige)\n"
    "#bot 5215512345678 – regresar una conversacion al asistente"
)


def _comando_staff(texto: str) -> str:
    """Comandos que solo acepta el telefono del consultorio (TELEFONO_HUMANO)."""
    partes = texto.split()
    cmd = partes[0].lower()
    arg = partes[1] if len(partes) == 2 and partes[1].lstrip("#").isdigit() else None
    if cmd == "#reporte":
        return metricas.reporte()
    if cmd == "#hoy":
        return metricas.citas_de_hoy()
    if cmd in ("#falta", "#asistio") and arg:
        cita_id = int(arg.lstrip("#"))
        if metricas.marcar_falta(cita_id, falto=cmd == "#falta"):
            return f"Anotado: cita {cita_id} {'no llego' if cmd == '#falta' else 'si llego'}."
        return f"No encontre la cita {cita_id} (o todavia no es su hora)."
    if cmd == "#bot" and arg:
        if db.desactivar_humano(arg):
            return f"Listo, el asistente vuelve a atender a {arg}."
        return f"{arg} no estaba en atencion humana."
    return AYUDA_STAFF


def _boton_recordatorio(telefono: str, texto: str) -> list[str] | None:
    """Confirmo/Cancelar de la plantilla se resuelven sin Claude (mas rapido y sin costo).
    Devuelve los mensajes a mandar, o None si lo debe atender Claude (ej. Reprogramar)."""
    accion = texto.strip().lower()
    if accion not in ("confirmo", "cancelar"):
        return None
    cita = agenda.proxima_cita(telefono)
    if not cita:
        return None
    cuando = datetime.strptime(cita["inicio"], "%Y-%m-%d %H:%M").strftime("%d/%m a las %H:%M")
    if accion == "confirmo":
        agenda.cambiar_estado(telefono, cita["id"], "confirmada")
        mensajes = [f"¡Gracias por confirmar! Te esperamos el {cuando}."]
        if cita["estado"] == "agendada":  # la primera vez; si ya habia confirmado no se repite
            mensajes.append(clinica.como_llegar())
        return mensajes
    agenda.cambiar_estado(telefono, cita["id"], "cancelada")
    return [f"Listo, cancelamos tu cita del {cuando}. Si quieres agendar otra, aqui estoy."]


def eco_de_recepcion(eco: dict):
    """Una persona del consultorio le escribio al paciente desde su celular: el bot se hace a un
    lado en esa conversacion para no contestarle encima. Vuelve con #bot o solo tras
    PAUSA_HUMANO_HORAS sin que la recepcion escriba."""
    paciente = eco.get("to")
    if not paciente or not eco.get("id") or db.ya_procesado(eco["id"]):
        return
    db.activar_humano(paciente)
    texto = _extraer_texto(eco)
    if texto:  # queda en el historial para que el bot sepa que se dijo cuando regrese
        db.guardar_mensaje(paciente, "assistant", texto)


def procesar(msg: dict):
    if db.ya_procesado(msg["id"]):
        return
    telefono = msg["from"]
    texto = _extraer_texto(msg)
    if not texto:
        whatsapp.enviar_texto(telefono, "Por ahora solo puedo leer mensajes de texto. ¿Me lo escribes?")
        return
    staff = os.getenv("TELEFONO_HUMANO")
    if staff and telefono == staff and texto.strip().startswith("#"):
        whatsapp.enviar_texto(telefono, _comando_staff(texto))
        return
    db.guardar_mensaje(telefono, "user", texto)
    if db.en_modo_humano(telefono):
        return  # una persona de la clinica esta atendiendo esta conversacion
    respuestas = _boton_recordatorio(telefono, texto) if msg.get("type") == "button" else None
    if respuestas is None:
        antes = {c["id"] for c in agenda.citas_del_paciente(telefono)}
        try:
            respuestas = [ia.responder(telefono)]
        except Exception as e:
            log.exception("Error con Claude: %s", e)
            respuestas = ["Disculpa, tuve un problema. En un momento te atiende una persona de la clinica."]
        # cita nueva o movida en esta conversacion: mandar como llegar (gratis, el paciente acaba de escribir)
        if {c["id"] for c in agenda.citas_del_paciente(telefono)} - antes:
            respuestas.append(clinica.como_llegar())
    for respuesta in respuestas:
        db.guardar_mensaje(telefono, "assistant", respuesta)
        whatsapp.enviar_texto(telefono, respuesta)
    if db.en_modo_humano(telefono):
        avisos.avisar(f"Atender a {telefono}: {texto} | Para regresarlo al asistente: #bot {telefono}")
