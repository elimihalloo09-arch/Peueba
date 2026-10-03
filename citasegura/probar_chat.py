"""Platica con el bot en la terminal, sin WhatsApp ni Meta. Solo necesita ANTHROPIC_API_KEY.

Uso:  python probar_chat.py
  - Escribe como si fueras el paciente.
  - /boton Confirmo   simula tocar un boton del recordatorio (Confirmo, Cancelar, Reprogramar)
  - /staff #reporte   escribe como el consultorio (#reporte, #hoy, #falta 3, #bot <tel>)
  - /recordatorios    corre la revision de recordatorios ahora mismo
  - /salir
Usa su propia base de datos (prueba_chat.db), no toca la real.
"""
import itertools
import os

from dotenv import load_dotenv

load_dotenv()
os.environ["DB_PATH"] = "prueba_chat.db"
os.environ.setdefault("TELEFONO_HUMANO", "5215500000000")

from app import db, main, recordatorios, whatsapp  # noqa: E402

PACIENTE = "5215511111111"
STAFF = os.environ["TELEFONO_HUMANO"]
ids = itertools.count(1)


def _mostrar(tel, texto):
    quien = "consultorio" if tel == STAFF else "paciente"
    print(f"\n  [bot -> {quien}] {texto}\n")


whatsapp.enviar_texto = _mostrar
whatsapp.enviar_plantilla = lambda tel, plantilla, params: print(f"\n  [plantilla {plantilla} -> {tel}] {params}\n")


def _mensaje(texto, de=PACIENTE, tipo="text"):
    m = {"id": f"local.{os.getpid()}.{next(ids)}", "from": de, "type": tipo}
    m.update({"button": {"text": texto}} if tipo == "button" else {"text": {"body": texto}})
    return m


def main_loop():
    if not os.getenv("ANTHROPIC_API_KEY"):
        print("Falta ANTHROPIC_API_KEY en .env")
        return
    db.init()
    print(__doc__)
    while True:
        try:
            texto = input("tu> ").strip()
        except (EOFError, KeyboardInterrupt):
            break
        if not texto:
            continue
        if texto == "/salir":
            break
        if texto == "/recordatorios":
            recordatorios.revisar()
        elif texto.startswith("/boton "):
            main.procesar(_mensaje(texto[7:], tipo="button"))
        elif texto.startswith("/staff "):
            main.procesar(_mensaje(texto[7:], de=STAFF))
        else:
            main.procesar(_mensaje(texto))


if __name__ == "__main__":
    main_loop()
