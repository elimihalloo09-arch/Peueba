import { Component, OnInit, inject, signal } from '@angular/core';

import { Api } from '../nucleo/api';
import { DineroPipe } from '../nucleo/dinero.pipe';
import { CuentaPersona, Resumen } from '../nucleo/modelos';

@Component({
  selector: 'app-resumen',
  standalone: true,
  imports: [DineroPipe],
  template: `
    @if (cargando()) {
      <p class="cargando">Cargando…</p>
    }
    @if (error()) {
      <p class="error">{{ error() }}</p>
    }
    @if (r(); as d) {
      <section>
        <div class="sec-head"><h2>El mes</h2><span class="hint">Ingreso del hogar contra lo que sale</span></div>
        <div class="rejilla">
          <div class="tarjeta">
            <p class="kicker">Entra</p>
            <p class="cifra">{{ d.ingreso_mensual | dinero }}</p>
          </div>
          <div class="tarjeta">
            <p class="kicker">Sale en gastos</p>
            <p class="cifra">{{ d.gasto_mensual | dinero }}</p>
            <p class="pie">{{ d.gasto_recortable | dinero }} es recortable</p>
          </div>
          <div class="tarjeta">
            <p class="kicker">Te queda</p>
            <p class="cifra" [style.color]="d.capacidad_mensual < 0 ? 'var(--deuda)' : 'var(--good)'">
              {{ d.capacidad_mensual | dinero }}
            </p>
            <p class="pie">antes de pagos de deuda</p>
          </div>
        </div>
      </section>

      <section style="margin-top:30px">
        <div class="sec-head"><h2>La deuda</h2><span class="hint">Con las quitas ya aplicadas</span></div>
        <div class="rejilla">
          <div class="tarjeta">
            <p class="kicker">Por pagar</p>
            <p class="cifra" style="color:var(--deuda)">{{ d.deuda_por_pagar | dinero }}</p>
          </div>
          <div class="tarjeta">
            <p class="kicker">Ya liquidado</p>
            <p class="cifra" style="color:var(--good)">{{ d.deuda_liquidada | dinero }}</p>
          </div>
          <div class="tarjeta">
            <p class="kicker">Ahorrado en quitas</p>
            <p class="cifra" style="color:var(--c)">{{ d.ahorro_por_quitas | dinero }}</p>
          </div>
          <div class="tarjeta">
            <p class="kicker">Mínimos al mes</p>
            <p class="cifra">{{ d.pagos_mensuales_deuda | dinero }}</p>
            <p class="pie">sin contar lo de nómina</p>
          </div>
        </div>
      </section>

      @if (cuentas().length) {
        <section style="margin-top:30px">
          <div class="sec-head"><h2>Préstamos de personas</h2><span class="hint">Lo que te dieron y lo que has pagado</span></div>
          @for (c of cuentas(); track c.persona) {
            <div class="tarjeta" style="margin-bottom:10px">
              <div class="fila">
                <strong>{{ c.persona }}</strong>
                <span class="num" style="color:var(--deuda)">Le debes {{ c.saldo | dinero }}</span>
              </div>
              <p class="pie">Te dio {{ c.entregado | dinero }} · le has pagado {{ c.pagado | dinero }}</p>
            </div>
          }
        </section>
      }
    }
  `,
  styles: [`
    .rejilla{ display:grid; grid-template-columns:repeat(auto-fit,minmax(170px,1fr)); gap:12px; }
    .cifra{ font-family:"Archivo",sans-serif; font-weight:800; font-size:clamp(22px,4vw,30px);
            letter-spacing:-.03em; font-variant-numeric:tabular-nums; margin:8px 0 0; }
    .pie{ font-size:12.5px; color:var(--ink-3); margin:6px 0 0; }
    .fila{ display:flex; justify-content:space-between; gap:12px; align-items:baseline; }
  `],
})
export class ResumenComponent implements OnInit {
  private api = inject(Api);
  r = signal<Resumen | null>(null);
  cuentas = signal<CuentaPersona[]>([]);
  cargando = signal(true);
  error = signal('');

  ngOnInit(): void {
    this.api.resumen().subscribe({
      next: (d) => { this.r.set(d); this.cargando.set(false); },
      error: () => { this.error.set('No se pudo conectar con el backend. ¿Está corriendo en el puerto 3000?'); this.cargando.set(false); },
    });
    this.api.movimientos().subscribe({ next: (c) => this.cuentas.set(c), error: () => {} });
  }
}
