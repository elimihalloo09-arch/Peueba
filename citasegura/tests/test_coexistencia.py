import json
from datetime import datetime, timedelta

import pytest
from fastapi.testclient import TestClient

from app import db, ia, main

TEL = "5215511111111"
CLINICA = "5215599999999"
_n = 0


def _eco(texto="Hola Ana, soy Lupita de recepcion", tipo="text"):
    global _n
    _n += 1
    e = {"from": CLINICA, "to": TEL, "id": f"wamid.eco{_n}", "timestamp": "1790000000", "type": tipo}
    if tipo == "text":
        e["text"] = {"body": texto}
    return e


def _msg(texto):
    global _n
    _n += 1
    return {"id": f"wamid.m{_n}", "from": TEL, "type": "text", "text": {"body": texto}}


@pytest.fixture
def claude(monkeypatch):
    llamadas = []
    monkeypatch.setattr(ia, "responder", lambda tel: llamadas.append(tel) or "respuesta del bot")
    return llamadas


def test_recepcion_contesta_y_el_bot_se_pausa(enviados, claude):
    main.eco_de_recepcion(_eco())
    main.procesar(_msg("ok gracias, entonces a las 5?"))
    assert claude == [] and enviados == []  # el bot no le contesta encima a la recepcion
    with db.conn() as c:
        filas = [tuple(f) for f in c.execute("SELECT rol, texto FROM mensajes WHERE telefono=? ORDER BY id", (TEL,))]
    assert filas[0] == ("assistant", "Hola Ana, soy Lupita de recepcion")


def test_vuelve_con_comando_bot(enviados, claude, monkeypatch):
    monkeypatch.setenv("TELEFONO_HUMANO", "5215500000000")
    main.eco_de_recepcion(_eco())
    main.procesar({"id": "w.staff", "from": "5215500000000", "type": "text", "text": {"body": f"#bot {TEL}"}})
    main.procesar(_msg("hola"))
    assert claude == [TEL]


def test_vuelve_solo_tras_horas_sin_actividad(enviados, claude, monkeypatch):
    monkeypatch.setenv("PAUSA_HUMANO_HORAS", "12")
    viejo = (datetime.now() - timedelta(hours=13)).strftime("%Y-%m-%d %H:%M:%S")
    with db.conn() as c:
        c.execute("INSERT INTO humano (telefono, desde) VALUES (?, ?)", (TEL, viejo))
    main.procesar(_msg("hola otra vez"))
    assert claude == [TEL]


def test_cada_mensaje_de_recepcion_renueva_la_pausa(monkeypatch):
    viejo = (datetime.now() - timedelta(hours=13)).strftime("%Y-%m-%d %H:%M:%S")
    with db.conn() as c:
        c.execute("INSERT INTO humano (telefono, desde) VALUES (?, ?)", (TEL, viejo))
    main.eco_de_recepcion(_eco("sigo aqui"))
    assert db.en_modo_humano(TEL)


def test_pausa_sin_limite(monkeypatch):
    monkeypatch.setenv("PAUSA_HUMANO_HORAS", "0")
    with db.conn() as c:
        c.execute("INSERT INTO humano (telefono, desde) VALUES (?, '2020-01-01 00:00:00')", (TEL,))
    assert db.en_modo_humano(TEL)


def test_eco_repetido_o_sin_texto(enviados):
    e = _eco()
    main.eco_de_recepcion(e)
    main.eco_de_recepcion(e)  # Meta puede reenviarlo
    main.eco_de_recepcion(_eco(tipo="image"))  # foto, edicion o borrado: pausa pero no guarda texto
    with db.conn() as c:
        assert c.execute("SELECT COUNT(*) FROM mensajes").fetchone()[0] == 1
    assert db.en_modo_humano(TEL)


def test_webhook_con_eco(enviados, claude, monkeypatch):
    """Formato del webhook de coexistencia (campo smb_message_echoes)."""
    monkeypatch.delenv("WA_APP_SECRET", raising=False)
    cuerpo = {
        "object": "whatsapp_business_account",
        "entry": [{"id": "1", "changes": [{"field": "smb_message_echoes", "value": {
            "messaging_product": "whatsapp", "metadata": {"phone_number_id": "1"},
            "message_echoes": [_eco()]}}]}],
    }
    with TestClient(main.app) as c:
        assert c.post("/webhook", content=json.dumps(cuerpo)).status_code == 200
    assert db.en_modo_humano(TEL)
