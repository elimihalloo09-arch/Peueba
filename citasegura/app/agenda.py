"""Logica de agenda: horarios libres, agendar, cancelar."""
from datetime import datetime, timedelta

from . import db
from .clinica import HORARIO


def horarios_libres(fecha: str) -> list[str]:
    """fecha 'YYYY-MM-DD' -> lista de horas libres 'HH:MM'."""
    dia = datetime.strptime(fecha, "%Y-%m-%d")
    if dia.weekday() not in HORARIO:
        return []
    ini, fin = HORARIO[dia.weekday()]
    with db.conn() as c:
        ocupadas = {
            f["inicio"][11:16]
            for f in c.execute(
                "SELECT inicio FROM citas WHERE inicio LIKE ? AND estado != 'cancelada'", (fecha + "%",)
            )
        }
    ahora = datetime.now()
    libres = []
    for h in range(ini, fin):
        slot = dia.replace(hour=h, minute=0)
        if slot > ahora + timedelta(hours=1) and f"{h:02d}:00" not in ocupadas:
            libres.append(f"{h:02d}:00")
    return libres


def agendar(telefono, nombre, motivo, fecha, hora) -> str:
    if hora not in horarios_libres(fecha):
        return "Ese horario ya no esta disponible."
    with db.conn() as c:
        c.execute(
            "INSERT INTO citas (telefono, nombre, motivo, inicio) VALUES (?,?,?,?)",
            (telefono, nombre, motivo, f"{fecha} {hora}"),
        )
    return f"Cita agendada: {fecha} {hora} a nombre de {nombre}."


def citas_del_paciente(telefono) -> list[dict]:
    with db.conn() as c:
        filas = c.execute(
            "SELECT id, inicio, motivo, estado FROM citas WHERE telefono=? AND estado != 'cancelada' "
            "AND inicio >= ? ORDER BY inicio",
            (telefono, datetime.now().strftime("%Y-%m-%d %H:%M")),
        ).fetchall()
    return [dict(f) for f in filas]


def cambiar_estado(telefono, cita_id, estado) -> str:
    with db.conn() as c:
        cur = c.execute("UPDATE citas SET estado=? WHERE id=? AND telefono=?", (estado, cita_id, telefono))
    return "Listo." if cur.rowcount else "No encontre esa cita."
