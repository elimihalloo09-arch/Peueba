"""Base de datos SQLite: pacientes, mensajes y citas."""
import os
import sqlite3
from contextlib import contextmanager
from datetime import datetime, timedelta

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
            estado TEXT DEFAULT 'agendada', -- agendada, confirmada, cancelada, reprogramada
            recordatorio_48h INTEGER DEFAULT 0,
            recordatorio_2h INTEGER DEFAULT 0,
            creado TEXT,                    -- 'YYYY-MM-DD HH:MM' en que el bot la agendo
            falto INTEGER DEFAULT 0         -- 1 si recepcion marco que el paciente no llego
        );
        CREATE TABLE IF NOT EXISTS humano (
            telefono TEXT PRIMARY KEY,      -- conversaciones que atiende una persona
            desde TEXT DEFAULT (datetime('now','localtime'))
        );
        CREATE TABLE IF NOT EXISTS espera (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            telefono TEXT NOT NULL,
            nombre TEXT NOT NULL,
            motivo TEXT,
            fecha TEXT NOT NULL,            -- 'YYYY-MM-DD' que queria el paciente
            estado TEXT DEFAULT 'esperando', -- esperando, ofrecida, tomada, resuelta, rechazada, vencida
            ofrecido TEXT,                  -- horario ofrecido 'YYYY-MM-DD HH:MM'
            ofrecido_en TEXT,
            tomada_en TEXT,
            creado TEXT DEFAULT (datetime('now','localtime'))
        );
        CREATE TABLE IF NOT EXISTS regreso_envios (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            telefono TEXT NOT NULL,
            cita_id INTEGER,
            enviado_en TEXT NOT NULL        -- 'YYYY-MM-DD HH:MM'
        );
        CREATE TABLE IF NOT EXISTS no_regreso (
            telefono TEXT PRIMARY KEY,      -- pidio no recibir recordatorios de regreso
            desde TEXT DEFAULT (datetime('now','localtime'))
        );
        CREATE TABLE IF NOT EXISTS interesados ( -- modo ventas: dentistas que quieren el piloto
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            telefono TEXT NOT NULL,
            nombre TEXT,
            consultorio TEXT,
            zona TEXT,
            faltas_semana REAL,
            precio_consulta REAL,
            whatsapp_business TEXT,
            interes TEXT,
            notas TEXT,
            creado TEXT DEFAULT (datetime('now','localtime'))
        );
        CREATE TABLE IF NOT EXISTS procesados (
            wamid TEXT PRIMARY KEY          -- evita contestar dos veces el mismo mensaje
        );
        -- una hora solo puede tener una cita activa (evita empalmes si dos agendan a la vez)
        DROP INDEX IF EXISTS una_cita_por_hora;
        CREATE UNIQUE INDEX IF NOT EXISTS una_cita_activa_por_hora ON citas(inicio)
            WHERE estado IN ('agendada','confirmada');
        """)
        # bases creadas antes de las metricas no tienen estas columnas
        columnas = {f["name"] for f in c.execute("PRAGMA table_info(citas)")}
        if "creado" not in columnas:
            c.execute("ALTER TABLE citas ADD COLUMN creado TEXT")
        if "falto" not in columnas:
            c.execute("ALTER TABLE citas ADD COLUMN falto INTEGER DEFAULT 0")
        if "regreso" not in columnas:  # 1 = ya se mando (o ya no toca) el recordatorio de regreso
            c.execute("ALTER TABLE citas ADD COLUMN regreso INTEGER DEFAULT 0")
    os.chmod(DB_PATH, 0o600)  # datos de pacientes: solo el usuario del bot puede leerlos


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


def _horas_pausa() -> float:
    """Horas sin actividad de la recepcion tras las que el bot vuelve solo (0 = nunca)."""
    try:
        return float(os.getenv("PAUSA_HUMANO_HORAS", "12"))
    except ValueError:
        return 12.0


def en_modo_humano(telefono) -> bool:
    with conn() as c:
        fila = c.execute("SELECT desde FROM humano WHERE telefono=?", (telefono,)).fetchone()
        if fila is None:
            return False
        horas = _horas_pausa()
        if horas and fila["desde"] < (datetime.now() - timedelta(hours=horas)).strftime("%Y-%m-%d %H:%M:%S"):
            c.execute("DELETE FROM humano WHERE telefono=?", (telefono,))  # nadie la atendio: vuelve el bot
            return False
        return True


def activar_humano(telefono):
    """Pausa al bot en esta conversacion. Si ya estaba pausada, reinicia el conteo de horas."""
    with conn() as c:
        c.execute(
            "INSERT OR REPLACE INTO humano (telefono, desde) VALUES (?, ?)",
            (telefono, datetime.now().strftime("%Y-%m-%d %H:%M:%S")),
        )


def desactivar_humano(telefono) -> bool:
    """Regresa la conversacion al bot. True si estaba en modo humano."""
    with conn() as c:
        return c.execute("DELETE FROM humano WHERE telefono=?", (telefono,)).rowcount > 0
