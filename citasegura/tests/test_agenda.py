from datetime import datetime, timedelta

import sqlite3

import pytest

from app import agenda, db

TEL = "5215511111111"
OTRO = "5215522222222"


def test_horarios_libres_lunes(lunes):
    assert agenda.horarios_libres(lunes) == [f"{h:02d}:00" for h in range(10, 19)]


def test_domingo_cerrado(lunes):
    domingo = (datetime.strptime(lunes, "%Y-%m-%d") - timedelta(days=1)).strftime("%Y-%m-%d")
    assert agenda.horarios_libres(domingo) == []


def test_agendar_ocupa_el_horario(lunes):
    assert "agendada" in agenda.agendar(TEL, "Ana", "limpieza", lunes, "10:00")
    assert "10:00" not in agenda.horarios_libres(lunes)
    assert agenda.agendar(OTRO, "Beto", "dolor", lunes, "10:00") == "Ese horario ya no esta disponible."


def test_hora_en_varios_formatos(lunes):
    assert agenda._normalizar_hora("9:00") == "09:00"
    assert agenda._normalizar_hora(" 12 ") == "12:00"
    assert "agendada" in agenda.agendar(TEL, "Ana", "limpieza", lunes, "12")
    assert "12:00" not in agenda.horarios_libres(lunes)


def test_bd_impide_empalme_aunque_se_salten_la_revision(lunes):
    """Simula dos pacientes agendando al mismo instante."""
    with db.conn() as c:
        agenda._insertar(c, TEL, "Ana", "x", lunes, "13:00")
    with pytest.raises(sqlite3.IntegrityError), db.conn() as c:
        agenda._insertar(c, OTRO, "Beto", "x", lunes, "13:00")


def test_cancelar_libera_y_solo_el_dueno(lunes):
    agenda.agendar(TEL, "Ana", "limpieza", lunes, "10:00")
    cita = agenda.proxima_cita(TEL)
    assert agenda.cambiar_estado(OTRO, cita["id"], "cancelada") == "No encontre esa cita."
    assert agenda.cambiar_estado(TEL, cita["id"], "cancelada") == "Listo."
    assert "10:00" in agenda.horarios_libres(lunes)
    assert agenda.citas_del_paciente(TEL) == []
    # se puede volver a agendar la misma hora (el indice ignora canceladas)
    assert "agendada" in agenda.agendar(OTRO, "Beto", "x", lunes, "10:00")


def test_no_se_confirma_una_cita_cancelada(lunes):
    agenda.agendar(TEL, "Ana", "limpieza", lunes, "10:00")
    cita = agenda.proxima_cita(TEL)
    agenda.cambiar_estado(TEL, cita["id"], "cancelada")
    assert agenda.cambiar_estado(TEL, cita["id"], "confirmada") == "No encontre esa cita."


def test_reprogramar(lunes):
    agenda.agendar(TEL, "Ana", "limpieza", lunes, "10:00")
    cita = agenda.proxima_cita(TEL)
    assert "reprogramada" in agenda.reprogramar(TEL, cita["id"], lunes, "15:00")
    libres = agenda.horarios_libres(lunes)
    assert "10:00" in libres and "15:00" not in libres
    [nueva] = agenda.citas_del_paciente(TEL)
    assert nueva["inicio"].endswith("15:00") and nueva["motivo"] == "limpieza"


def test_reprogramar_a_hora_ocupada_no_toca_la_cita(lunes):
    agenda.agendar(TEL, "Ana", "limpieza", lunes, "10:00")
    agenda.agendar(OTRO, "Beto", "x", lunes, "11:00")
    cita = agenda.proxima_cita(TEL)
    assert agenda.reprogramar(TEL, cita["id"], lunes, "11:00") == "Ese horario ya no esta disponible."
    assert agenda.proxima_cita(TEL)["inicio"].endswith("10:00")


def test_reprogramar_cita_ajena(lunes):
    agenda.agendar(TEL, "Ana", "limpieza", lunes, "10:00")
    cita = agenda.proxima_cita(TEL)
    assert agenda.reprogramar(OTRO, cita["id"], lunes, "15:00") == "No encontre esa cita."
    assert "15:00" in agenda.horarios_libres(lunes)
