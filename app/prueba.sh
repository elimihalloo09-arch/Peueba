#!/usr/bin/env bash
# Prueba de humo: verifica que la API responde y que los números cuadran.
API="${1:-http://localhost:3000}"
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

echo "Probando $API"
revisa "el servidor responde"        "curl -s $API/health" "ok"
revisa "hay acreedores cargados"     "curl -s $API/api/acreedores" "HeyBanco"
revisa "el resumen calcula"          "curl -s $API/api/resumen" "ingreso_mensual"
revisa "hay pendientes"              "curl -s $API/api/pendientes" "HeyBanco"
revisa "el préstamo de Mario está"   "curl -s $API/api/movimientos" "Mario"
revisa "el plan simula"              "curl -s '$API/api/plan?capacidad=1335200&metodo=avalancha&apoyo_mensual=1000000&meses_apoyo=6'" '"meses"'
revisa "una etapa inválida rebota"   "curl -s -X PUT $API/api/acreedores/00000000-0000-0000-0000-000000000000 -H 'Content-Type: application/json' -d '{\"etapa\":\"inventada\"}'" "etapa desconocida"

echo ""
echo "Tus números ahora mismo:"
curl -s "$API/api/resumen" | python3 -c "
import json,sys
d=json.load(sys.stdin)
p=lambda t,c: print(f'  {t:<28} \${round(c/100):>10,}')
p('Entra al mes', d['ingreso_mensual'])
p('Se va en gastos', d['gasto_mensual'])
p('Mínimos de deuda', d['pagos_mensuales_deuda'])
p('Deuda por pagar', d['deuda_por_pagar'])
p('Ahorrado en quitas', d['ahorro_por_quitas'])
p('Le debes a personas', d['saldo_con_personas'])
" 2>/dev/null || echo "  (no pude leer el resumen)"

echo ""
echo "$ok pruebas pasaron, $falla fallaron"
[ "$falla" -eq 0 ]
