"""Lista de espera: cuando alguien cancela o mueve su cita, el lugar se ofrece al primero que lo queria.

Flujo:
1. El dia que pide el paciente esta lleno -> Claude lo anota (anotar).
2. Se libera un horario de ese dia -> se le ofrece al primero de la lista (hueco_liberado),
   con botones "Lo quiero" / "No, gracias".
3. Si acepta, se agenda como cualquier cita. Si dice que no o no contesta en OFERTA_MINUTOS,
   el lugar pasa al siguiente.

El lugar ofrecido no se aparta: si otro paciente lo agenda antes, se le dice que ya se ocupo.
"""
import logging
import os
from datetime import datetime, timedelta

from . import avisos, db, whatsapp
from .clinica import NOMBRE

log = logging.getLogger("espera")
FMT = "%Y-%m-%d %H:%M"
PLANTILLA = os.getenv("WA_TEMPLATE_LISTA_ESPERA", "")
ACTIVA = "estado IN ('esperando','ofrecida')"


def _minutos_oferta() -> int:
    try:
        return int(os.getenv("OFERTA_MINUTOS", "90"))
    except ValueError:
        return 90


def anotar(telefono, nombre, motivo, fecha) -> str:
    from . import agenda  # aqui para evitar importacion circular (agenda avisa a espera)

    dia = datetime.strptime(fecha, "%Y-%m-%d")  # ValueError si viene mal: Claude lo corrige
    if dia.date() < datetime.now().date():
        return "Esa fecha ya paso."
    libres = agenda.horarios_libres(fecha)
    if libres:
        return f"Ese dia si hay horarios libres: {', '.join(libres)}. Ofrecelos en vez de la lista de espera."
    with db.conn() as c:
        ya = c.execute(f"SELECT 1 FROM espera WHERE telefono=? AND fecha=? AND {ACTIVA}", (telefono, fecha)).fetchone()
        if ya:
            return "El paciente ya esta en la lista de espera de ese dia."
        c.execute(
            "INSERT INTO espera (telefono, nombre, motivo, fecha) VALUES (?,?,?,?)", (telefono, nombre, motivo, fecha)
        )
    return f"Anotado en la lista de espera del {fecha}. Si se libera un lugar ese dia se le avisa por WhatsApp."


def _libre(inicio: str) -> bool:
    """El horario sigue libre y falta al menos una hora (igual que horarios_libres)."""
    if datetime.strptime(inicio, FMT) <= datetime.now() + timedelta(hours=1):
        return False
    with db.conn() as c:
        return not c.execute(
            "SELECT 1 FROM citas WHERE inicio=? AND estado IN ('agendada','confirmada')", (inicio,)
        ).fetchone()


def _ventana_abierta(telefono) -> bool:
    """El paciente escribio en las ultimas 24 h: se le puede mandar texto normal (gratis)."""
    limite = (datetime.now() - timedelta(hours=24)).strftime("%Y-%m-%d %H:%M:%S")
    with db.conn() as c:
        return bool(
            c.execute(
                "SELECT 1 FROM mensajes WHERE telefono=? AND rol='user' AND creado >= ?", (telefono, limite)
            ).fetchone()
        )


def _mandar_oferta(fila, inicio) -> bool:
    cuando = avisos.cuando(inicio)
    try:
        if PLANTILLA:
            # Plantilla de Utilidad con botones "Lo quiero" / "No, gracias" (ver README)
            whatsapp.enviar_plantilla(fila["telefono"], PLANTILLA, [fila["nombre"], NOMBRE, cuando])
        elif _ventana_abierta(fila["telefono"]):
            whatsapp.enviar_texto(
                fila["telefono"],
                f"¡Hola {fila['nombre']}! Se libero un lugar en {NOMBRE} el {cuando}. "
                "¿Lo quieres? Contesta *Lo quiero* o *No, gracias*.",
            )
        else:
            return False  # fuera de 24 h y sin plantilla aprobada: WhatsApp no lo entregaria
    except Exception as e:
        log.error("No se pudo ofrecer el lugar a la espera %s: %s", fila["id"], e)
        return False
    db.guardar_mensaje(fila["telefono"], "assistant", f"Te ofrecimos un lugar que se libero: {cuando}.")
    return True


