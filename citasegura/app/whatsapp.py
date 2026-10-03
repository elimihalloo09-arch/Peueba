"""Envio de mensajes por WhatsApp Cloud API de Meta."""
import os

import httpx

GRAPH = os.getenv("GRAPH_VERSION", "v21.0")


def _url():
    return f"https://graph.facebook.com/{GRAPH}/{os.environ['WA_PHONE_NUMBER_ID']}/messages"


def _headers():
    return {"Authorization": f"Bearer {os.environ['WA_TOKEN']}"}


def enviar_texto(telefono: str, texto: str):
    payload = {"messaging_product": "whatsapp", "to": telefono, "type": "text", "text": {"body": texto}}
    r = httpx.post(_url(), headers=_headers(), json=payload, timeout=20)
    r.raise_for_status()


def _limpiar(texto: str) -> str:
    """Meta rechaza variables de plantilla con saltos de linea, tabuladores o muchos espacios
    seguidos (ej. una direccion copiada en varias lineas)."""
    return " ".join(str(texto).split())


def enviar_plantilla(telefono: str, plantilla: str, parametros: list[str], idioma="es_MX"):
    """Las plantillas (ej. recordatorios) deben estar aprobadas por Meta antes de usarse."""
    payload = {
        "messaging_product": "whatsapp",
        "to": telefono,
        "type": "template",
        "template": {
            "name": plantilla,
            "language": {"code": idioma},
            "components": [
                {"type": "body", "parameters": [{"type": "text", "text": _limpiar(p)} for p in parametros]}
            ],
        },
    }
    r = httpx.post(_url(), headers=_headers(), json=payload, timeout=20)
    r.raise_for_status()
