"""Servidor principal: recibe mensajes de WhatsApp y responde con Claude."""
import hashlib
import hmac
import logging
import os

from apscheduler.schedulers.background import BackgroundScheduler
from dotenv import load_dotenv

load_dotenv()

from fastapi import BackgroundTasks, FastAPI, HTTPException, Request  # noqa: E402
from fastapi.responses import PlainTextResponse  # noqa: E402

from . import db, ia, recordatorios, whatsapp  # noqa: E402

logging.basicConfig(level=logging.INFO)
log = logging.getLogger("citasegura")
app = FastAPI(title="CitaSegura")
scheduler = BackgroundScheduler()


@app.on_event("startup")
def inicio():
    db.init()
    scheduler.add_job(recordatorios.revisar, "interval", minutes=10)
    scheduler.start()


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
            for msg in change.get("value", {}).get("messages", []):
                tareas.add_task(procesar, msg)
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


def procesar(msg: dict):
    if db.ya_procesado(msg["id"]):
        return
    telefono = msg["from"]
    texto = _extraer_texto(msg)
    if not texto:
        whatsapp.enviar_texto(telefono, "Por ahora solo puedo leer mensajes de texto. ¿Me lo escribes?")
        return
    db.guardar_mensaje(telefono, "user", texto)
    if db.en_modo_humano(telefono):
        return  # una persona de la clinica esta atendiendo esta conversacion
    try:
        respuesta = ia.responder(telefono)
    except Exception as e:
        log.exception("Error con Claude: %s", e)
        respuesta = "Disculpa, tuve un problema. En un momento te atiende una persona de la clinica."
    db.guardar_mensaje(telefono, "assistant", respuesta)
    whatsapp.enviar_texto(telefono, respuesta)
    if db.en_modo_humano(telefono) and os.getenv("TELEFONO_HUMANO"):
        whatsapp.enviar_texto(os.environ["TELEFONO_HUMANO"], f"Atender a {telefono}: {texto}")
