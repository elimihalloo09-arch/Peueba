#!/usr/bin/env python3
"""Arma un index.html que se basta solo.

El alojamiento sirve los archivos a traves de un manifiesto con nombres con
hash, no por su ruta, asi que cualquier referencia relativa falla y no hay
ruta que adivinar. La salida de aqui no referencia ningun archivo externo: el
codigo compilado va dentro del HTML y los assets van incrustados en base64,
servidos interceptando fetch.
"""
import base64, json, pathlib, re, sys

RAIZ = pathlib.Path(__file__).parent / 'build' / 'web'

# Los assets que el motor pide en tiempo de ejecucion. NOTICES son 1.7 MB de
# licencias que solo se usan en la pantalla de creditos: se sirve vacio.
ASSETS = [
    'assets/FontManifest.json',
    'assets/AssetManifest.json',
    'assets/AssetManifest.bin.json',
    'assets/fonts/MaterialIcons-Regular.otf',
    'assets/packages/cupertino_icons/assets/CupertinoIcons.ttf',
    'assets/shaders/ink_sparkle.frag',
]

def leer(nombre):
    return (RAIZ / nombre).read_bytes()

def b64(datos):
    return base64.b64encode(datos).decode('ascii')

bootstrap = leer('flutter_bootstrap.js').decode('utf-8')

# El bootstrap es flutter.js, luego buildConfig, luego la llamada a load() que
# inyectaria main.dart.js por URL. Se toma todo menos esa llamada.
corte = bootstrap.index('_flutter.loader.load(')
cabeza = bootstrap[:corte]

principal = leer('main.dart.js').decode('utf-8')
for prohibido in ('</script', '<!--'):
    if prohibido in principal or prohibido in cabeza:
        sys.exit(f'El codigo trae {prohibido!r} y romperia el HTML')

embebidos = {nombre: b64(leer(nombre)) for nombre in ASSETS}
icono = b64(leer('icons/Icon-192.png'))

html = f'''<title>Mis Finanzas</title>
<link rel="apple-touch-icon" href="data:image/png;base64,{icono}">
<style>
  :root{{ --fondo:#eceff1; --tinta:#1a1c1e; --suave:#5f6368; }}
  @media (prefers-color-scheme: dark){{
    :root:not([data-theme="light"]){{ --fondo:#101418; --tinta:#e3e2e6; --suave:#a8abb0; }}
  }}
  :root[data-theme="dark"]{{ --fondo:#101418; --tinta:#e3e2e6; --suave:#a8abb0; }}
  html, body {{ height:100%; }}
  body{{ margin:0; background:var(--fondo); color:var(--tinta);
        font:15px/1.55 -apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif; }}
  #aviso{{ padding:28px 16px; max-width:34rem; margin:0 auto; }}
  #aviso h1{{ font-size:19px; margin:0 0 12px; }}
  #aviso dt{{ font-size:11px; letter-spacing:.09em; text-transform:uppercase;
             color:var(--suave); margin-top:14px; }}
  #aviso dd{{ margin:4px 0 0; font-family:ui-monospace,SFMono-Regular,Menlo,monospace;
             font-size:12.5px; word-break:break-all; }}
</style>

<div id="aviso" hidden>
  <h1>La app no arrancó</h1>
  <p>Todo viene dentro de esta misma página, así que no es un archivo que
     falte. Pásale esto a Claude.</p>
  <dl><dt>Error</dt><dd id="detalle"></dd></dl>
</div>

<script>
// Los assets viven aqui dentro. Se interceptan las peticiones del motor y se
// responden con estos bytes, sin tocar la red.
(function () {{
  var EMB = {json.dumps(embebidos)};
  var red = window.fetch ? window.fetch.bind(window) : null;

  function bytes(b64) {{
    var s = atob(b64), a = new Uint8Array(s.length);
    for (var i = 0; i < s.length; i++) a[i] = s.charCodeAt(i);
    return a;
  }}
  function tipo(k) {{
    if (/\\.json$/.test(k)) return 'application/json';
    if (/\\.otf$/.test(k))  return 'font/otf';
    if (/\\.ttf$/.test(k))  return 'font/ttf';
    return 'text/plain';
  }}
  function responder(cuerpo, ct) {{
    return Promise.resolve(new Response(cuerpo, {{ status: 200, headers: {{ 'Content-Type': ct }} }}));
  }}

  // Las fuentes no pasan por fetch: el motor usa la API FontFace, que baja la
  // URL por su cuenta. Se le cambia la URL por los bytes ya incrustados.
  var FuenteOriginal = window.FontFace;
  if (FuenteOriginal) {{
    var Envoltura = function (familia, origen, desc) {{
      if (typeof origen === 'string') {{
        for (var k in EMB) {{
          if (origen.indexOf(k) !== -1) {{
            origen = 'url(data:' + tipo(k) + ';base64,' + EMB[k] + ')';
            break;
          }}
        }}
      }}
      return new FuenteOriginal(familia, origen, desc);
    }};
    Envoltura.prototype = FuenteOriginal.prototype;
    window.FontFace = Envoltura;
  }}

  window.fetch = function (entrada, opciones) {{
    var url = typeof entrada === 'string' ? entrada : (entrada && entrada.url) || '';
    for (var k in EMB) if (url.indexOf(k) !== -1) return responder(bytes(EMB[k]), tipo(k));
    // Las licencias no se incrustan: son 1.7 MB y solo salen en una pantalla.
    if (url.indexOf('assets/NOTICES') !== -1) return responder('', 'text/plain');
    return red ? red(entrada, opciones) : Promise.reject(new Error('sin red'));
  }};
}})();
</script>

<script>
{cabeza}
</script>

<script>
// Normalmente el cargador inyectaria main.dart.js por URL. Aqui ya viene
// incrustado mas abajo, asi que solo se deja puesto el enlace que ese codigo
// busca al terminar de ejecutarse.
window._flutter = window._flutter || {{}};
_flutter.loader = _flutter.loader || {{}};
_flutter.loader.didCreateEngineInitializer = function (init) {{
  init.initializeEngine({{}}).then(function (app) {{ app.runApp(); }});
}};
window.addEventListener('error', function (e) {{
  window.__falla = (e && e.message) || 'error sin mensaje';
}});
</script>

<script>
{principal}
</script>

<script>
setTimeout(function () {{
  if (document.querySelector('flutter-view, flt-glass-pane, flt-scene-host')) return;
  document.getElementById('detalle').textContent = window.__falla || 'sin error reportado';
  document.getElementById('aviso').hidden = false;
}}, 12000);
</script>
'''

salida = RAIZ / 'index.html'
salida.write_text(html, encoding='utf-8')
print(f'index.html autocontenido: {salida.stat().st_size / 1048576:.2f} MB')
