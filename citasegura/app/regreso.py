"""Recordatorio de regreso: "ya pasaron 6 meses de tu limpieza, ¿agendamos la siguiente?".

Trae de vuelta a pacientes que no iban a regresar. Reglas para no molestar ni arriesgar el numero:
- Solo si el consultorio lo activa (REGRESO=1) y hay plantilla aprobada (WA_TEMPLATE_REGRESO).
- Un solo mensaje por cita, y solo por la cita mas reciente del paciente.
- Nunca si el paciente ya tiene otra cita futura, falto a esa cita o pidio no recibirlos.
- Solo citas que "vencieron" en los ultimos 30 dias: al activarlo no se le escribe a todo
  el historial viejo.
- El mensaje trae "No por ahora", que lo da de baja de estos recordatorios.
Meta probablemente lo cobra como marketing (~USD 0.04): por eso es opcional.
"""
import logging
import os
import unicodedata
from datetime import datetime, timedelta

from . import db, whatsapp
from .clinica import NOMBRE

log = logging.getLogger("regreso")
FMT = "%Y-%m-%d %H:%M"
VENTANA_DIAS = 30


def _sin_acentos(texto: str) -> str:
    return "".join(c for c in unicodedata.normalize("NFD", texto.lower()) if unicodedata.category(c) != "Mn")


def reglas() -> dict[str, int]:
    """REGRESO_REGLAS="limpieza:6,revision:6,ortodoncia:1" -> {palabra del motivo: meses}."""
    resultado = {}
    for parte in os.getenv("REGRESO_REGLAS", "limpieza:6").split(","):
        if ":" in parte:
            palabra, meses = parte.split(":", 1)
            try:
                resultado[_sin_acentos(palabra.strip())] = int(meses)
            except ValueError:
                log.error("Regla de regreso invalida: %s", parte)
    return resultado


def _meses_para(motivo: str | None) -> tuple[str, int] | None:
    m = _sin_acentos(motivo or "")
    for palabra, meses in reglas().items():
        if palabra and palabra in m:
            return palabra, meses
    return None


def _sumar_meses(d: datetime, meses: int) -> datetime:
    mes = d.month - 1 + meses
    anio, mes = d.year + mes // 12, mes % 12 + 1
    for dia in (d.day, 28):  # 31 de agosto + 6 meses -> 28 de febrero
        try:
            return d.replace(year=anio, month=mes, day=dia)
        except ValueError:
            continue


def pendientes(ahora: datetime | None = None) -> list[dict]:
    """Citas a las que ya les toca el recordatorio de regreso."""
    ahora = ahora or datetime.now()
    with db.conn() as c:
        citas = c.execute(
            "SELECT * FROM citas WHERE estado IN ('agendada','confirmada') AND falto=0 AND regreso=0 "
            "AND inicio < ? AND telefono NOT IN (SELECT telefono FROM no_regreso) ORDER BY inicio DESC",
            (ahora.strftime(FMT),),
        ).fetchall()
        futuras = {
            f["telefono"]
            for f in c.execute(
                "SELECT telefono FROM citas WHERE estado IN ('agendada','confirmada') AND inicio >= ?",
                (ahora.strftime(FMT),),
            )
        }
    lista, vistos = [], set()
    for cita in citas:  # de la mas reciente a la mas vieja: solo cuenta la ultima de cada paciente
        tel = cita["telefono"]
        if tel in vistos or tel in futuras:
            continue
        vistos.add(tel)
        regla = _meses_para(cita["motivo"])
        if not regla:
            continue
        toca = _sumar_meses(datetime.strptime(cita["inicio"], FMT), regla[1])
        if toca <= ahora and toca >= ahora - timedelta(days=VENTANA_DIAS):
            lista.append({**dict(cita), "servicio": regla[0]})
    return lista


def revisar(ahora: datetime | None = None):
    """Una vez al dia: manda los recordatorios de regreso que tocan."""
    plantilla = os.getenv("WA_TEMPLATE_REGRESO")
    if os.getenv("REGRESO") != "1" or not plantilla:
        return
    ahora = ahora or datetime.now()
    for cita in pendientes(ahora):
        try:
            whatsapp.enviar_plantilla(cita["telefono"], plantilla, [cita["nombre"], cita["servicio"], NOMBRE])
        except Exception as e:
            log.error("No se pudo mandar el recordatorio de regreso de la cita %s: %s", cita["id"], e)
            continue
        with db.conn() as c:
            # esta cita y las anteriores del paciente ya no vuelven a disparar el recordatorio
            c.execute(
                "UPDATE citas SET regreso=1 WHERE telefono=? AND inicio <= ?", (cita["telefono"], cita["inicio"])
            )
            c.execute(
                "INSERT INTO regreso_envios (telefono, cita_id, enviado_en) VALUES (?,?,?)",
                (cita["telefono"], cita["id"], ahora.strftime(FMT)),
            )
        db.guardar_mensaje(
            cita["telefono"], "assistant",
            f"Te recordamos que ya toca tu {cita['servicio']}. ¿Quieres agendar tu siguiente cita?",
        )


def no_por_ahora(telefono) -> list[str]:
    with db.conn() as c:
        c.execute("INSERT OR IGNORE INTO no_regreso (telefono) VALUES (?)", (telefono,))
    return ["Entendido, ya no te mandaremos estos recordatorios. Cuando quieras una cita, aqui estoy 😊"]


def regresaron(desde: datetime, hasta: datetime) -> int:
    """Pacientes que agendaron en el periodo dentro de los 30 dias siguientes a su recordatorio."""
    with db.conn() as c:
        return c.execute(
            "SELECT COUNT(DISTINCT c.telefono) FROM citas c JOIN regreso_envios r ON r.telefono = c.telefono "
            "WHERE c.estado != 'reprogramada' AND c.creado >= r.enviado_en "
            "AND c.creado <= datetime(r.enviado_en, '+30 days') AND c.creado >= ? AND c.creado <= ?",
            (desde.strftime(FMT), hasta.strftime(FMT)),
        ).fetchone()[0]
