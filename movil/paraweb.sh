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
</style>

<div id="aviso" hidden>
  <h1>La app no arrancó</h1>
  <p>Estuvo diez segundos intentando cargar y no lo logró. Pásale estos dos
     datos a Claude y con eso lo arregla.</p>
  <dl>
    <dt>Ruta que usó</dt><dd id="ruta"></dd>
    <dt>Qué falló</dt><dd id="detalle"></dd>
  </dl>
</div>

<script>
(function () {
  // La pagina se sirve en una ruta sin diagonal final, asi que una referencia
  // relativa buscaria los archivos un nivel arriba. Se fija la base a mano.
  var ruta = location.pathname;
  if (ruta.charAt(ruta.length - 1) !== '/') ruta += '/';

  var base = document.createElement('base');
  base.href = ruta;
  document.head.appendChild(base);

  var fallo = '';
  window.addEventListener('error', function (e) {
    if (e && e.target && e.target.src) fallo = 'no cargo ' + e.target.src;
    else if (e && e.message) fallo = e.message;
  }, true);

  // Absoluta a proposito: esta primera carga no debe depender del <base>.
  var s = document.createElement('script');
  s.src = ruta + 'flutter_bootstrap.js';
  s.async = true;
  document.head.appendChild(s);

  setTimeout(function () {
    if (document.querySelector('flutter-view, flt-glass-pane, flt-scene-host')) return;
    document.getElementById('ruta').textContent = ruta;
    document.getElementById('detalle').textContent = fallo || 'sin error reportado';
    document.getElementById('aviso').hidden = false;
  }, 10000);
})();
</script>
HTML

echo "build/web listo para publicar"
