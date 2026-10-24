import pytest

from app import agenda, db, ia, main, ventas, whatsapp

DOC = "5215577777777"
YO = "5215500000000"
_n = 0


def _msg(texto, de=DOC, tipo="text"):
    global _n
    _n += 1
    m = {"id": f"wamid.v{_n}", "from": de, "type": tipo}
    if tipo == "interactive":
        m["interactive"] = {"type": "button_reply", "button_reply": {"id": "b0", "title": texto}}
    else:
        m["text"] = {"body": texto}
    return m


@pytest.fixture
def modo_ventas(monkeypatch):
    monkeypatch.setenv("MODO", "ventas")
    monkeypatch.setenv("TELEFONO_HUMANO", YO)
    monkeypatch.setenv("VENDEDOR_NOMBRE", "Eli")


@pytest.fixture
def botones(monkeypatch):
    mandados = []
    monkeypatch.setattr(whatsapp, "enviar_botones", lambda tel, texto, bts: mandados.append((tel, texto, bts)))
    return mandados


def test_calculo_de_perdida():
    r = ventas.calcular_perdida(4, 500)
    assert "Faltas al mes: 17" in r and "$8,500" in r and "$2,833" in r and "4.1 veces" in r


def test_calculo_con_datos_malos():
    assert "Datos invalidos" in ia._ejecutar("calcular_perdida", {"faltas_semana": 0, "precio_consulta": 500}, DOC)
    assert "Datos invalidos" in ia._ejecutar("calcular_perdida", {"faltas_semana": "muchas", "precio_consulta": 500}, DOC)


def test_prompt_y_herramientas_de_ventas(modo_ventas, monkeypatch):
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
    db.guardar_mensaje(DOC, "user", "hola, vi su anuncio")
    ia.responder(DOC)
    nombres = {t["name"] for t in capturado["tools"]}
    assert {"calcular_perdida", "demo_recordatorio", "registrar_interesado", "agendar_cita"} <= nombres
    assert "lista_de_espera" not in nombres
    assert "asistente de ventas de CitaSegura" in capturado["system"] and "Eli le escribe" in capturado["system"]


def test_modo_normal_no_trae_herramientas_de_ventas(monkeypatch):
    monkeypatch.delenv("MODO", raising=False)
    assert not ventas.NOMBRES & {t["name"] for t in ia.TOOLS}


def test_demo_recordatorio_con_botones(enviados, modo_ventas, botones, lunes):
    assert "Todavia no tiene cita" in ventas.demo_recordatorio(DOC)
    agenda.agendar(DOC, "Dr. Ramirez", "limpieza", lunes, "10:00")
    ventas.demo_recordatorio(DOC)
    [(tel, texto, bts)] = botones
    assert tel == DOC and "Dr. Ramirez" in texto and bts == ["Confirmo", "Reprogramar", "Cancelar"]


def test_tocar_confirmo_en_la_demo(enviados, modo_ventas, botones, lunes):
    agenda.agendar(DOC, "Dr. Ramirez", "limpieza", lunes, "10:00")
    main.procesar(_msg("Confirmo", tipo="interactive"))
    al_doc = [t for tel, t in enviados if tel == DOC]
    assert "Gracias por confirmar" in al_doc[0] and "Como llegar" in al_doc[1]
    assert agenda.proxima_cita(DOC)["estado"] == "confirmada"


def test_las_citas_de_prueba_no_avisan_al_vendedor(enviados, modo_ventas, lunes):
    agenda.agendar(DOC, "Dr. Ramirez", "limpieza", lunes, "10:00")
    assert [t for tel, t in enviados if tel == YO] == []


def test_interesado_avisa_al_vendedor_y_el_bot_se_hace_a_un_lado(enviados, modo_ventas, monkeypatch):
    def claude(tel):
        ia._ejecutar("registrar_interesado", {
            "nombre": "Dr. Ramirez", "consultorio": "Sonrisas Ramirez", "zona": "Neza centro",
            "faltas_semana": 5, "precio_consulta": 600, "whatsapp_business": "si", "interes": "piloto",
        }, tel)
        return "Perfecto doctor, Eli le escribe hoy mismo."

    monkeypatch.setattr(ia, "responder", claude)
    main.procesar(_msg("Si, quiero el piloto"))
    [aviso] = [t for tel, t in enviados if tel == YO]
    assert aviso.startswith("Interesado (piloto): Dr. Ramirez") and "5 faltas/semana" in aviso and "$600" in aviso
    assert f"#bot {DOC}" in aviso
    assert db.en_modo_humano(DOC)

    llamadas = []
    monkeypatch.setattr(ia, "responder", lambda tel: llamadas.append(tel) or "?")
    main.procesar(_msg("¿entonces cuando empezamos?"))
    assert llamadas == []  # ya lo atiende el vendedor


def test_comando_interesados(enviados, modo_ventas):
    ventas.registrar_interesado(DOC, {"nombre": "Dra. Lopez", "interes": "llamada"})
    main.procesar(_msg("#interesados", de=YO))
    assert "Dra. Lopez (llamada)" in enviados[-1][1]


def test_en_modo_ventas_no_hay_recordatorios_programados(modo_ventas, monkeypatch):
    from fastapi.testclient import TestClient

    monkeypatch.delenv("PRODUCCION", raising=False)
    with TestClient(main.app):
        trabajos = {j.func.__module__ for j in main.scheduler.get_jobs()}
    main.scheduler.remove_all_jobs()
    assert trabajos == set()


def test_enviar_botones_formato(monkeypatch):
    monkeypatch.setenv("WA_PHONE_NUMBER_ID", "1")
    monkeypatch.setenv("WA_TOKEN", "t")
    enviado = {}

    class R:
        def raise_for_status(self):
            pass

    monkeypatch.setattr(whatsapp.httpx, "post", lambda url, headers, json, timeout: enviado.update(json) or R())
    whatsapp.enviar_botones("521", "¿Confirmas?", ["Confirmo", "Un titulo demasiado largo para WhatsApp", "C", "D"])
    bts = enviado["interactive"]["action"]["buttons"]
    assert len(bts) == 3 and len(bts[1]["reply"]["title"]) == 20 and enviado["type"] == "interactive"
