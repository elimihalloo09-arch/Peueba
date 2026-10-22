# Avance de proyectos

Junta los avances que manda cada equipo —en texto, como llegan por correo o
chat— y genera la presentación institucional del corte en **PDF** y en
**PowerPoint** editable, sin acomodar nada a mano.

**Rust + Axum + SQLx** en el backend, **Angular 18** en el frontend,
**PostgreSQL** de base. Mismo stack y mismas convenciones que `../app`.

## Cómo se usa

1. **Cortes → Nuevo corte.** Pon la fecha del corte y la del siguiente. Si
   marcas *traer lo que quedó en proceso o pendiente*, el corte arranca con
   esos puntos del corte anterior y solo los actualizas.
2. **Pega el texto del equipo** en *Sumar avances* y dale *Leer texto*.
3. **Revisa** lo que entendió: proyecto, equipo, cada punto con su estado.
   Lo que dice *adivinado* no traía estado y lo dedujo de las palabras.
4. **Sumar al corte.** Repite con el texto de cada equipo: se va sumando.
   Si un punto ya estaba (por ejemplo, arrastrado), no se duplica: se
   actualiza su estado.
5. **PDF** o **PowerPoint.** *Vista previa* lo muestra ahí mismo.

## Qué texto entiende

No hace falta un formato fijo. Todo esto funciona, mezclado:

```
PROYECTO: PORTAL DE TRÁMITES
Equipo jurídico
• Continúan las pruebas del checklist. Estado: En proceso.

Mesa de ayuda - Equipo de soporte
- Se liberó la base de conocimiento (concluido)
- Falta la capacitación a usuarios
1. QA aprobó la versión
Estado: concluido

Próximos pasos:
- Mesa de ayuda: agendar la capacitación
```

- **Proyecto**: `PROYECTO: X`, `X:` solo en su renglón, un renglón corto
  seguido de viñetas, o `X: avance` cuando X ya está en el catálogo.
  Reconoce los del catálogo aunque vengan en mayúsculas, sin acentos o solo
  con la primera palabra ("Asistente" → "Asistente virtual (Fase 2)").
- **Equipo**: renglón que empieza con *Equipo*, o `Proyecto - Equipo`.
- **Estado**: `Estado: X`, `(X)`, `[X]` o `- X` al final, o en el renglón de
  abajo. Concluido, en proceso o pendiente, con sus sinónimos (terminado,
  en curso, en espera…).
- **Sin estado**: lo deduce ("continúan" → en proceso, "falta" → pendiente,
  "se liberó" → concluido) y lo marca para revisarlo.
- **Próximos pasos**: después de *Próximos pasos:*, *Acuerdos:* o en un
  renglón *Próximo paso: …*. Si no hay, la diapositiva se arma sola con lo
  que sigue abierto de cada proyecto.
- Saludos y despedidas ("Hola, les paso el avance", "Saludos") se ignoran;
  renglones cortados en el correo se vuelven a unir.

## La presentación

Sale con el formato del deck *Avance de proyectos* que ya se presenta
(Google Slides, corte del 24/09/2026), que es la plantilla institucional:

- 16:9, **Montserrat**, paleta del Gobierno de México: guinda `#611232`,
  dorado `#A57F2C`, verde `#1E5B4F`.
- **Portada** guinda: título, *Corte al …*, proyectos y quién presenta.
- **Resumen del corte** (con dos o más proyectos): cuántos puntos hay
  concluidos, en proceso y pendientes por proyecto.
- **Una diapositiva por proyecto**: `PROYECTO: NOMBRE`, el equipo y las
  viñetas con *Estado:* en color. Si no cabe, baja la letra un poco y, si
  aun así no cabe, la parte en (1/2), (2/2).
- **Próximos pasos** con la fecha del siguiente corte.

El PDF lleva la fuente incrustada: se ve igual en cualquier computadora. El
PowerPoint se abre en PowerPoint, Keynote y Google Slides, y cada viñeta va
en su propia caja de texto, como en el deck original. Si la computadora no
tiene Montserrat, PowerPoint la sustituye; Google Slides sí la tiene.

Para ver un ejemplo sin base de datos:

```bash
cd backend && cargo run --example muestra -- /tmp
```

## Correr

```bash
./arranca.sh     # levanta base, backend y frontend
./prueba.sh      # crea un corte de prueba, genera PDF y PowerPoint, y lo borra
```

O a mano:

```bash
docker compose up -d                      # Postgres en el puerto 5433
cp .env.example backend/.env
cd backend && cargo run                   # API en el 3001, aplica migraciones
cd frontend && npm install && npm start   # en otra terminal
```

La app queda en <http://localhost:4200> y la API en <http://localhost:3001>.
Los puertos no chocan con `../app` (3000 y 5432), pero los dos frontends usan
el 4200: corre uno a la vez o usa `npm start -- --port 4201` y ajusta
`ORIGEN_PERMITIDO`.

## Estructura

```
backend/
  fuentes/              Montserrat (licencia OFL), incrustada en el PDF.
  migrations/           Esquema. Se aplica solo al arrancar.
  src/dominio/
    lector.rs           Entiende el texto de los equipos.
    maqueta.rs          Qué va en cada diapositiva y dónde: lo comparten PDF y PowerPoint.
    pdf.rs, pptx.rs     Dibujan la maqueta.
    estilo.rs           Colores, medidas y métricas de la fuente.
  src/rutas/            Un archivo por recurso de la API.
  examples/muestra.rs   Presentación de ejemplo sin base.
frontend/
  src/app/paginas/      Cortes, el corte (donde se pega el texto) y proyectos.
```

## La API

| Método | Ruta | Qué hace |
|---|---|---|
| `GET POST` | `/api/cortes` | Lista y alta (`arrastrar: true` copia lo no concluido del anterior) |
| `GET PUT DELETE` | `/api/cortes/:id` | Detalle, portada y borrado |
| `POST` | `/api/leer` | Lee un texto sin guardar nada: `{ texto, proyecto? }` |
| `POST` | `/api/cortes/:id/sumar` | Suma lo leído y revisado |
| `PUT DELETE` | `/api/avances/:id` | Cambia texto o estado (`""` lo quita) |
| `POST` | `/api/cortes/:id/pasos` | Agrega un próximo paso |
| `PUT DELETE` | `/api/pasos/:id` | Cambia o quita un paso |
| `GET POST` | `/api/proyectos` | Catálogo; el orden es el de las diapositivas |
| `PUT DELETE` | `/api/proyectos/:id` | Nombre, equipo, orden, activo |
| `GET` | `/api/cortes/:id/pdf` | La presentación en PDF |
| `GET` | `/api/cortes/:id/pptx` | La presentación en PowerPoint |

## Pruebas

```bash
cd backend && cargo test
```

Cubren el lector (formato del deck, correo informal, catálogo, saludos,
renglones cortados, estados en renglón aparte), la maqueta (orden de las
diapositivas, reparto cuando no cabe, renglones dentro del ancho) y las
fechas en español.

## Seguridad

Igual que `../app`: pensada para correr en tu máquina. Antes de exponerla,
pon `APP_API_KEY` en `backend/.env` y la misma en
`frontend/src/app/nucleo/api.ts`, cambia la contraseña de Postgres y ponla
detrás de HTTPS. El repositorio es público: los avances reales viven solo en
tu base, nunca en el código.
