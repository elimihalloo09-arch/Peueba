from datetime import datetime, timedelta

from app import agenda, db, recordatorios

TEL = "5215511111111"


def _cita(horas, r48=0, r2=0):
    inicio = (datetime.now() + timedelta(hours=horas)).strftime("%Y-%m-%d %H:%M")
    with db.conn() as c:
        c.execute(
            "INSERT INTO citas (telefono, nombre, inicio, recordatorio_48h, recordatorio_2h) VALUES (?,?,?,?,?)",
            (TEL, "Ana", inicio, r48, r2),
        )


def _flags():
    with db.conn() as c:
        f = c.execute("SELECT recordatorio_48h, recordatorio_2h FROM citas").fetchone()
    return f["recordatorio_48h"], f["recordatorio_2h"]


def test_48h_y_luego_2h(enviados):
    _cita(30)
    recordatorios.revisar()
    recordatorios.revisar()  # la siguiente vuelta no repite
    assert len(enviados) == 1 and _flags() == (1, 0)


def test_cita_cercana_recibe_un_solo_recordatorio(enviados):
    """Antes salian el de 48 h y 10 min despues el de 2 h."""
    _cita(1.5)
    recordatorios.revisar()
    recordatorios.revisar()
    assert len(enviados) == 1 and _flags() == (1, 1)


def test_lejana_no_recibe_nada(enviados):
    _cita(72)
    recordatorios.revisar()
    assert enviados == []


def test_agendar_dentro_de_48h_no_programa_el_de_48h():
    """Quien agenda para manana acaba de hablar con el bot: el de 48 h sobra."""
    for horas in (30, 72):
        inicio = datetime.now() + timedelta(hours=horas)
        with db.conn() as c:
            agenda._insertar(c, TEL, "Ana", "x", inicio.strftime("%Y-%m-%d"), inicio.strftime("%H:%M"))
    with db.conn() as c:
        flags = [f[0] for f in c.execute("SELECT recordatorio_48h FROM citas ORDER BY inicio")]
    assert flags == [1, 0]


def test_cancelada_no_recibe(enviados):
    _cita(30)
    with db.conn() as c:
        c.execute("UPDATE citas SET estado='cancelada'")
    recordatorios.revisar()
    assert enviados == []


def test_recordatorio_queda_en_historial(enviados):
    _cita(30)
    recordatorios.revisar()
    with db.conn() as c:
        assert c.execute("SELECT rol FROM mensajes WHERE telefono=?", (TEL,)).fetchone()["rol"] == "assistant"
