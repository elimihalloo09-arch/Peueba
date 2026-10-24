from datetime import datetime, timedelta

import pytest

from app import agenda, db, espera, main, metricas

ANA = "5215511111111"
BETO = "5215522222222"
CARLA = "5215533333333"
_n = 0


def _msg(de, texto, tipo="text"):
    global _n
    _n += 1
    m = {"id": f"wamid.esp{_n}", "from": de, "type": tipo}
    m.update({"button": {"text": texto}} if tipo == "button" else {"text": {"body": texto}})
    return m


@pytest.fixture
def lleno(lunes):
    """El lunes con todos sus horarios ocupados por otros pacientes."""
    for i, hora in enumerate(agenda.horarios_libres(lunes)):
        agenda.agendar(f"52155000000{i:02d}", f"Paciente {i}", "x", lunes, hora)
    assert agenda.horarios_libres(lunes) == []
    return lunes


def _cita_de(tel):
    return agenda.citas_del_paciente(tel)[0]


def _escribio(tel):
    db.guardar_mensaje(tel, "user", "hola")  # abre la ventana de 24 h


def _al(tel, enviados):
    return [t for x, t in enviados if x == tel]


def test_solo_se_anota_si_el_dia_esta_lleno(lunes):
    assert "si hay horarios libres" in espera.anotar(ANA, "Ana", "limpieza", lunes)


def test_anotar_y_no_duplicar(lleno):
    assert "Anotado" in espera.anotar(ANA, "Ana", "limpieza", lleno)
    assert "ya esta" in espera.anotar(ANA, "Ana", "limpieza", lleno)


def test_fecha_pasada_o_mal_escrita(lleno):
    ayer = (datetime.now() - timedelta(days=1)).strftime("%Y-%m-%d")
    assert "ya paso" in espera.anotar(ANA, "Ana", "x", ayer)
    with pytest.raises(ValueError):
        espera.anotar(ANA, "Ana", "x", "el lunes")


def test_cancelacion_se_ofrece_al_primero_y_acepta(enviados, lleno):
    espera.anotar(ANA, "Ana", "limpieza", lleno)
    espera.anotar(BETO, "Beto", "resina", lleno)
    _escribio(ANA)
    _escribio(BETO)
    cita = _cita_de("5215500000003")
    agenda.cambiar_estado("5215500000003", cita["id"], "cancelada")
    assert "Se libero un lugar" in _al(ANA, enviados)[-1]  # Ana se anoto primero
    assert _al(BETO, enviados) == []

    main.procesar(_msg(ANA, "Lo quiero", "button"))
    assert "Te agendamos" in _al(ANA, enviados)[-1] or "Como llegar" in _al(ANA, enviados)[-1]
    assert _cita_de(ANA)["inicio"] == cita["inicio"]
    with db.conn() as c:
        assert c.execute("SELECT estado FROM espera WHERE telefono=?", (ANA,)).fetchone()[0] == "tomada"


def test_si_dice_que_no_pasa_al_siguiente(enviados, lleno):
    espera.anotar(ANA, "Ana", "x", lleno)
    espera.anotar(BETO, "Beto", "x", lleno)
    _escribio(ANA)
    _escribio(BETO)
    cita = _cita_de("5215500000002")
    agenda.cambiar_estado("5215500000002", cita["id"], "cancelada")
    main.procesar(_msg(ANA, "No, gracias", "button"))
    assert "Sin problema" in _al(ANA, enviados)[-1]
    assert "Se libero un lugar" in _al(BETO, enviados)[-1]


def test_si_no_contesta_pasa_al_siguiente(enviados, lleno, monkeypatch):
    espera.anotar(ANA, "Ana", "x", lleno)
    espera.anotar(BETO, "Beto", "x", lleno)
    _escribio(ANA)
    _escribio(BETO)
    cita = _cita_de("5215500000001")
    agenda.cambiar_estado("5215500000001", cita["id"], "cancelada")
    hace_rato = (datetime.now() - timedelta(minutes=120)).strftime("%Y-%m-%d %H:%M")
    with db.conn() as c:
        c.execute("UPDATE espera SET ofrecido_en=? WHERE telefono=?", (hace_rato, ANA))
    espera.revisar()
    assert "Se libero un lugar" in _al(BETO, enviados)[-1]


def test_si_alguien_lo_tomo_primero(enviados, lleno):
    espera.anotar(ANA, "Ana", "x", lleno)
    _escribio(ANA)
    cita = _cita_de("5215500000004")
    agenda.cambiar_estado("5215500000004", cita["id"], "cancelada")
    agenda.agendar(CARLA, "Carla", "x", lleno, cita["inicio"][11:])  # lo agenda otra persona
    main.procesar(_msg(ANA, "lo quiero"))  # tambien por texto
    assert "ya lo tomo alguien" in _al(ANA, enviados)[-1]
    with db.conn() as c:
        assert c.execute("SELECT estado FROM espera WHERE telefono=?", (ANA,)).fetchone()[0] == "esperando"


def test_mover_una_cita_tambien_libera(enviados, lleno, monkeypatch):
    espera.anotar(ANA, "Ana", "x", lleno)
    _escribio(ANA)
    otro_dia = (datetime.strptime(lleno, "%Y-%m-%d") + timedelta(days=1)).strftime("%Y-%m-%d")
    cita = _cita_de("5215500000005")
    agenda.reprogramar("5215500000005", cita["id"], otro_dia, "12:00")
    assert "Se libero un lugar" in _al(ANA, enviados)[-1]


def test_fuera_de_24h_sin_plantilla_no_se_manda(enviados, lleno):
    espera.anotar(ANA, "Ana", "x", lleno)  # nunca escribio: ventana cerrada
    cita = _cita_de("5215500000006")
    agenda.cambiar_estado("5215500000006", cita["id"], "cancelada")
    assert _al(ANA, enviados) == []


def test_con_plantilla_llega_aunque_no_haya_escrito(enviados, lleno, monkeypatch):
    monkeypatch.setattr(espera, "PLANTILLA", "lugar_liberado")
    espera.anotar(ANA, "Ana", "x", lleno)
    cita = _cita_de("5215500000007")
    agenda.cambiar_estado("5215500000007", cita["id"], "cancelada")
    [(tel, params)] = [e for e in enviados if e[0] == ANA]
    assert params[0] == "Ana"


def test_si_agenda_por_su_cuenta_sale_de_la_lista(lleno):
    espera.anotar(ANA, "Ana", "x", lleno)
    cita = _cita_de("5215500000008")
    agenda.cambiar_estado("5215500000008", cita["id"], "cancelada")  # sin ventana: no se le ofrece
    agenda.agendar(ANA, "Ana", "x", lleno, cita["inicio"][11:])
    with db.conn() as c:
        assert c.execute("SELECT estado FROM espera WHERE telefono=?", (ANA,)).fetchone()[0] == "resuelta"


def test_lo_quiero_sin_oferta_va_a_claude(enviados, monkeypatch):
    from app import ia

    llamadas = []
    monkeypatch.setattr(ia, "responder", lambda tel: llamadas.append(tel) or "?")
    main.procesar(_msg(ANA, "lo quiero"))
    assert llamadas == [ANA]


def test_reporte_cuenta_lugares_rellenados(enviados, lleno):
    espera.anotar(ANA, "Ana", "x", lleno)
    _escribio(ANA)
    cita = _cita_de("5215500000000")
    agenda.cambiar_estado("5215500000000", cita["id"], "cancelada")
    espera.responder(ANA, acepta=True)
    assert "lista de espera: 1" in metricas.reporte()
