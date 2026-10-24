from app import whatsapp


def test_variables_de_plantilla_sin_saltos(monkeypatch):
    monkeypatch.setenv("WA_PHONE_NUMBER_ID", "1")
    monkeypatch.setenv("WA_TOKEN", "x")
    enviado = {}

    class Resp:
        def raise_for_status(self):
            pass

    monkeypatch.setattr(whatsapp.httpx, "post", lambda url, headers, json, timeout: enviado.update(json) or Resp())
    whatsapp.enviar_plantilla("521", "recordatorio_2h", ["Av. Ejemplo 123,\n  Col.\tCentro     Neza"])
    texto = enviado["template"]["components"][0]["parameters"][0]["text"]
    assert texto == "Av. Ejemplo 123, Col. Centro Neza"
