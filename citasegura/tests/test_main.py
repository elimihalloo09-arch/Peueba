from datetime import datetime, timedelta

import pytest

from app import agenda, db, ia, main

TEL = "5215511111111"
STAFF = "5215500000000"
_n = 0


def _msg(texto, de=TEL, tipo="text"):
    global _n
    _n += 1
    m = {"id": f"wamid.{_n}", "from": de, "type": tipo}
    m.update({"button": {"text": texto}} if tipo == "button" else {"text": {"body": texto}})
    return m


@pytest.fixture
def claude(monkeypatch):
    """Cuenta las veces que se llamaria a Claude."""
    llamadas = []
    monkeypatch.setattr(ia, "responder", lambda tel: llamadas.append(tel) or "respuesta de Claude")
    return llamadas


@pytest.fixture
def cita(lunes):
    agenda.agendar(TEL, "Ana", "limpieza", lunes, "10:00")
    return agenda.proxima_cita(TEL)


def test_boton_confirmo_sin_claude(enviados, claude, cita):
    main.procesar(_msg("Confirmo", tipo="button"))
    assert agenda.proxima_cita(TEL)["estado"] == "confirmada"
    assert claude == [] and "confirmar" in enviados[0][1]


def test_boton_cancelar_libera_horario(enviados, claude, cita, lunes):
    main.procesar(_msg("Cancelar", tipo="button"))
    assert agenda.proxima_cita(TEL) is None
    assert "10:00" in agenda.horarios_libres(lunes)
    assert claude == []


def test_boton_reprogramar_va_a_claude(enviados, claude, cita):
    main.procesar(_msg("Reprogramar", tipo="button"))
    assert claude == [TEL]


def test_texto_confirmo_va_a_claude(enviados, claude, cita):
    """Solo el boton se resuelve directo; si lo escribe, Claude decide con contexto."""
    main.procesar(_msg("confirmo"))
    assert claude == [TEL]


def test_mensaje_repetido_se_ignora(enviados, claude):
    m = _msg("hola")
    main.procesar(m)
    main.procesar(m)
    assert claude == [TEL]


def test_modo_humano_y_comando_bot(enviados, claude, monkeypatch):
    monkeypatch.setenv("TELEFONO_HUMANO", STAFF)
    db.activar_humano(TEL)
    main.procesar(_msg("me duele mucho"))
    assert claude == [] and enviados == []

    main.procesar(_msg(f"#bot {TEL}", de=STAFF))
    assert enviados[-1][0] == STAFF and "vuelve" in enviados[-1][1]
    assert not db.en_modo_humano(TEL)

    main.procesar(_msg("ya estoy mejor"))
    assert claude == [TEL]


def test_comando_bot_solo_desde_el_consultorio(enviados, claude, monkeypatch):
    monkeypatch.setenv("TELEFONO_HUMANO", STAFF)
    db.activar_humano(TEL)
    main.procesar(_msg(f"#bot {TEL}"))  # el paciente no se puede sacar solo
    assert db.en_modo_humano(TEL)


def test_comando_bot_mal_escrito(enviados, monkeypatch):
    monkeypatch.setenv("TELEFONO_HUMANO", STAFF)
    main.procesar(_msg("#bot", de=STAFF))
    assert "#bot 52" in enviados[-1][1]


def test_herramienta_con_datos_malos_no_truena():
    assert "Datos invalidos" in ia._ejecutar("ver_horarios", {"fecha": "manana"}, TEL)
    assert "Datos invalidos" in ia._ejecutar("agendar_cita", {"nombre": "Ana"}, TEL)


def test_herramientas_confirmar_y_reprogramar(cita, lunes):
    assert ia._ejecutar("confirmar_cita", {"cita_id": cita["id"]}, TEL) == "Listo."
    r = ia._ejecutar("reprogramar_cita", {"cita_id": cita["id"], "fecha": lunes, "hora": "16:00"}, TEL)
    assert "reprogramada" in r
