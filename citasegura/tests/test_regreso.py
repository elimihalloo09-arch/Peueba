from datetime import datetime, timedelta

import pytest

from app import db, main, metricas, regreso

ANA = "5215511111111"
BETO = "5215522222222"
AHORA = datetime(2026, 10, 12, 10, 50)


def _cita(tel, nombre, motivo, inicio: datetime, estado="confirmada", falto=0):
    with db.conn() as c:
        return c.execute(
            "INSERT INTO citas (telefono, nombre, motivo, inicio, estado, falto, creado) VALUES (?,?,?,?,?,?,?)",
            (tel, nombre, motivo, inicio.strftime("%Y-%m-%d %H:%M"), estado, falto, "2026-01-01 00:00"),
        ).lastrowid


@pytest.fixture
def activo(monkeypatch):
    monkeypatch.setenv("REGRESO", "1")
    monkeypatch.setenv("WA_TEMPLATE_REGRESO", "regreso_cita")
    monkeypatch.delenv("REGRESO_REGLAS", raising=False)


def test_reglas():
    assert regreso.reglas() == {"limpieza": 6}


def test_reglas_configurables(monkeypatch):
    monkeypatch.setenv("REGRESO_REGLAS", "Limpieza:6, revisión:12,mal")
    assert regreso.reglas() == {"limpieza": 6, "revision": 12}


def test_sumar_meses_fin_de_mes():
    assert regreso._sumar_meses(datetime(2026, 8, 31, 10), 6) == datetime(2027, 2, 28, 10)


def test_le_toca_a_quien_vino_hace_6_meses(enviados, activo):
    _cita(ANA, "Ana", "Limpieza dental", datetime(2026, 4, 8, 11))  # 6 meses y 4 dias
    regreso.revisar(AHORA)
    [(tel, params)] = enviados
    assert tel == ANA and params == ["Ana", "limpieza", "Clinica Dental Sonrisa"]
    regreso.revisar(AHORA)  # al dia siguiente no se repite
    assert len(enviados) == 1


def test_todavia_no(enviados, activo):
    _cita(ANA, "Ana", "limpieza", datetime(2026, 5, 1, 11))  # 5 meses
    regreso.revisar(AHORA)
    assert enviados == []


def test_historial_viejo_no_se_dispara(enviados, activo):
    _cita(ANA, "Ana", "limpieza", datetime(2025, 1, 10, 11))  # vencio hace mucho
    regreso.revisar(AHORA)
    assert enviados == []


def test_solo_cuenta_la_ultima_cita(enviados, activo):
    _cita(ANA, "Ana", "limpieza", datetime(2026, 4, 8, 11))
    _cita(ANA, "Ana", "resina", datetime(2026, 9, 1, 11))  # vino despues por otra cosa
    regreso.revisar(AHORA)
    assert enviados == []


@pytest.mark.parametrize("estado,falto", [("cancelada", 0), ("confirmada", 1)])
def test_cancelada_o_falto_no(enviados, activo, estado, falto):
    _cita(ANA, "Ana", "limpieza", datetime(2026, 4, 8, 11), estado=estado, falto=falto)
    regreso.revisar(AHORA)
    assert enviados == []


def test_si_ya_tiene_cita_futura_no(enviados, activo):
    _cita(ANA, "Ana", "limpieza", datetime(2026, 4, 8, 11))
    _cita(ANA, "Ana", "limpieza", datetime(2026, 10, 20, 11), estado="agendada")
    regreso.revisar(AHORA)
    assert enviados == []


def test_apagado_o_sin_plantilla(enviados, monkeypatch):
    _cita(ANA, "Ana", "limpieza", datetime(2026, 4, 8, 11))
    monkeypatch.setenv("REGRESO", "1")
    monkeypatch.delenv("WA_TEMPLATE_REGRESO", raising=False)
    regreso.revisar(AHORA)
    monkeypatch.setenv("WA_TEMPLATE_REGRESO", "x")
    monkeypatch.setenv("REGRESO", "0")
    regreso.revisar(AHORA)
    assert enviados == []


def test_no_por_ahora_lo_da_de_baja(enviados, activo):
    main.procesar({"id": "w.reg1", "from": ANA, "type": "button", "button": {"text": "No por ahora"}})
    assert "ya no te mandaremos" in enviados[-1][1]
    enviados.clear()
    _cita(ANA, "Ana", "limpieza", datetime(2026, 4, 8, 11))
    regreso.revisar(AHORA)
    assert enviados == []


def test_si_falla_el_envio_se_intenta_otro_dia(activo, monkeypatch):
    def falla(*a):
        raise RuntimeError("Meta caido")

    monkeypatch.setattr(regreso.whatsapp, "enviar_plantilla", falla)
    _cita(ANA, "Ana", "limpieza", datetime(2026, 4, 8, 11))
    regreso.revisar(AHORA)
    assert len(regreso.pendientes(AHORA)) == 1


def test_queda_en_el_historial_para_claude(enviados, activo):
    _cita(ANA, "Ana", "limpieza", datetime(2026, 4, 8, 11))
    regreso.revisar(AHORA)
    with db.conn() as c:
        texto = c.execute("SELECT texto FROM mensajes WHERE telefono=?", (ANA,)).fetchone()[0]
    assert "ya toca tu limpieza" in texto


def test_reporte_cuenta_a_los_que_regresaron(enviados, activo):
    _cita(ANA, "Ana", "limpieza", datetime(2026, 4, 8, 11))
    _cita(BETO, "Beto", "limpieza", datetime(2026, 4, 9, 11))
    regreso.revisar(AHORA - timedelta(days=3))
    with db.conn() as c:  # Ana agendo 2 dias despues del recordatorio; Beto no
        c.execute(
            "INSERT INTO citas (telefono, nombre, motivo, inicio, creado) VALUES (?,?,?,?,?)",
            (ANA, "Ana", "limpieza", "2026-10-20 11:00", (AHORA - timedelta(days=1)).strftime("%Y-%m-%d %H:%M")),
        )
    assert regreso.regresaron(AHORA - timedelta(days=7), AHORA) == 1
    assert "gracias al recordatorio de regreso: 1" in metricas.reporte(AHORA)
