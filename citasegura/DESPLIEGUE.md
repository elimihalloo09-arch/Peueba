# Poner CitaSegura en un servidor (VPS)

Para un consultorio real el bot no puede vivir en tu compu: si se apaga o se va el internet,
no salen los recordatorios. Esta guia lo deja corriendo 24/7 en un servidor barato, con
HTTPS, reinicio automatico y respaldo diario cifrado.

Tiempo: 1 a 2 horas la primera vez.

## Lo que necesitas

| Que | Para que | Costo aproximado |
|---|---|---|
| Un VPS con **Ubuntu 24.04**, 1 GB de RAM | donde corre el bot | ~5-6 USD al mes |
| Un **dominio o subdominio** (ej. `bot.tudominio.com`) | Meta exige una direccion `https://` fija | desde gratis (DuckDNS) |
| Tu `.env` con llaves reales | Claude y WhatsApp | — |
| Un **token permanente** de WhatsApp | el temporal caduca en 24 h | gratis |

### Token permanente de WhatsApp
El token del panel de pruebas dura 24 horas. Para produccion crea un **usuario del sistema**
en Meta Business (Configuracion del negocio -> Usuarios -> Usuarios del sistema), asignale
tu app y genera un token con los permisos `whatsapp_business_messaging` y
`whatsapp_business_management`. Ese va en `WA_TOKEN`. Meta cambia sus pantallas seguido:
si algo no coincide, busca "system user access token" en la documentacion de WhatsApp Cloud API.

## Paso 1. Preparar el servidor

Entra por SSH (`ssh root@IP_DEL_SERVIDOR`) y corre:

```bash
timedatectl set-timezone America/Mexico_City     # para que el respaldo de las 3 am sea hora de Mexico
curl -fsSL https://get.docker.com | sh            # instala Docker
ufw allow OpenSSH && ufw allow 80 && ufw allow 443 && ufw --force enable   # firewall
```

## Paso 2. Apuntar el dominio al servidor

En tu proveedor de dominio crea un registro **A**: `bot.tudominio.com` -> IP del servidor.
Puede tardar unos minutos. Comprueba con `ping bot.tudominio.com` que responda la IP correcta.

## Paso 3. Bajar el codigo y configurarlo

```bash
git clone -b claude/new-session-wxd9j1 https://github.com/elimihalloo09-arch/Peueba.git
cd Peueba/citasegura
mkdir -p datos && chown 1000:1000 datos           # aqui viven la base de datos y los respaldos
cp .env.example .env
nano .env
```

En `.env` llena **todo**: llaves de Claude y de Meta, datos de la clinica, `TELEFONO_HUMANO`
y `DOMINIO=bot.tudominio.com`. El servidor revisa al arrancar que esten las llaves
(`WA_TOKEN`, `WA_PHONE_NUMBER_ID`, `WA_VERIFY_TOKEN`, `WA_APP_SECRET`, `ANTHROPIC_API_KEY`):
si falta una, no arranca y te dice cual.

## Paso 4. Arrancar

```bash
docker compose up -d --build
docker compose ps                  # bot "healthy" y caddy "Up"
curl https://bot.tudominio.com/salud     # debe responder {"ok":true}
```

Caddy saca solo el certificado HTTPS (gratis, de Let's Encrypt) la primera vez. Si `curl`
falla, espera un minuto y revisa `docker compose logs caddy`: casi siempre es que el dominio
todavia no apunta al servidor o el puerto 80/443 esta cerrado.

## Paso 5. Conectar Meta

En Meta -> WhatsApp -> Configuracion:
- Callback URL: `https://bot.tudominio.com/webhook`
- Verify token: el mismo de `WA_VERIFY_TOKEN`
- Suscribete al campo **messages**

Escribele al numero desde tu celular. Para ver lo que pasa: `docker compose logs -f bot`.

## Paso 6. Respaldo diario cifrado

Los datos de pacientes son sensibles: el respaldo se guarda cifrado.

```bash
openssl rand -base64 32 > ~/.citasegura-clave && chmod 600 ~/.citasegura-clave
cat ~/.citasegura-clave      # GUARDA esta clave en otro lado (gestor de contrasenas). Sin ella no se recupera nada.
./respaldar.sh               # prueba: debe decir "Respaldo listo: cifrados/..."
```

Programalo todos los dias a las 3 am con `crontab -e` y esta linea (ajusta la ruta):

```
0 3 * * * /root/Peueba/citasegura/respaldar.sh >> /root/Peueba/citasegura/respaldo.log 2>&1
```

Guarda 30 dias en `cifrados/`. **Un respaldo en el mismo servidor se pierde si se pierde el
servidor**: bajalos de vez en cuando a tu compu
(`scp root@IP:/root/Peueba/citasegura/cifrados/* .`) o configura `rclone` para mandarlos a la
nube (ver el final de `respaldar.sh`).

### Restaurar un respaldo

```bash
docker compose stop bot
gpg --batch --pinentry-mode loopback --passphrase-file ~/.citasegura-clave \
    -d cifrados/citasegura-FECHA.db.gz.gpg | gunzip > datos/citasegura.db
chown 1000:1000 datos/citasegura.db
docker compose start bot
```

## Dia a dia

| Quiero… | Comando (dentro de `Peueba/citasegura`) |
|---|---|
| Ver que esta pasando | `docker compose logs -f bot` |
| Actualizar a la ultima version | `git pull && docker compose up -d --build` |
| Cambiar algo del `.env` | editar `.env` y `docker compose up -d` |
| Reiniciar | `docker compose restart bot` |
| Ver si esta sano | `docker compose ps` |

Si el servidor se reinicia, Docker vuelve a levantar el bot solo.

## Por que asi

- **Un solo proceso del bot:** con dos, cada uno mandaria su propio recordatorio y el paciente
  recibiria dos (y se pagarian dos).
- **Caddy en vez de Cloudflare Tunnel:** no necesita cuenta extra; solo un dominio. Si
  prefieres el tunel de Cloudflare, quita el servicio `caddy` y usa un tunel con nombre.
- **SQLite:** suficiente para varios consultorios pequenos. Con 5 o mas clientes, PostgreSQL.
- **Los logs no guardan el contenido de los mensajes** (privacidad de los pacientes).
