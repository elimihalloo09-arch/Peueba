from pathlib import Path

import pytest

from app import espera, ia, main, voz, whatsapp

TEL = "5215511111111"
OGG = Path(__file__).parent / "nota_de_voz.ogg"  # 3 s de tono en OGG/Opus, el formato de WhatsApp
_n = 0


def _audio():
    global _n
    _n += 1
    return {"id": f"wamid.voz{_n}", "from": TEL, "type": "audio", "audio": {"id": "media123", "mime_type": "audio/ogg; codecs=opus"}}


@pytest.fixture
def voz_activa(monkeypatch):
    monkeypatch.setenv("VOZ", "1")
    monkeypatch.setattr(whatsapp, "descargar_media", lambda media_id: OGG.read_bytes())


@pytest.fixture
def claude(monkeypatch):
    vio = []

    def responder(tel):
        from app import db

        vio.append(db.historial(tel)[-1]["content"])
        return "ok"

    monkeypatch.setattr(ia, "responder", responder)
    return vio


def test_nota_de_voz_llega_a_claude_como_texto(enviados, voz_activa, claude, monkeypatch):
    monkeypatch.setattr(voz, "transcribir", lambda audio: "quiero una cita para el lunes")
    main.procesar(_audio())
    assert claude == ["🎤 quiero una cita para el lunes"]


def test_si_no_se_entiende_pide_que_lo_escriba(enviados, voz_activa, claude, monkeypatch):
    monkeypatch.setattr(voz, "transcribir", lambda audio: None)
    main.procesar(_audio())
    assert claude == [] and "No alcance a escuchar" in enviados[-1][1]


def test_si_falla_la_descarga(enviados, voz_activa, claude, monkeypatch):
    def falla(media_id):
        raise RuntimeError("Meta caido")

    monkeypatch.setattr(whatsapp, "descargar_media", falla)
    main.procesar(_audio())
    assert "No alcance a escuchar" in enviados[-1][1]


def test_sin_voz_activada(enviados, claude, monkeypatch):
    monkeypatch.delenv("VOZ", raising=False)
    main.procesar(_audio())
    assert "solo puedo leer mensajes de texto" in enviados[-1][1]


def test_lo_quiero_en_audio_acepta_la_oferta(enviados, voz_activa, monkeypatch):
    monkeypatch.setattr(voz, "transcribir", lambda audio: "Lo quiero.")
    vistos = []
    monkeypatch.setattr(espera, "responder", lambda tel, acepta: vistos.append(acepta) or ["listo"])
    main.procesar(_audio())
    assert vistos == [True]


class _Seg:
    def __init__(self, t):
        self.text = t


class _Info:
    def __init__(self, d):
        self.duration = d


class _ModeloFalso:
    def __init__(self, duracion=5.0, textos=(" Hola, ", "quiero cita. ")):
        self.duracion, self.textos, self.args = duracion, textos, None

    def transcribe(self, audio, **kw):
        self.args = kw
        return iter(_Seg(t) for t in self.textos), _Info(self.duracion)


def test_transcribir_une_y_limpia(monkeypatch):
    m = _ModeloFalso()
    monkeypatch.setattr(voz, "_cargar", lambda: m)
    assert voz.transcribir(b"x") == "Hola, quiero cita."
    assert m.args["language"] == "es"


def test_audio_demasiado_largo(monkeypatch):
    monkeypatch.setattr(voz, "_cargar", lambda: _ModeloFalso(duracion=300))
    assert voz.transcribir(b"x") is None


def test_error_del_modelo_no_truena(monkeypatch):
    def roto():
        raise RuntimeError("sin memoria")

    monkeypatch.setattr(voz, "_cargar", roto)
    assert voz.transcribir(b"x") is None


def test_audio_de_whatsapp_se_puede_leer():
    """El formato real de las notas de voz (OGG/Opus) se decodifica sin programas extra."""
    fw = pytest.importorskip("faster_whisper")
    import io

    muestras = fw.decode_audio(io.BytesIO(OGG.read_bytes()), sampling_rate=16000)
    assert 2.5 < len(muestras) / 16000 < 3.5


def test_descargar_media(monkeypatch):
    monkeypatch.setenv("WA_TOKEN", "t")
    llamadas = []

    class R:
        def __init__(self, data=None, content=b""):
            self._d, self.content = data, content

        def raise_for_status(self):
            pass

        def json(self):
            return self._d

    def get(url, headers, timeout):
        llamadas.append((url, headers["Authorization"]))
        if "media123" in url:
            return R({"url": "https://lookaside.fbsbx.com/audio", "file_size": 1000})
        return R(content=b"OGG")

    monkeypatch.setattr(whatsapp.httpx, "get", get)
    assert whatsapp.descargar_media("media123") == b"OGG"
    assert all(auth == "Bearer t" for _, auth in llamadas) and len(llamadas) == 2


def test_descargar_media_rechaza_archivos_enormes(monkeypatch):
    monkeypatch.setenv("WA_TOKEN", "t")

    class R:
        def raise_for_status(self):
            pass

        def json(self):
            return {"url": "u", "file_size": 50_000_000}

    monkeypatch.setattr(whatsapp.httpx, "get", lambda *a, **k: R())
    with pytest.raises(ValueError):
        whatsapp.descargar_media("x")
