"""Cerebro del bot: Claude con herramientas de agenda."""
import json
import os
from datetime import datetime

from anthropic import Anthropic

from . import agenda, db
from .clinica import SYSTEM_PROMPT

client = Anthropic()  # lee ANTHROPIC_API_KEY del entorno
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
        "name": "cancelar_cita",
        "description": "Cancela una cita del paciente por id.",
        "input_schema": {
            "type": "object",
            "properties": {"cita_id": {"type": "integer"}},
            "required": ["cita_id"],
        },
    },
    {
        "name": "pasar_a_humano",
        "description": "Pasa la conversacion a una persona de la clinica (urgencias, dolor, quejas).",
        "input_schema": {"type": "object", "properties": {"motivo": {"type": "string"}}},
    },
]


def _ejecutar(nombre_tool, args, telefono) -> str:
    if nombre_tool == "ver_horarios":
        libres = agenda.horarios_libres(args["fecha"])
        return json.dumps(libres) if libres else "Sin horarios libres ese dia."
    if nombre_tool == "agendar_cita":
        return agenda.agendar(telefono, args["nombre"], args["motivo"], args["fecha"], args["hora"])
    if nombre_tool == "mis_citas":
        return json.dumps(agenda.citas_del_paciente(telefono), ensure_ascii=False)
    if nombre_tool == "cancelar_cita":
        return agenda.cambiar_estado(telefono, args["cita_id"], "cancelada")
    if nombre_tool == "pasar_a_humano":
        db.activar_humano(telefono)
        return "Conversacion marcada para atencion humana."
    return "Herramienta desconocida."


def responder(telefono: str) -> str:
    ahora = datetime.now().strftime("%A %Y-%m-%d %H:%M")
    system = SYSTEM_PROMPT + f"\nFecha y hora actual: {ahora}."
    mensajes = db.historial(telefono)

    for _ in range(5):  # maximo 5 vueltas de herramientas
        r = client.messages.create(
            model=MODELO, max_tokens=500, system=system, tools=TOOLS, messages=mensajes
        )
        if r.stop_reason != "tool_use":
            return "".join(b.text for b in r.content if b.type == "text").strip()

        mensajes.append({"role": "assistant", "content": r.content})
        resultados = []
        for b in r.content:
            if b.type == "tool_use":
                resultados.append(
                    {"type": "tool_result", "tool_use_id": b.id, "content": _ejecutar(b.name, b.input, telefono)}
                )
        mensajes.append({"role": "user", "content": resultados})

    return "Permiteme un momento, te confirma una persona de la clinica."
