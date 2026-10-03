"""Revisa cada 10 minutos las citas y manda recordatorios de 48 h y 2 h."""
import logging
import os
from datetime import datetime, timedelta

from . import db, whatsapp
from .clinica import NOMBRE

log = logging.getLogger("recordatorios")
PLANTILLA = os.getenv("WA_TEMPLATE_RECORDATORIO", "recordatorio_cita")


def revisar():
    ahora = datetime.now()
    with db.conn() as c:
        citas = c.execute(
            "SELECT * FROM citas WHERE estado IN ('agendada','confirmada') AND inicio >= ?",
            (ahora.strftime("%Y-%m-%d %H:%M"),),
        ).fetchall()

    for cita in citas:
        inicio = datetime.strptime(cita["inicio"], "%Y-%m-%d %H:%M")
        faltan = inicio - ahora
        fecha_txt = inicio.strftime("%d/%m a las %H:%M")
        try:
            if faltan <= timedelta(hours=2) and not cita["recordatorio_2h"]:
                _enviar(cita, fecha_txt)
                # si nunca salio el de 48 h (cita agendada de ultimo momento) ya no se manda:
                # un solo mensaje basta y cada plantilla cuesta
                _marcar(cita["id"], "recordatorio_2h", "recordatorio_48h")
            elif timedelta(hours=2) < faltan <= timedelta(hours=48) and not cita["recordatorio_48h"]:
                # Plantilla con botones Confirmo / Reprogramar / Cancelar (se configuran en Meta)
                _enviar(cita, fecha_txt)
                _marcar(cita["id"], "recordatorio_48h")
        except Exception as e:  # un error con un paciente no debe detener a los demas
            log.error("Fallo recordatorio cita %s: %s", cita["id"], e)


def _enviar(cita, fecha_txt):
    whatsapp.enviar_plantilla(cita["telefono"], PLANTILLA, [cita["nombre"], NOMBRE, fecha_txt])
    # queda en el historial para que Claude entienda si el paciente contesta "Reprogramar"
    db.guardar_mensaje(cita["telefono"], "assistant", f"Recordatorio enviado: tu cita es el {fecha_txt}.")


def _marcar(cita_id, *campos):
    with db.conn() as c:
        for campo in campos:  # nombres fijos del codigo, nunca vienen del usuario
            c.execute(f"UPDATE citas SET {campo}=1 WHERE id=?", (cita_id,))