def hueco_liberado(inicio: str):
    """Se cancelo o movio una cita: ofrecer ese horario al primero de la lista de ese dia."""
    if not _libre(inicio):
        return
    with db.conn() as c:
        if c.execute("SELECT 1 FROM espera WHERE estado='ofrecida' AND ofrecido=?", (inicio,)).fetchone():
            return  # ya se esta ofreciendo a alguien
        candidatos = c.execute(
            "SELECT * FROM espera WHERE estado='esperando' AND fecha=? ORDER BY id", (inicio[:10],)
        ).fetchall()
    for fila in candidatos:
        if _mandar_oferta(fila, inicio):
            with db.conn() as c:
                c.execute(
                    "UPDATE espera SET estado='ofrecida', ofrecido=?, ofrecido_en=? WHERE id=?",
                    (inicio, datetime.now().strftime(FMT), fila["id"]),
                )
            return


def responder(telefono, acepta: bool) -> list[str] | None:
    """Respuesta a "Lo quiero" / "No, gracias". None si no habia oferta para este paciente."""
    from . import agenda
    from .clinica import como_llegar

    with db.conn() as c:
        fila = c.execute(
            "SELECT * FROM espera WHERE telefono=? AND estado='ofrecida' ORDER BY ofrecido_en DESC", (telefono,)
        ).fetchone()
    if not fila:
        return None
    inicio = fila["ofrecido"]
    if not acepta:
        with db.conn() as c:
            c.execute("UPDATE espera SET estado='rechazada' WHERE id=?", (fila["id"],))
        hueco_liberado(inicio)  # pasa al siguiente
        return ["Sin problema, ya no te apartamos ese lugar. Si quieres otra fecha, aqui estoy."]
    resultado = agenda.agendar(telefono, fila["nombre"], fila["motivo"], inicio[:10], inicio[11:])
    if "agendada" not in resultado:
        with db.conn() as c:  # alguien mas lo tomo primero; sigue esperando por si se libera otro
            c.execute("UPDATE espera SET estado='esperando', ofrecido=NULL WHERE id=?", (fila["id"],))
        return ["¡Uy! Ese lugar ya lo tomo alguien mas. Sigues en la lista por si se libera otro ese dia."]
    return [f"¡Listo {fila['nombre']}! Te agendamos el {avisos.cuando(inicio)}. Te esperamos.", como_llegar()]


def al_agendar(telefono, inicio):
    """El paciente consiguio cita ese dia (por la oferta o por su cuenta): sale de la lista."""
    with db.conn() as c:
        c.execute(
            f"UPDATE espera SET estado = CASE WHEN estado='ofrecida' AND ofrecido=? THEN 'tomada' ELSE 'resuelta' END, "
            f"tomada_en=? WHERE telefono=? AND fecha=? AND {ACTIVA}",
            (inicio, datetime.now().strftime(FMT), telefono, inicio[:10]),
        )
    with db.conn() as c:
        tomada = c.execute(
            "SELECT 1 FROM espera WHERE telefono=? AND ofrecido=? AND estado='tomada'", (telefono, inicio)
        ).fetchone()
    return bool(tomada)


def revisar():
    """Cada 10 minutos: ofertas sin respuesta pasan al siguiente; listas de dias pasados se cierran."""
    limite = (datetime.now() - timedelta(minutes=_minutos_oferta())).strftime(FMT)
    with db.conn() as c:
        vencidas = c.execute(
            "SELECT id, ofrecido FROM espera WHERE estado='ofrecida' AND ofrecido_en < ?", (limite,)
        ).fetchall()
        for f in vencidas:
            c.execute("UPDATE espera SET estado='vencida' WHERE id=?", (f["id"],))
        c.execute(
            f"UPDATE espera SET estado='vencida' WHERE {ACTIVA} AND fecha < ?", (datetime.now().strftime("%Y-%m-%d"),)
        )
    for f in vencidas:
        hueco_liberado(f["ofrecido"])
