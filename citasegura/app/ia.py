"""Cerebro del bot: Claude con herramientas de agenda."""
import json
import os
from datetime import datetime

from anthropic import Anthropic

from . import agenda, db, espera, ventas
from .clinica import SYSTEM_PROMPT, como_llegar

_client = None
MODELO = os.getenv("CLAUDE_MODEL", "claude-haiku-4-5-20251001")

TOOLS = [
    {
        "name": "ver_horarios",
        "description": "Devuelve las horas libres de una fecha.",
        "input_schema": {
            "type": "object",
            "properties": {"fecha": {"type": "string", "description": "YYYY-MM-DD"}},
            "required": ["fecha"],
        },
    },
    {
        "name": "agendar_cita",
        "description": "Agenda una cita. Usala solo cuando el paciente ya eligio horario y dio su nombre.",
        "input_schema": {
            "type": "object",
            "properties": {
                "nombre": {"type": "string"},
                "motivo": {"type": "string"},
                "fecha": {"type": "string", "description": "YYYY-MM-DD"},
                "hora": {"type": "string", "description": "HH:MM"},
            },
            "required": ["nombre", "motivo", "fecha", "hora"],
        },
    },
    {
        "name": "mis_citas",
        "description": "Lista las proximas citas del paciente (con su id).",
        "input_schema": {"type": "object", "properties": {}},
    },
    {
        "name": "confirmar_cita",
        "description": "Marca como confirmada una cita del paciente por id (cuando dice que si asistira).",
        "input_schema": {
            "type": "object",
            "properties": {"cita_id": {"type": "integer"}},
            "required": ["cita_id"],
        },
    },
    {
        "name": "reprogramar_cita",
        "description": "Cambia una cita del paciente a otra fecha y hora libre. Libera el horario anterior. "
        "Usala solo cuando el paciente ya eligio el nuevo horario de ver_horarios.",
        "input_schema": {
            "type": "object",
            "properties": {
                "cita_id": {"type": "integer"},
                "fecha": {"type": "string", "description": "YYYY-MM-DD"},
                "hora": {"type": "string", "description": "HH:MM"},
            },
            "required": ["cita_id", "fecha", "hora"],
        },
    },
    {
        "name": "cancelar_cita",
        "description": "Cancela una cita del paciente por id.",
        "input_schema": {
            "type": "object",
            "properties": {"cita_id": {"type": "integer"}},
            "required": ["cita_id"],
        },
    },
    {
        "name": "lista_de_espera",
        "description": "Anota al paciente en la lista de espera de una fecha que ya no tiene horarios libres. "
        "Si alguien cancela ese dia, se le ofrece el lugar por WhatsApp. Pide antes nombre y motivo.",
        "input_schema": {
            "type": "object",
            "properties": {
                "nombre": {"type": "string"},
                "motivo": {"type": "string"},
                "fecha": {"type": "string", "description": "YYYY-MM-DD"},
            },
            "required": ["nombre", "motivo", "fecha"],
        },
    },
    {
        "name": "pasar_a_humano",
        "description": "Pasa la conversacion a una persona de la clinica (urgencias, dolor, quejas).",
        "input_schema": {"type": "object", "properties": {"motivo": {"type": "string"}}},
    },
]


def _cliente():
    global _client
    if _client is None:
        _client = Anthropic()  # lee ANTHROPIC_API_KEY del entorno
    return _client


def _ejecutar(nombre_tool, args, telefono) -> str:
    """Corre la herramienta; si Claude manda datos mal formados se lo decimos para que corrija."""
    try:
        return _correr_herramienta(nombre_tool, args, telefono)
    except (KeyError, ValueError, TypeError) as e:
        return f"Datos invalidos ({e}). Revisa el formato: fecha YYYY-MM-DD, hora HH:MM."


def _correr_herramienta(nombre_tool, args, telefono) -> str:
    if nombre_tool in ventas.NOMBRES:
        return ventas.ejecutar(nombre_tool, args, telefono)
    if nombre_tool == "ver_horarios":
        libres = agenda.horarios_libres(args["fecha"])
        return json.dumps(libres) if libres else "Sin horarios libres ese dia."
    if nombre_tool == "agendar_cita":
        return agenda.agendar(telefono, args["nombre"], args["motivo"], args["fecha"], args["hora"])
    if nombre_tool == "mis_citas":
        return json.dumps(agenda.citas_del_paciente(telefono), ensure_ascii=False)
    if nombre_tool == "confirmar_cita":
        return agenda.cambiar_estado(telefono, args["cita_id"], "confirmada")
    if nombre_tool == "reprogramar_cita":
        return agenda.reprogramar(telefono, args["cita_id"], args["fecha"], args["hora"])
    if nombre_tool == "cancelar_cita":
        return agenda.cambiar_estado(telefono, args["cita_id"], "cancelada")
    if nombre_tool == "lista_de_espera":
        return espera.anotar(telefono, args["nombre"], args["motivo"], args["fecha"])
    if nombre_tool == "pasar_a_humano":
        db.activar_humano(telefono)
        return "Conversacion marcada para atencion humana."
    return "Herramienta desconocida."


def responder(telefono: str) -> str:
    ahora = datetime.now().strftime("%A %Y-%m-%d %H:%M")
    if ventas.activo():  # otro numero: vende CitaSegura a dentistas, con la demo dentro de la platica
        system = ventas.system_prompt() + f"\n\nFecha y hora actual: {ahora}."
        tools = [t for t in TOOLS if t["name"] != "lista_de_espera"] + ventas.TOOLS
    else:
        system = SYSTEM_PROMPT + f"\nComo llegar (usalo si preguntan):\n{como_llegar()}\n\nFecha y hora actual: {ahora}."
        tools = TOOLS
    mensajes = db.historial(telefono)

    for _ in range(5):  # maximo 5 vueltas de herramientas
        r = _cliente().messages.create(
            model=MODELO, max_tokens=500, system=system, tools=tools, messages=mensajes
        )
        if r.stop_reason != "tool_use":
            texto = "".join(b.text for b in r.content if b.type == "text").strip()
            # WhatsApp rechaza mensajes vacios (pasa a veces despues de una herramienta)
            return texto or "Listo. Si necesitas algo mas, aqui estoy."

        mensajes.append({"role": "assistant", "content": r.content})
        resultados = []
        for b in r.content:
            if b.type == "tool_use":
                resultados.append(
                    {"type": "tool_result", "tool_use_id": b.id, "content": _ejecutar(b.name, b.input, telefono)}
                )
        mensajes.append({"role": "user", "content": resultados})

    return "Permiteme un momento, te confirma una persona de la clinica."
