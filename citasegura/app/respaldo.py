"""Respaldo de la base de datos: copia consistente (aunque el bot este funcionando), comprimida.

Uso: python -m app.respaldo [carpeta]   -> imprime la ruta del respaldo creado.
Guarda los ultimos 14. El cifrado y la copia fuera del servidor los hace respaldar.sh.
"""
import gzip
import os
import shutil
import sqlite3
import sys
import tempfile
from datetime import datetime
from pathlib import Path

from . import db

CONSERVAR = 14


def respaldar(carpeta: str) -> Path:
    destino = Path(carpeta)
    destino.mkdir(parents=True, exist_ok=True)
    os.chmod(destino, 0o700)
    final = destino / f"citasegura-{datetime.now():%Y%m%d-%H%M}.db.gz"
    with tempfile.TemporaryDirectory() as tmp:
        copia = Path(tmp) / "copia.db"
        origen = sqlite3.connect(db.DB_PATH)
        dst = sqlite3.connect(copia)
        try:
            origen.backup(dst)  # API de SQLite: copia segura aunque haya escrituras en curso
        finally:
            dst.close()
            origen.close()
        with open(copia, "rb") as f, gzip.open(final, "wb") as g:
            shutil.copyfileobj(f, g)
    os.chmod(final, 0o600)  # datos de pacientes: solo el dueno del archivo
    for viejo in sorted(destino.glob("citasegura-*.db.gz"))[:-CONSERVAR]:
        viejo.unlink()
    return final


if __name__ == "__main__":
    print(respaldar(sys.argv[1] if len(sys.argv) > 1 else os.getenv("RESPALDOS", "respaldos")))
