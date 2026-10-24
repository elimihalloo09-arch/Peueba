import gzip
import sqlite3

import pytest

from app import agenda, main, respaldo


def test_produccion_exige_llaves(monkeypatch):
    monkeypatch.setenv("PRODUCCION", "1")
    for v in main.OBLIGATORIAS:
        monkeypatch.setenv(v, "algo")
    monkeypatch.setenv("WA_APP_SECRET", "pega_aqui_el_app_secret")  # valor de ejemplo sin cambiar
    with pytest.raises(RuntimeError, match="WA_APP_SECRET"):
        main.revisar_configuracion()
    monkeypatch.setenv("WA_APP_SECRET", "secreto-real")
    main.revisar_configuracion()


def test_en_desarrollo_no_exige(monkeypatch):
    monkeypatch.delenv("PRODUCCION", raising=False)
    monkeypatch.delenv("WA_APP_SECRET", raising=False)
    main.revisar_configuracion()


def test_respaldo_se_puede_restaurar(tmp_path, lunes):
    agenda.agendar("521551", "Ana", "limpieza", lunes, "10:00")
    archivo = respaldo.respaldar(tmp_path / "resp")
    restaurada = tmp_path / "restaurada.db"
    with gzip.open(archivo) as g:
        restaurada.write_bytes(g.read())
    c = sqlite3.connect(restaurada)
    assert c.execute("SELECT nombre FROM citas").fetchone()[0] == "Ana"
    c.close()
    assert oct(archivo.stat().st_mode)[-3:] == "600"


def test_respaldo_conserva_solo_los_ultimos(tmp_path):
    carpeta = tmp_path / "resp"
    carpeta.mkdir()
    for i in range(20):
        (carpeta / f"citasegura-202601{i:02d}-0300.db.gz").write_bytes(b"x")
    respaldo.respaldar(carpeta)
    assert len(list(carpeta.glob("citasegura-*.db.gz"))) == respaldo.CONSERVAR


def test_archivos_privados(tmp_path):
    from app import db

    assert oct((tmp_path / "prueba.db").stat().st_mode)[-3:] == "600"
    respaldo.respaldar(tmp_path / "resp")
    assert oct((tmp_path / "resp").stat().st_mode)[-3:] == "700"
