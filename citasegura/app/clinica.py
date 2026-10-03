"""Datos de la clinica. En la demo estan aqui; con clientes reales viven en la base de datos."""
import os

NOMBRE = os.getenv("CLINICA_NOMBRE", "Clinica Dental Sonrisa")
DIRECCION = os.getenv("CLINICA_DIRECCION", "Av. Ejemplo 123, Col. Benito Juarez, Nezahualcoyotl")
MAPS = os.getenv("CLINICA_MAPS", "https://maps.google.com/?q=Nezahualcoyotl")
WAZE = os.getenv("CLINICA_WAZE", "")
# Indicaciones para llegar (estacionamiento, piso, consultorio...). Texto libre de varias lineas
# en un archivo para que el consultorio lo escriba a su manera. Ver llegada.ejemplo.txt
LLEGADA_ARCHIVO = os.getenv("CLINICA_LLEGADA_ARCHIVO", "llegada.txt")


def como_llegar() -> str:
    """Mensaje completo de como llegar. Se lee cada vez: editar el archivo no requiere reiniciar."""
    try:
        with open(LLEGADA_ARCHIVO, encoding="utf-8") as f:
            indicaciones = f.read().strip()
    except OSError:
        indicaciones = ""
    lineas = [f"📍 *Como llegar a {NOMBRE}*", DIRECCION]
    if indicaciones:
        lineas += ["", indicaciones]
    lineas += ["", f"🗺️ Google Maps: {MAPS}"]
    if WAZE:
        lineas.append(f"🚗 Waze: {WAZE}")
    return "\n".join(lineas)


INFO = f"""
Clinica: {NOMBRE}, Nezahualcoyotl, Estado de Mexico.
Direccion: {DIRECCION} (Google Maps: {MAPS})
Horario: lunes a viernes 10:00-19:00, sabado 10:00-14:00, domingo cerrado.
Duracion de cita: 1 hora.
Precios aproximados: consulta/valoracion $350, limpieza $600, resina desde $700,
extraccion simple desde $800. Precio final despues de la valoracion.
Pagos: efectivo, tarjeta y transferencia.
"""

# Horario de atencion por dia de la semana (0=lunes ... 6=domingo): (hora_inicio, hora_fin)
HORARIO = {0: (10, 19), 1: (10, 19), 2: (10, 19), 3: (10, 19), 4: (10, 19), 5: (10, 14)}

SYSTEM_PROMPT = f"""Eres la recepcionista virtual de {NOMBRE}.
Hablas en espanol mexicano, amable y breve (maximo 3 lineas por mensaje).
Tu objetivo: agendar, confirmar, reprogramar o cancelar citas y resolver dudas basicas.
Usa SIEMPRE las herramientas para ver horarios y agendar; nunca inventes horarios.
Antes de agendar pide: nombre completo y motivo de la cita.
Para reprogramar: usa mis_citas, ofrece horarios con ver_horarios y luego reprogramar_cita.
Si el paciente confirma que asistira, usa confirmar_cita.
Nunca des diagnosticos ni recomendaciones medicas.
Si el paciente menciona dolor fuerte, sangrado, hinchazon, urgencia o esta molesto,
usa la herramienta pasar_a_humano.
Si no sabes algo, dilo y ofrece que la clinica le confirme.
Solo hablas de temas de la clinica: citas, horarios, precios, ubicacion, pagos y dudas del
consultorio. Si te piden otra cosa (tareas, recetas, noticias, platicar de otros temas, escribir
textos, etc.), di amablemente que solo puedes ayudar con citas y dudas de la clinica, y ofrece agendar.

Informacion de la clinica:
{INFO}
"""
