"""Logica de agenda: horarios libres, agendar, confirmar, cancelar, reprogramar."""
import sqlite3
from datetime import datetime, timedelta

from . import avisos, db
from .clinica import HORARIO


def _normalizar_hora(hora: str) -> str:
    """'9:00' o '9' -> '09:00'. Lanza ValueError si no es una hora valida."""
    hora = hora.strip()
    if ":" not in hora:
        hora += ":00"
    return datetime.strptime(hora, "%H:%M").strftime("%H:%M")


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
                "SELECT inicio FROM citas WHERE inicio LIKE ? AND estado IN ('agendada','confirmada')", (fecha + "%",)
            )
        }
    ahora = datetime.now()
    libres = []
    for h in range(ini, fin):
        slot = dia.replace(hour=h, minute=0)
        if slot > ahora + timedelta(hours=1) and f"{h:02d}:00" not in ocupadas:
            libres.append(f"{h:02d}:00")
    return libres


def _insertar(c, telefono, nombre, motivo, fecha, hora, creado=None):
    """Inserta la cita; el indice unico de la BD impide dar la misma hora a dos pacientes."""
    inicio = f"{fecha} {hora}"
    ahora = datetime.now()
    # si faltan menos de 48 h el paciente acaba de agendar: el recordatorio de 48 h sobra
    sin_48h = datetime.strptime(inicio, "%Y-%m-%d %H:%M") - ahora <= timedelta(hours=48)
    c.execute(
        "INSERT INTO citas (telefono, nombre, motivo, inicio, recordatorio_48h, creado) VALUES (?,?,?,?,?,?)",
        (telefono, nombre, motivo, inicio, int(sin_48h), creado or ahora.strftime("%Y-%m-%d %H:%M")),
    )


def agendar(telefono, nombre, motivo, fecha, hora) -> str:
    hora = _normalizar_hora(hora)
    if hora not in horarios_libres(fecha):
        return "Ese horario ya no esta disponible."
    try:
        with db.conn() as c:
            _insertar(c, telefono, nombre, motivo, fecha, hora)
    except sqlite3.IntegrityError:  # otro paciente la tomo en el mismo instante
        return "Ese horario ya no esta disponible."
    from . import espera  # aqui para evitar importacion circular

    de_espera = espera.al_agendar(telefono, f"{fecha} {hora}")
    origen = " (lugar liberado, de la lista de espera)" if de_espera else ""
    avisos.avisar(f"Nueva cita{origen}: {nombre}, {avisos.cuando(f'{fecha} {hora}')} ({motivo}). Tel {telefono}")
    return f"Cita agendada: {fecha} {hora} a nombre de {nombre}."


def citas_del_paciente(telefono) -> list[dict]:
    with db.conn() as c:
        filas = c.execute(
            "SELECT id, inicio, motivo, estado FROM citas WHERE telefono=? AND estado IN ('agendada','confirmada') "
            "AND inicio >= ? ORDER BY inicio",
            (telefono, datetime.now().strftime("%Y-%m-%d %H:%M")),
        ).fetchall()
    return [dict(f) for f in filas]


def cambiar_estado(telefono, cita_id, estado) -> str:
    with db.conn() as c:
        cita = c.execute(
            "SELECT nombre, inicio FROM citas WHERE id=? AND telefono=? AND estado IN ('agendada','confirmada')",
            (cita_id, telefono),
        ).fetchone()
        if not cita:
            return "No encontre esa cita."
        c.execute("UPDATE citas SET estado=? WHERE id=?", (estado, cita_id))
    if estado == "cancelada":  # las confirmaciones no se avisan: se ven en #hoy y no gastan mensajes
        avisos.avisar(f"Cancelada: {cita['nombre']}, {avisos.cuando(cita['inicio'])}. El horario quedo libre.")
        from . import espera

        espera.hueco_liberado(cita["inicio"])
    return "Listo."


def reprogramar(telefono, cita_id, fecha, hora) -> str:
    """Cancela la cita y crea la nueva en un solo paso: o pasan las dos cosas o ninguna."""
    hora = _normalizar_hora(hora)
    if hora not in horarios_libres(fecha):
        return "Ese horario ya no esta disponible."
    try:
        with db.conn() as c:
            vieja = c.execute(
                "SELECT nombre, motivo, creado, inicio FROM citas "
                "WHERE id=? AND telefono=? AND estado IN ('agendada','confirmada')",
                (cita_id, telefono),
            ).fetchone()
            if not vieja:
                return "No encontre esa cita."
            c.execute("UPDATE citas SET estado='reprogramada' WHERE id=?", (cita_id,))
            # conserva la fecha de creacion: mover una cita no es una cita nueva en las metricas
            _insertar(c, telefono, vieja["nombre"], vieja["motivo"], fecha, hora, vieja["creado"])
    except sqlite3.IntegrityError:
        return "Ese horario ya no esta disponible."
    avisos.avisar(
        f"Cita movida: {vieja['nombre']}, de {avisos.cuando(vieja['inicio'])} a {avisos.cuando(f'{fecha} {hora}')}."
    )
    from . import espera

    espera.al_agendar(telefono, f"{fecha} {hora}")
    espera.hueco_liberado(vieja["inicio"])  # el horario viejo quedo libre
    return f"Cita reprogramada: {fecha} {hora} a nombre de {vieja['nombre']}."


def proxima_cita(telefono) -> dict | None:
    """La siguiente cita activa del paciente (la que se le recordo)."""
    citas = citas_del_paciente(telefono)
    return citas[0] if citas else None
