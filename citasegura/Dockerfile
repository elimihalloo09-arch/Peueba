FROM python:3.12-slim

# Hora de Mexico para agenda y recordatorios (la imagen ya trae las zonas horarias)
ENV TZ=America/Mexico_City \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    DB_PATH=/datos/citasegura.db \
    RESPALDOS=/datos/respaldos \
    CLINICA_LLEGADA_ARCHIVO=/datos/llegada.txt \
    PRODUCCION=1

WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY app ./app

# Sin permisos de administrador dentro del contenedor
RUN useradd --create-home --uid 1000 bot && mkdir -p /datos && chown bot /datos
USER bot

EXPOSE 8000
HEALTHCHECK --interval=1m --timeout=5s \
  CMD python -c "import urllib.request; urllib.request.urlopen('http://localhost:8000/salud')"

# Un solo proceso: con mas de uno, cada uno mandaria sus propios recordatorios
CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000", "--workers", "1", "--no-access-log"]
