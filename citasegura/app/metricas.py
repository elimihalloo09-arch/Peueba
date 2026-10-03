"""Metricas para el doctor: lo que justifica la mensualidad (sobre todo, menos faltas)."""
import logging
import os
from datetime import datetime, timedelta

from . import db, whatsapp
from .clinica import NOMBRE

log = logging.getLogger("metricas")
FMT = "%Y-%m-%d %H:%M"
# reprogramada = la cita se movio a otra hora; la nueva es la que cuenta
VALIDA = "estado != 'reprogramada'"
ACTIVA = "estado IN ('agendada','confirmada')"


def calcular(desde: datetime, hasta: datetime) -> dict:
    """Cuenta las citas con inicio entre desde (incluido) y hasta (excluido)."""
    rango = (desde.strftime(FMT), hasta.strftime(FMT))
    with db.conn() as c:
        f = c.execute(
            f"""SELECT COUNT(*) AS total,
                   COALESCE(SUM(estado = 'cancelada'), 0) AS canceladas,
                   COALESCE(SUM(estado = 'confirmada'), 0) AS confirmadas,
                   COALESCE(SUM({ACTIVA} AND falto = 1), 0) AS faltas
            FROM citas WHERE {VALIDA} AND inicio >= ? AND inicio < ?""",
            rango,
        ).fetchone()
        nuevas = c.execute(
            f"SELECT COUNT(*) FROM citas WHERE {VALIDA} AND creado >= ? AND creado <= ?", rango  # incluye este minuto
        ).fetchone()[0]
    m = dict(f)
    m["nuevas"] = nuevas
    with db.conn() as c:  # lugares cancelados que la lista de espera volvio a llenar
        m["rellenados"] = c.execute(
            "SELECT COUNT(*) FROM espera WHERE estado='tomada' AND tomada_en >= ? AND tomada_en <= ?", rango
        ).fetchone()[0]
    m["atendibles"] = m["total"] - m["canceladas"]  # las que el consultorio esperaba
    m["asistencias"] = m["atendibles"] - m["faltas"]
    m["tasa_faltas"] = m["faltas"] / m["atendibles"] if m["atendibles"] else 0.0
    m["tasa_confirmacion"] = m["confirmadas"] / m["atendibles"] if m["atendibles"] else 0.0
    return m


def _pct(x: float) -> str:
    return f"{round(x * 100)}%"


def reporte(ahora: datetime | None = None) -> str:
    """Texto del reporte semanal (ultimos 7 dias + lo que viene)."""
    ahora = ahora or datetime.now()
    semana = calcular(ahora - timedelta(days=7), ahora)
    anterior = calcular(ahora - timedelta(days=14), ahora - timedelta(days=7))
    with db.conn() as c:
        proximas, sin_confirmar = c.execute(
            f"SELECT COUNT(*), COALESCE(SUM(estado = 'agendada'), 0) FROM citas "
            f"WHERE {ACTIVA} AND inicio >= ? AND inicio < ?",
            (ahora.strftime(FMT), (ahora + timedelta(days=7)).strftime(FMT)),
        ).fetchone()

    lineas = [
        f"*Reporte semanal – {NOMBRE}*",
        f"Del {(ahora - timedelta(days=7)).strftime('%d/%m')} al {ahora.strftime('%d/%m')}",
        "",
        f"Citas agendadas por el asistente: {semana['nuevas']}",
        f"Citas de la semana: {semana['total']}",
        f"• Canceladas con aviso (horario liberado): {semana['canceladas']}",
        f"• Lugares liberados que se volvieron a llenar con la lista de espera: {semana['rellenados']}",
        f"• Confirmadas por WhatsApp: {semana['confirmadas']} ({_pct(semana['tasa_confirmacion'])})",
        f"• Asistieron: {semana['asistencias']}",
        f"• Faltaron: {semana['faltas']}",
    ]
    if semana["atendibles"]:
        linea = f"*Inasistencia: {_pct(semana['tasa_faltas'])}*"
        if anterior["atendibles"]:
            linea += f" (semana anterior: {_pct(anterior['tasa_faltas'])})"
        lineas += ["", linea, "Referencia del sector dental sin recordatorios: 25-35%."]
    lineas += ["", f"Proximos 7 dias: {proximas} citas, {sin_confirmar} sin confirmar."]
    if semana["faltas"] == 0 and semana["atendibles"]:
        lineas.append("Si alguien falto y no lo marcaron, escribe: #falta <numero de cita>")
    return "\n".join(lineas)


def citas_de_hoy(ahora: datetime | None = None) -> str:
    """Lista para recepcion con el numero de cada cita (para marcar faltas)."""
    ahora = ahora or datetime.now()
    with db.conn() as c:
        filas = c.execute(
            f"SELECT id, inicio, nombre, estado, falto FROM citas WHERE {ACTIVA} AND inicio LIKE ? ORDER BY inicio",
            (ahora.strftime("%Y-%m-%d") + "%",),
        ).fetchall()
    if not filas:
        return "Hoy no hay citas."
    lineas = ["*Citas de hoy*"]
    for f in filas:
        marca = " – FALTO" if f["falto"] else (" ✓ confirmada" if f["estado"] == "confirmada" else "")
        lineas.append(f"#{f['id']} {f['inicio'][11:]} {f['nombre']}{marca}")
    lineas.append("Si alguien no llega: #falta <numero>")
    return "\n".join(lineas)


def marcar_falta(cita_id: int, falto: bool = True, ahora: datetime | None = None) -> bool:
    """Solo citas activas que ya empezaron; True si se marco."""
    ahora = ahora or datetime.now()
    with db.conn() as c:
        cur = c.execute(
            f"UPDATE citas SET falto=? WHERE id=? AND {ACTIVA} AND inicio <= ?",
            (int(falto), cita_id, ahora.strftime(FMT)),
        )
    return cur.rowcount > 0


def enviar_reporte():
    """Tarea semanal. Es texto libre: solo llega si el consultorio le escribio al bot
    en las ultimas 24 h. Si falla, recepcion lo puede pedir con #reporte."""
    destino = os.getenv("TELEFONO_HUMANO")
    if not destino:
        return
    try:
        whatsapp.enviar_texto(destino, reporte())
    except Exception as e:
        log.error("No se pudo enviar el reporte semanal: %s", e)
