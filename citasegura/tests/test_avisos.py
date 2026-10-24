import pytest

from app import agenda, avisos, main

TEL = "5215511111111"
STAFF = "5215500000000"


@pytest.fixture
def consultorio(monkeypatch):
    monkeypatch.setenv("TELEFONO_HUMANO", STAFF)


def _al_consultorio(enviados):
    return [txt for tel, txt in enviados if tel == STAFF]


def test_cuando():
    assert avisos.cuando("2026-10-05 10:00") == "lun 05/10 10:00"


def test_aviso_de_cita_nueva(enviados, consultorio, lunes):
    agenda.agendar(TEL, "Ana Lopez", "limpieza", lunes, "10:00")
    [aviso] = _al_consultorio(enviados)
    assert aviso.startswith("Nueva cita: Ana Lopez, lun") and "limpieza" in aviso and TEL in aviso


def test_aviso_de_cancelacion_pero_no_de_confirmacion(enviados, consultorio, lunes):
    agenda.agendar(TEL, "Ana", "x", lunes, "10:00")
    cita = agenda.proxima_cita(TEL)
    agenda.cambiar_estado(TEL, cita["id"], "confirmada")
    assert len(_al_consultorio(enviados)) == 1  # solo el de la cita nueva
    agenda.cambiar_estado(TEL, cita["id"], "cancelada")
    assert _al_consultorio(enviados)[-1].startswith("Cancelada: Ana")


def test_aviso_de_cita_movida(enviados, consultorio, lunes):
    agenda.agendar(TEL, "Ana", "x", lunes, "10:00")
    cita = agenda.proxima_cita(TEL)
    agenda.reprogramar(TEL, cita["id"], lunes, "15:00")
    aviso = _al_consultorio(enviados)[-1]
    assert aviso.startswith("Cita movida: Ana") and "10:00" in aviso and "15:00" in aviso


def test_sin_avisos_si_algo_falla(enviados, consultorio, lunes):
    agenda.agendar(TEL, "Ana", "x", lunes, "10:00")
    enviados.clear()
    agenda.agendar("5215522222222", "Beto", "x", lunes, "10:00")  # hora ocupada
    agenda.cambiar_estado(TEL, 999, "cancelada")  # cita que no existe
    assert enviados == []


def test_boton_cancelar_avisa(enviados, consultorio, lunes, monkeypatch):
    agenda.agendar(TEL, "Ana", "x", lunes, "10:00")
    main.procesar({"id": "w.btn", "from": TEL, "type": "button", "button": {"text": "Cancelar"}})
    assert _al_consultorio(enviados)[-1].startswith("Cancelada: Ana")


def test_con_plantilla(enviados, consultorio, monkeypatch, lunes):
    monkeypatch.setenv("WA_TEMPLATE_AVISO", "aviso_consultorio")
    agenda.agendar(TEL, "Ana", "x", lunes, "10:00")
    [(tel, params)] = enviados
    assert tel == STAFF and isinstance(params, list) and params[0].startswith("Nueva cita")


def test_se_pueden_apagar(enviados, consultorio, monkeypatch, lunes):
    monkeypatch.setenv("AVISAR_CONSULTORIO", "0")
    agenda.agendar(TEL, "Ana", "x", lunes, "10:00")
    assert enviados == []


def test_sin_telefono_del_consultorio_no_manda_nada(enviados, lunes):
    agenda.agendar(TEL, "Ana", "x", lunes, "10:00")
    assert enviados == []


def test_si_whatsapp_falla_la_cita_igual_se_agenda(monkeypatch, consultorio, lunes):
    def falla(*a):
        raise RuntimeError("Meta caido")

    monkeypatch.setattr(avisos.whatsapp, "enviar_texto", falla)
    assert "agendada" in agenda.agendar(TEL, "Ana", "x", lunes, "10:00")
