import pytest

from app import agenda, clinica, ia, main

TEL = "5215511111111"
_n = 0


def _msg(texto, tipo="text"):
    global _n
    _n += 1
    m = {"id": f"wamid.ll{_n}", "from": TEL, "type": tipo}
    m.update({"button": {"text": texto}} if tipo == "button" else {"text": {"body": texto}})
    return m


@pytest.fixture
def llegada(tmp_path, monkeypatch):
    archivo = tmp_path / "llegada.txt"
    archivo.write_text("Estacionamiento frente a la plaza.\nLocal 12, planta alta.\n", encoding="utf-8")
    monkeypatch.setattr(clinica, "LLEGADA_ARCHIVO", str(archivo))
    monkeypatch.setattr(clinica, "WAZE", "https://waze.com/ul/ejemplo")
    return archivo


def _al_paciente(enviados):
    return [t for tel, t in enviados if tel == TEL]


def test_mensaje_completo(llegada):
    texto = clinica.como_llegar()
    assert "Local 12, planta alta." in texto and clinica.DIRECCION in texto
    assert "Google Maps" in texto and "Waze: https://waze.com/ul/ejemplo" in texto


def test_sin_archivo_ni_waze(monkeypatch, tmp_path):
    monkeypatch.setattr(clinica, "LLEGADA_ARCHIVO", str(tmp_path / "no-existe.txt"))
    monkeypatch.setattr(clinica, "WAZE", "")
    texto = clinica.como_llegar()
    assert clinica.DIRECCION in texto and "Waze" not in texto


def test_al_confirmar_con_el_boton(enviados, llegada, lunes):
    agenda.agendar(TEL, "Ana", "limpieza", lunes, "10:00")
    main.procesar(_msg("Confirmo", "button"))
    msgs = _al_paciente(enviados)
    assert "Gracias por confirmar" in msgs[0] and "Local 12" in msgs[1]
    main.procesar(_msg("Confirmo", "button"))  # confirmar otra vez no repite las indicaciones
    assert len(_al_paciente(enviados)) == 3


def test_al_agendar_por_chat(enviados, llegada, lunes, monkeypatch):
    def claude_agenda(tel):
        agenda.agendar(tel, "Ana", "limpieza", lunes, "11:00")
        return "Listo, te agende el lunes a las 11."

    monkeypatch.setattr(ia, "responder", claude_agenda)
    main.procesar(_msg("quiero cita el lunes a las 11"))
    msgs = _al_paciente(enviados)
    assert msgs[0].startswith("Listo") and "Como llegar" in msgs[1]


def test_al_mover_la_cita(enviados, llegada, lunes, monkeypatch):
    agenda.agendar(TEL, "Ana", "limpieza", lunes, "10:00")
    cita = agenda.proxima_cita(TEL)
    monkeypatch.setattr(ia, "responder", lambda tel: agenda.reprogramar(tel, cita["id"], lunes, "16:00"))
    main.procesar(_msg("cambiala a las 4"))
    assert "Como llegar" in _al_paciente(enviados)[-1]


def test_una_duda_no_manda_indicaciones(enviados, llegada, lunes, monkeypatch):
    agenda.agendar(TEL, "Ana", "limpieza", lunes, "10:00")
    monkeypatch.setattr(ia, "responder", lambda tel: "La limpieza cuesta unos $600.")
    main.procesar(_msg("cuanto cuesta"))
    assert _al_paciente(enviados) == ["La limpieza cuesta unos $600."]


def test_claude_conoce_las_indicaciones(llegada, monkeypatch):
    capturado = {}

    class Falso:
        class messages:
            @staticmethod
            def create(**kw):
                capturado.update(kw)

                class R:
                    stop_reason = "end_turn"
                    content = []

                return R()

    monkeypatch.setattr(ia, "_cliente", lambda: Falso)
    from app import db

    db.guardar_mensaje(TEL, "user", "como llego?")
    ia.responder(TEL)
    assert "Local 12, planta alta." in capturado["system"]
