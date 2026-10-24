"""Notas de voz -> texto, en el propio servidor con faster-whisper (gratis, codigo abierto, MIT).

El audio nunca sale del servidor ni se guarda en disco: se transcribe en memoria y se descarta.
Se activa con VOZ=1 e instalando requirements-voz.txt. El modelo (~250 MB con "small") se
descarga la primera vez y queda en VOZ_CARPETA.
"""
import io
import logging
import os
import threading

log = logging.getLogger("voz")
_modelo = None
_candado = threading.Lock()  # el modelo se carga una sola vez aunque lleguen dos audios juntos


def activa() -> bool:
    return os.getenv("VOZ") == "1"


def _cargar():
    global _modelo
    with _candado:
        if _modelo is None:
            from faster_whisper import WhisperModel  # solo se importa si VOZ=1

            nombre = os.getenv("VOZ_MODELO", "small")  # base: mas rapido y ligero; small: mas preciso
            _modelo = WhisperModel(
                nombre, device="cpu", compute_type="int8", download_root=os.getenv("VOZ_CARPETA") or None
            )
            log.info("Modelo de voz '%s' listo", nombre)
    return _modelo


def transcribir(audio: bytes) -> str | None:
    """Texto de la nota de voz, o None si no se entendio o es demasiado larga."""
    max_seg = float(os.getenv("VOZ_MAX_SEGUNDOS", "120"))
    try:
        segmentos, info = _cargar().transcribe(
            io.BytesIO(audio), language="es", vad_filter=True, beam_size=1,
            initial_prompt="Mensaje de un paciente a un consultorio dental para pedir o cambiar una cita.",
        )
        if info.duration > max_seg:
            return None
        texto = " ".join(s.text.strip() for s in segmentos).strip()
    except Exception as e:
        log.error("No se pudo transcribir una nota de voz: %s", e)  # sin el contenido: privacidad
        return None
    return texto or None


if __name__ == "__main__":
    # Prueba rapida con una nota de voz guardada desde WhatsApp:  python -m app.voz audio.ogg
    import sys
    import time

    inicio = time.time()
    with open(sys.argv[1], "rb") as f:
        resultado = transcribir(f.read())
    print(resultado or "(no se entendio)")
    print(f"[{time.time() - inicio:.1f} s, incluye cargar el modelo la primera vez]")
