#!/usr/bin/env bash
# Prueba de humo: crea un corte de prueba, le suma un texto, genera el PDF y
# el PowerPoint, y al final lo borra.
API="${1:-http://localhost:3001}"
ok=0; falla=0

revisa() { # nombre, comando, patrón esperado
  local salida
  salida=$(eval "$2" 2>/dev/null || true)
  if echo "$salida" | grep -q "$3"; then
    echo "  ✓ $1"; ok=$((ok+1))
  else
    echo "  ✗ $1"; echo "      esperaba: $3"; echo "      recibí:   ${salida:0:120}"; falla=$((falla+1))
  fi
}

TEXTO='PROYECTO: PRUEBA DE HUMO\nEquipo de prueba\n• Se probó la lectura. Estado: Concluido.\n• Falta revisar el PDF (pendiente)'
J='-H Content-Type:application/json'

echo "Probando $API"
revisa "el servidor responde"       "curl -s $API/health" "ok"
revisa "los cortes responden"       "curl -s $API/api/cortes" "\["
CORTE=$(curl -s -X POST $J $API/api/cortes -d '{"fecha":"2000-01-01"}' | python3 -c 'import json,sys;print(json.load(sys.stdin)["id"])' 2>/dev/null)
revisa "se crea un corte"           "echo $CORTE" "-"
LEIDO=$(curl -s -X POST $J $API/api/leer -d "{\"texto\":\"$TEXTO\"}")
revisa "lee el texto del equipo"    "echo '$LEIDO'" "Prueba de Humo"
revisa "entiende los estados"       "echo '$LEIDO'" '"pendiente"'
revisa "suma al corte"              "curl -s -X POST $J $API/api/cortes/$CORTE/sumar -d '$LEIDO'" "Se probó la lectura"
revisa "genera el PDF"              "curl -s $API/api/cortes/$CORTE/pdf | head -c 5" "%PDF-"
revisa "genera el PowerPoint"       "curl -s $API/api/cortes/$CORTE/pptx | head -c 2" "PK"
revisa "un estado inválido rebota"  "curl -s -X PUT $J $API/api/avances/00000000-0000-0000-0000-000000000000 -d '{\"estado\":\"inventado\"}'" "estado desconocido"

curl -s -X DELETE "$API/api/cortes/$CORTE" >/dev/null
PROY=$(curl -s $API/api/proyectos | python3 -c 'import json,sys;print(next((p["id"] for p in json.load(sys.stdin) if p["nombre"]=="Prueba de Humo"),""))' 2>/dev/null)
[ -n "$PROY" ] && curl -s -X DELETE "$API/api/proyectos/$PROY" >/dev/null

echo ""
echo "$ok pruebas pasaron, $falla fallaron"
[ "$falla" -eq 0 ]
