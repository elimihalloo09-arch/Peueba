"""Avisos al consultorio (TELEFONO_HUMANO): cita nueva, cancelada, movida o que necesita a una persona."""
import logging
import os
from datetime import datetime

from . import whatsapp

log = logging.getLogger("avisos")
DIAS = ["lun", "mar", "mie", "jue", "vie", "sab", "dom"]


def cuando(inicio: str) -> str:
    """'2026-10-05 10:00' -> 'lun 05/10 10:00'."""
    d = datetime.strptime(inicio, "%Y-%m-%d %H:%M")
    return f"{DIAS[d.weekday()]} {d.strftime('%d/%m %H:%M')}"


def avisar(texto: str):
    """Manda el aviso sin detener al paciente si falla.

    Texto libre solo llega si el consultorio le escribio al bot en las ultimas 24 h.
    Con WA_TEMPLATE_AVISO (plantilla de Utilidad con una variable) llega siempre,
    pero cada aviso cuesta como mensaje de utilidad."""
    destino = os.getenv("TELEFONO_HUMANO")
    if not destino or os.getenv("AVISAR_CONSULTORIO", "1") == "0":
        return
    try:
        plantilla = os.getenv("WA_TEMPLATE_AVISO")
        if plantilla:
            whatsapp.enviar_plantilla(destino, plantilla, [texto])
        else:
            whatsapp.enviar_texto(destino, texto)
    except Exception as e:
        log.error("No se pudo avisar al consultorio: %s", e)
