from datetime import datetime, timedelta

import pytest

from app import db, whatsapp


@pytest.fixture(autouse=True)
def bd_temporal(tmp_path, monkeypatch):
    """Cada prueba usa su propia base de datos vacia y nunca le escribe a un telefono real."""
    monkeypatch.setattr(db, "DB_PATH", str(tmp_path / "prueba.db"))
    for var in ("TELEFONO_HUMANO", "WA_TEMPLATE_AVISO", "AVISAR_CONSULTORIO"):
        monkeypatch.delenv(var, raising=False)
    db.init()


@pytest.fixture
def enviados(monkeypatch):
    """Captura lo que se mandaria por WhatsApp en vez de llamar a Meta."""
    salida = []
    monkeypatch.setattr(whatsapp, "enviar_texto", lambda tel, txt: salida.append((tel, txt)))
    monkeypatch.setattr(whatsapp, "enviar_plantilla", lambda tel, pl, params: salida.append((tel, params)))
    return salida


@pytest.fixture
def lunes():
    """Un lunes de dentro de 1-2 semanas, 'YYYY-MM-DD' (dia con horario de 10 a 19)."""
    d = datetime.now() + timedelta(days=7)
    while d.weekday() != 0:
        d += timedelta(days=1)
    return d.strftime("%Y-%m-%d")
