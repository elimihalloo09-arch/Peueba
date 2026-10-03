# CONTEXTO DEL PROYECTO – CitaSegura

> Lee este archivo completo antes de escribir o cambiar codigo.
> Idioma de trabajo: espanol (codigo, comentarios, mensajes al usuario y commits).

## 1. Que es CitaSegura
Servicio de asistentes de WhatsApp con IA para negocios que trabajan con citas,
empezando por **consultorios dentales en Nezahualcoyotl, Estado de Mexico**.
No es una app para pacientes: el paciente usa el WhatsApp que ya tiene.

Modelo de negocio: servicio "hecho para ti" con instalacion y soporte en persona.
- Instalacion unica + mensualidad (ingreso recurrente es la meta principal).
- Diferenciador vs plataformas (Doctoralia Pro, Doctocliq, Kosmo): pago mensual sin
  contrato anual, funciona en el WhatsApp propio del consultorio, configuracion a la
  medida y soporte local en persona.

## 2. Objetivos
1. **Corto plazo (semanas 1-3):** demo funcional para ensenar a dentistas en su celular.
2. **Mes 1-2:** 2-3 consultorios piloto pagando (aunque sea con descuento) + casos de exito medibles.
3. **Mes 4-6:** 10 clientes con mensualidad.
4. **Despues:** panel web para el doctor y multi-cliente (un solo servidor, muchos consultorios).

Metrica que vende: **reduccion de inasistencias** (en odontologia faltan 25-35% de las citas;
gran parte por olvido). Todo lo que construyamos debe ayudar a medir y mostrar ese resultado.

## 3. Nichos (en orden)
1. **Consultorios dentales** – foco actual. Citas, recordatorios, confirmaciones.
2. **Restaurantes** – siguiente. Pedidos directos por WhatsApp sin comision de apps
   (Rappi/Uber Eats/DiDi cobran 30-35% efectivo). Requiere modulo de menu y pedidos.
3. **Veterinarias** – citas + recordatorios recurrentes (vacunas, banos, desparasitacion).
4. Barberias con cita – plan B. Autolavado – descartado por ahora.

Regla de diseno: el **nucleo de citas** debe ser reutilizable entre nichos. Lo especifico
de cada negocio (textos, servicios, precios, horarios) va en configuracion, no en el codigo.

## 4. Alcance del MVP dental (lo unico que se construye ahora)
1. Agendar cita: preguntar motivo y nombre, ofrecer horarios libres, confirmar.
2. Recordatorio 48 h antes con botones: Confirmo / Reprogramar / Cancelar.
3. Recordatorio 2 h antes con direccion y link de Maps.
4. Reprogramar o cancelar sin intervencion humana; liberar el horario.
5. Responder dudas basicas: precios aproximados, horarios, ubicacion, formas de pago.
6. Pasar a humano: dolor fuerte, sangrado, hinchazon, urgencia, enojo o quejas.

**Fuera de alcance por ahora:** app movil, pagos en linea, expediente clinico, facturacion,
multi-sucursal, panel web (llega despues), campanas de marketing masivas.

## 5. Stack y arquitectura
- Python 3.12, FastAPI, Uvicorn.
- IA: API de Claude (SDK `anthropic`), modelo por defecto `claude-haiku-4-5-20251001`
  (configurable con `CLAUDE_MODEL`). Uso de herramientas (tool use) para la agenda.
- WhatsApp: **Cloud API directa de Meta** (sin BSP intermediario).
- Base de datos: SQLite ahora; PostgreSQL cuando haya 5+ clientes.
- Tareas programadas: APScheduler (revisa recordatorios cada 10 min).
- Desarrollo: compu local + Cloudflare Tunnel. Produccion: VPS Ubuntu barato con Docker o systemd.
- Futuro panel web: TypeScript + React.

```
app/
  main.py          webhook de WhatsApp (GET verificacion, POST mensajes), arranque,
                   botones del recordatorio y comandos del consultorio (#reporte, #hoy,
                   #falta, #asistio, #bot)
  ia.py            Claude + herramientas: ver_horarios, agendar_cita, mis_citas,
                   confirmar_cita, cancelar_cita, reprogramar_cita, pasar_a_humano
  agenda.py        logica de horarios y citas
  recordatorios.py recordatorios 48 h (con botones) y 2 h (direccion y Maps) con plantilla
  metricas.py      calculo de metricas y reporte semanal (lunes 9:00)
  avisos.py        avisos al consultorio: cita nueva, cancelada, movida, urgencia
  whatsapp.py      enviar_texto, enviar_plantilla
  db.py            SQLite: mensajes, citas, humano, procesados (+ migracion de columnas)
  clinica.py       datos de la clinica y SYSTEM_PROMPT  (hoy hardcodeado, demo)
tests/             pruebas con pytest (no usan Claude ni Meta)
probar_chat.py     simulador en terminal: Claude real, WhatsApp falso
Dockerfile, docker-compose.yml, respaldar.sh, DESPLIEGUE.md   produccion en VPS
```

