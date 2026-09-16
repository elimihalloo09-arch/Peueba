#!/usr/bin/env bash
# Compila la version web y reescribe index.html para publicarla como pagina
# alojada: sin doctype ni html/head/body propios, porque el alojamiento
# envuelve la pagina en su propio esqueleto y ahi un <base> escrito a mano
# queda dentro del <body>, donde Safari lo ignora y nada carga.
set -euo pipefail
export PATH="$HOME/flutter/bin:/tmp/claude-0/-home-user-Peueba/5e937d27-69d5-5768-9851-492e3de7f843/scratchpad/flutter/bin:$PATH"
cd "$(dirname "$0")"

flutter build web --release --web-renderer html --pwa-strategy offline-first

# CanvasKit no se usa con el renderizador html: son 19 MB de mas.
rm -rf build/web/canvaskit build/web/assets/AssetManifest.bin build/web/.last_build_id

cat > build/web/index.html <<'HTML'
<title>Mis Finanzas</title>
<style>
  :root{ --fondo:#eceff1; --tinta:#1a1c1e; --suave:#5f6368; }
  @media (prefers-color-scheme: dark){
    :root:not([data-theme="light"]){ --fondo:#101418; --tinta:#e3e2e6; --suave:#a8abb0; }
  }
  :root[data-theme="dark"]{ --fondo:#101418; --tinta:#e3e2e6; --suave:#a8abb0; }
  html, body { height:100%; }
  body{ margin:0; background:var(--fondo); color:var(--tinta);
        font:15px/1.55 -apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif; }
  #aviso{ padding:28px 16px; max-width:34rem; margin:0 auto; }
  #aviso h1{ font-size:19px; margin:0 0 12px; }
  #aviso p{ margin:0 0 12px; }
  #aviso dt{ font-size:11px; letter-spacing:.09em; text-transform:uppercase;
             color:var(--suave); margin-top:14px; }
  #aviso dd{ margin:4px 0 0; font-family:ui-monospace,SFMono-Regular,Menlo,monospace;
             font-size:12.5px; word-break:break-all; }
  #aviso li{ font-family:ui-monospace,SFMono-Regular,Menlo,monospace;
             font-size:12.5px; word-break:break-all; margin-bottom:4px; }
</style>

<div id="aviso" hidden>
  <h1>La app no arrancó</h1>
  <p>Buscó sus archivos en varias rutas y ninguna sirvió. Pásale esta lista a
     Claude y con eso lo arregla.</p>
  <dl>
    <dt>Dirección de la página</dt><dd id="aqui"></dd>
    <dt>Rutas que intentó</dt><dd><ul id="intentos"></ul></dd>
  </dl>
</div>

<script>
(function () {
  // No se puede dar por hecho donde quedan los archivos: depende de como los
  // sirva el alojamiento, y suponerlo ya fallo dos veces. Se prueban las
  // rutas posibles en orden y se usa la primera que responda.
  var ruta = location.pathname;
  var candidatas = [];
  [
    ruta.replace(/[^/]*$/, ''),                                  // la carpeta de la pagina
    ruta.charAt(ruta.length - 1) === '/' ? ruta : ruta + '/',    // la pagina como carpeta
    '/'                                                          // la raiz del dominio
  ].forEach(function (c) { if (candidatas.indexOf(c) < 0) candidatas.push(c); });

  var intentos = [];

  function ponerBase(href) {
    var b = document.querySelector('base') || document.createElement('base');
    b.href = href;
    if (!b.parentNode) document.head.appendChild(b);
  }

  function intentar(i) {
    if (i >= candidatas.length) return rendirse();
    var base = candidatas[i];
    ponerBase(base);                    // el arranque lo usa para lo demas
    var s = document.createElement('script');
    s.src = base + 'flutter_bootstrap.js';
    s.async = true;
    s.onerror = function () {
      intentos.push('no ' + s.src);
      s.parentNode && s.parentNode.removeChild(s);
      intentar(i + 1);
    };
    s.onload = function () { intentos.push('si ' + s.src); };
    document.head.appendChild(s);
  }

  function rendirse() {
    document.getElementById('aqui').textContent = location.href;
    var ul = document.getElementById('intentos');
    intentos.forEach(function (t) {
      var li = document.createElement('li');
      li.textContent = t;
      ul.appendChild(li);
    });
    document.getElementById('aviso').hidden = false;
  }

  intentar(0);

  setTimeout(function () {
    if (document.querySelector('flutter-view, flt-glass-pane, flt-scene-host')) return;
    rendirse();
  }, 12000);
})();
</script>
HTML

echo "build/web listo para publicar"
