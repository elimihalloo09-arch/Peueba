from datetime import datetime, timedelta

from app import agenda, db, main, metricas

AHORA = datetime(2026, 10, 12, 9, 0)  # lunes 9:00, cuando sale el reporte
STAFF = "5215500000000"


def _cita(dias, estado="agendada", falto=0, creado_dias=None, hora=10):
    """dias: cuantos dias antes (negativo) o despues de AHORA es la cita."""
    inicio = (AHORA + timedelta(days=dias)).replace(hour=hora).strftime("%Y-%m-%d %H:%M")
    creado = (AHORA + timedelta(days=creado_dias if creado_dias is not None else dias - 3)).strftime(
        "%Y-%m-%d %H:%M"
    )
    with db.conn() as c:
        cur = c.execute(
            "INSERT INTO citas (telefono, nombre, inicio, estado, falto, creado) VALUES (?,?,?,?,?,?)",
            ("521551", "Ana", inicio, estado, falto, creado),
        )
    return cur.lastrowid


def test_calculo_de_la_semana():
    _cita(-6, "confirmada", hora=10)
    _cita(-5, "confirmada", hora=11)
    _cita(-4, "agendada", falto=1, hora=12)
    _cita(-3, "cancelada", hora=13)
    _cita(-2, "reprogramada", hora=14)  # se movio: no cuenta
    _cita(-20, "agendada", falto=1)  # fuera del rango
    m = metricas.calcular(AHORA - timedelta(days=7), AHORA)
    assert (m["total"], m["canceladas"], m["confirmadas"], m["faltas"]) == (4, 1, 2, 1)
    assert m["atendibles"] == 3 and m["asistencias"] == 2
    assert round(m["tasa_faltas"], 2) == 0.33


def test_semana_vacia_no_divide_entre_cero():
    m = metricas.calcular(AHORA - timedelta(days=7), AHORA)
    assert m["tasa_faltas"] == 0 and m["total"] == 0
    assert "Inasistencia" not in metricas.reporte(AHORA)


def test_reporte_con_comparacion():
    _cita(-3, "confirmada")
    _cita(-2, "agendada", falto=1)
    _cita(-10, "agendada", falto=1)  # semana anterior: 1 de 1
    _cita(2, "agendada")  # proxima semana sin confirmar
    texto = metricas.reporte(AHORA)
    assert "Inasistencia: 50%" in texto and "semana anterior: 100%" in texto
    assert "Proximos 7 dias: 1 citas, 1 sin confirmar" in texto


def test_reprogramar_no_cuenta_como_cancelacion_ni_cita_nueva(lunes):
    agenda.agendar("521551", "Ana", "limpieza", lunes, "10:00")
    cita = agenda.proxima_cita("521551")
    agenda.reprogramar("521551", cita["id"], lunes, "12:00")
    ahora = datetime.now()
    m = metricas.calcular(ahora - timedelta(days=1), ahora + timedelta(days=30))
    assert (m["total"], m["canceladas"], m["nuevas"]) == (1, 0, 1)


def test_marcar_falta_solo_si_ya_paso():
    pasada = _cita(-1)
    futura = _cita(2)
    cancelada = _cita(-1, "cancelada", hora=15)
    assert metricas.marcar_falta(pasada, ahora=AHORA)
    assert not metricas.marcar_falta(futura, ahora=AHORA)
    assert not metricas.marcar_falta(cancelada, ahora=AHORA)
    assert metricas.marcar_falta(pasada, falto=False, ahora=AHORA)


def test_citas_de_hoy():
    hoy = _cita(0, "confirmada", hora=11)
    texto = metricas.citas_de_hoy(AHORA)
    assert f"#{hoy} 11:00 Ana ✓ confirmada" in texto
    assert metricas.citas_de_hoy(AHORA + timedelta(days=1)) == "Hoy no hay citas."


def test_migracion_de_base_vieja(tmp_path, monkeypatch):
    """Una base creada antes de las metricas recibe las columnas nuevas sin perder citas."""
    monkeypatch.setattr(db, "DB_PATH", str(tmp_path / "vieja.db"))
    with db.conn() as c:
        c.executescript("""
        CREATE TABLE citas (id INTEGER PRIMARY KEY AUTOINCREMENT, telefono TEXT NOT NULL,
            nombre TEXT NOT NULL, motivo TEXT, inicio TEXT NOT NULL, estado TEXT DEFAULT 'agendada',
            recordatorio_48h INTEGER DEFAULT 0, recordatorio_2h INTEGER DEFAULT 0);
        INSERT INTO citas (telefono, nombre, inicio) VALUES ('521551', 'Ana', '2026-10-01 10:00');
        CREATE UNIQUE INDEX una_cita_por_hora ON citas(inicio) WHERE estado != 'cancelada';
        """)
    db.init()
    db.init()  # correr dos veces no truena
    with db.conn() as c:
        f = c.execute("SELECT falto, creado FROM citas").fetchone()
        indices = {r["name"] for r in c.execute("PRAGMA index_list(citas)")}
    assert f["falto"] == 0 and f["creado"] is None
    assert "una_cita_activa_por_hora" in indices and "una_cita_por_hora" not in indices


def _staff(texto, monkeypatch, enviados):
    monkeypatch.setenv("TELEFONO_HUMANO", STAFF)
    _staff.n = getattr(_staff, "n", 0) + 1
    main.procesar({"id": f"staff.{_staff.n}", "from": STAFF, "type": "text", "text": {"body": texto}})
    return enviados[-1][1]


def test_comandos_del_consultorio(enviados, monkeypatch):
    pasada = (datetime.now() - timedelta(hours=1)).strftime("%Y-%m-%d %H:%M")
    with db.conn() as c:
        cid = c.execute("INSERT INTO citas (telefono, nombre, inicio) VALUES ('521551','Ana',?)", (pasada,)).lastrowid
    assert "Reporte semanal" in _staff("#reporte", monkeypatch, enviados)
    assert f"#{cid}" in _staff("#hoy", monkeypatch, enviados) or pasada[:10] != datetime.now().strftime("%Y-%m-%d")
    assert "no llego" in _staff(f"#falta {cid}", monkeypatch, enviados)
    assert "Faltaron: 1" in _staff("#reporte", monkeypatch, enviados)
    assert "si llego" in _staff(f"#asistio #{cid}", monkeypatch, enviados)
    assert "No encontre" in _staff("#falta 999", monkeypatch, enviados)
    assert "Comandos" in _staff("#ayuda", monkeypatch, enviados)
    # nada de esto se guarda como conversacion ni pasa por Claude
    with db.conn() as c:
        assert c.execute("SELECT COUNT(*) FROM mensajes").fetchone()[0] == 0


def test_reporte_semanal_automatico(enviados, monkeypatch):
    monkeypatch.setenv("TELEFONO_HUMANO", STAFF)
    metricas.enviar_reporte()
    assert enviados[-1][0] == STAFF and "Reporte semanal" in enviados[-1][1]


def test_reporte_semanal_sin_telefono_no_hace_nada(enviados, monkeypatch):
    monkeypatch.delenv("TELEFONO_HUMANO", raising=False)
    metricas.enviar_reporte()
    assert enviados == []
