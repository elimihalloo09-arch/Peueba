"""Base de datos SQLite: pacientes, mensajes y citas."""
import os
import sqlite3
from contextlib import contextmanager

DB_PATH = os.getenv("DB_PATH", "citasegura.db")


@contextmanager
def conn():
    c = sqlite3.connect(DB_PATH)
    c.row_factory = sqlite3.Row
    try:
        yield c
        c.commit()
    finally:
        c.close()


def init():
    with conn() as c:
        c.executescript("""
        CREATE TABLE IF NOT EXISTS mensajes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            telefono TEXT NOT NULL,
            rol TEXT NOT NULL,              -- 'user' o 'assistant'
            texto TEXT NOT NULL,
            creado TEXT DEFAULT (datetime('now','localtime'))
        );
        CREATE TABLE IF NOT EXISTS citas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            telefono TEXT NOT NULL,
            nombre TEXT NOT NULL,
            motivo TEXT,
            inicio TEXT NOT NULL,           -- 'YYYY-MM-DD HH:MM'
            estado TEXT DEFAULT 'agendada', -- agendada, confirmada, cancelada
            recordatorio_48h INTEGER DEFAULT 0,
            recordatorio_2h INTEGER DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS humano (
            telefono TEXT PRIMARY KEY,      -- conversaciones que atiende una persona
            desde TEXT DEFAULT (datetime('now','localtime'))
        );
        CREATE TABLE IF NOT EXISTS procesados (
            wamid TEXT PRIMARY KEY          -- evita contestar dos veces el mismo mensaje
        );
        """)


def ya_procesado(wamid: str) -> bool:
    with conn() as c:
        try:
            c.execute("INSERT INTO procesados (wamid) VALUES (?)", (wamid,))
            return False
        except sqlite3.IntegrityError:
            return True


def guardar_mensaje(telefono, rol, texto):
    with conn() as c:
        c.execute("INSERT INTO mensajes (telefono, rol, texto) VALUES (?,?,?)", (telefono, rol, texto))


def historial(telefono, limite=20):
    with conn() as c:
        filas = c.execute(
            "SELECT rol, texto FROM mensajes WHERE telefono=? ORDER BY id DESC LIMIT ?", (telefono, limite)
        ).fetchall()
    msgs = [{"role": f["rol"], "content": f["texto"]} for f in reversed(filas)]
    while msgs and msgs[0]["role"] != "user":  # Claude requiere empezar con 'user'
        msgs.pop(0)
    return msgs


def en_modo_humano(telefono) -> bool:
    with conn() as c:
        return c.execute("SELECT 1 FROM humano WHERE telefono=?", (telefono,)).fetchone() is not None


def activar_humano(telefono):
    with conn() as c:
        c.execute("INSERT OR IGNORE INTO humano (telefono) VALUES (?)", (telefono,))