Flujo: Meta -> POST /webhook -> responde 200 rapido -> tarea en segundo plano ->
guarda mensaje -> si esta en modo humano no responde -> Claude con herramientas ->
guarda respuesta -> envia por WhatsApp.

## 6. Reglas y restricciones reales (no romperlas)
- **Ventana de 24 h de WhatsApp:** fuera de ella solo se pueden enviar **plantillas aprobadas**
  por Meta. Los recordatorios SIEMPRE son plantillas de categoria Utilidad.
- **Costos Meta Mexico (oct 2026):** utilidad ~USD 0.0085 por mensaje; marketing ~USD 0.0397;
  mensajes de servicio: primeros 1,000 por numero al mes gratis, despues tarifa de utilidad.
  Evitar mensajes innecesarios y nunca usar plantillas de marketing para recordatorios.
- **Nada de envios masivos ni prospeccion automatizada** a quien no escribio primero.
  Riesgo de bloqueo del numero.
- **Salud:** el bot nunca diagnostica ni recomienda tratamientos. Ante duda medica o urgencia,
  pasar a humano.
- **Privacidad:** datos de pacientes son sensibles. No loguear contenido de mensajes en
  produccion, respaldos cifrados, minimo de datos necesarios (nombre, telefono, cita).
- **Seguridad:** en produccion validar la firma `X-Hub-Signature-256` con `WA_APP_SECRET`.
  Llaves solo en `.env`, nunca en el codigo ni en git.
- **Idempotencia:** Meta puede reenviar el mismo mensaje; ya se controla con la tabla `procesados`.
- **Zona horaria:** America/Mexico_City.

## 7. Tono del bot
Espanol mexicano, amable, breve (max 3 lineas). Como una buena recepcionista de Neza:
calida, clara, sin tecnicismos. Nunca inventar horarios ni precios: usar herramientas y config.

## 8. Estado actual del codigo
Funciona (probado con pytest): horarios libres, agendar (sin empalmes), listar, confirmar,
cancelar, reprogramar, botones del recordatorio, modo humano y comandos del consultorio,
recordatorios sin duplicados, idempotencia, metricas y reporte semanal.
Estados de cita: agendada, confirmada, cancelada, reprogramada (la vieja; la nueva es otra fila).
"Activa" = agendada o confirmada. Faltas: columna `falto`, la marca recepcion con `#falta`.
Produccion: `PRODUCCION=1` (lo pone el Dockerfile) hace que el servidor no arranque si falta
una llave. Un solo worker de uvicorn (con mas, los recordatorios saldrian repetidos).
Pendiente de probar con llaves reales: Claude + Meta.

## 9. Backlog (en orden de prioridad)
1. Conectar llaves reales y probar conversacion completa con el numero de prueba de Meta.
2. ~~Manejar respuestas a botones del recordatorio~~ (hecho: Confirmo/Cancelar directo en
   codigo, Reprogramar pasa a Claude).
3. ~~Herramienta `reprogramar_cita`~~ (hecho).
4. ~~Recordatorio de 2 h con plantilla distinta~~ (hecho en codigo: `WA_TEMPLATE_RECORDATORIO_2H`,
   `CLINICA_DIRECCION`, `CLINICA_MAPS`). Falta darla de alta y que Meta la apruebe (ver README).
5. ~~Notificar al consultorio cada cita nueva y cada cancelacion~~ (hecho: `app/avisos.py`; tambien
   citas movidas y urgencias. Opcional `WA_TEMPLATE_AVISO` para que lleguen fuera de 24 h).
6. ~~Comando para salir de modo humano~~ (hecho: `TELEFONO_HUMANO` manda `#bot <telefono>`).
7. ~~**Metricas para el doctor**~~ (hecho: `#reporte`, `#hoy`, `#falta` y envio los lunes).
   Pendiente: plantilla de Utilidad para el reporte, para que llegue aunque el consultorio
   no haya escrito en 24 h.
8. Mover `clinica.py` a configuracion en BD (multi-cliente: un registro por consultorio,
   enrutado por `phone_number_id` del webhook).
9. Pruebas automaticas (pytest): ya hay base en `tests/`; ampliar con cada cambio.
10. ~~Dockerfile + guia de despliegue en VPS + respaldo diario de la BD~~ (hecho: `Dockerfile`,
    `docker-compose.yml` con Caddy, `DESPLIEGUE.md`, `respaldar.sh` cifrado con gpg, `app/respaldo.py`).
11. Panel web basico del doctor (agenda y metricas).

## 10. Forma de trabajar
- Desarrollador unico, tiempo parcial (tardes y fines de semana): preferir soluciones
  simples, poco codigo y pocas dependencias. Nada de sobreingenieria.
- Cambios pequenos y probados; explicar en espanol que se hizo y como probarlo.
- Antes de agregar una libreria o servicio de pago, justificar y preguntar.
- Si algo de la API de Meta o de Claude puede haber cambiado, verificar en la documentacion oficial.
- Correr pruebas: `pip install -r requirements-dev.txt && pytest`.
