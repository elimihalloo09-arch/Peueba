import { Component, OnInit, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';

import { Api } from '../nucleo/api';
import { DineroPipe } from '../nucleo/dinero.pipe';
import { Simulacion } from '../nucleo/modelos';

@Component({
  selector: 'app-plan',
  standalone: true,
  imports: [FormsModule, DineroPipe],
  template: `
    <div class="sec-head">
      <h2>Plan de liquidación</h2>
      <span class="hint">Mueve los números y vuelve a calcular</span>
    </div>

    <div class="tarjeta">
      <div class="campos">
        <div>
          <label class="lab" for="cap">Al mes para deuda</label>
          <input id="cap" type="number" [(ngModel)]="capacidad" (ngModelChange)="calcular()">
        </div>
        <div>
          <label class="lab" for="apoyo">Apoyo mensual</label>
          <input id="apoyo" type="number" [(ngModel)]="apoyo" (ngModelChange)="calcular()">
        </div>
        <div>
          <label class="lab" for="meses">Meses de apoyo</label>
          <input id="meses" type="number" [(ngModel)]="mesesApoyo" (ngModelChange)="calcular()">
        </div>
        <div>
          <label class="lab" for="metodo">Orden</label>
          <select id="metodo" [(ngModel)]="metodo" (ngModelChange)="calcular()">
            <option value="avalancha">Avalancha · tasa más alta</option>
            <option value="bola_de_nieve">Bola de nieve · saldo más chico</option>
          </select>
        </div>
      </div>
      <label class="check">
        <input type="checkbox" [(ngModel)]="sinQuitas" (ngModelChange)="calcular()">
        Calcular sin quitas, con los saldos completos
      </label>
    </div>

    @if (s(); as sim) {
      <div class="resultados">
        <div class="tarjeta">
          <p class="kicker">Libre de deudas en</p>
          <p class="cifra" [style.color]="sim.inalcanzable ? 'var(--deuda)' : 'var(--good)'">
            {{ sim.inalcanzable ? 'no alcanza' : sim.meses + (sim.meses === 1 ? ' mes' : ' meses') }}
          </p>
        </div>
        <div class="tarjeta">
          <p class="kicker">Intereses que pagarías</p>
          <p class="cifra" style="color:var(--deuda)">{{ sim.intereses | dinero }}</p>
        </div>
        <div class="tarjeta">
          <p class="kicker">Total desembolsado</p>
          <p class="cifra">{{ sim.total_pagado | dinero }}</p>
        </div>
      </div>

      @if (sim.inalcanzable) {
        <p class="error">
          Con ese abono no se liquidan: {{ sim.atoradas.join(', ') }}.
          Los intereses crecen más rápido que el pago.
        </p>
      }

      <div class="sec-head" style="margin-top:26px"><h2>Orden de pago</h2></div>
      <ol class="orden">
        @for (o of sim.orden; track o.nombre; let i = $index) {
          <li>
            <span class="n">{{ i + 1 }}</span>
            <span class="que">{{ o.nombre }}</span>
            <span class="num cuanto">{{ o.total_pagado | dinero }}</span>
            <span class="num cuando">mes {{ o.mes }}</span>
          </li>
        }
      </ol>
    } @else if (error()) {
      <p class="error">{{ error() }}</p>
    } @else {
      <p class="cargando">Calculando…</p>
    }
  `,
  styles: [`
    .campos{ display:grid; grid-template-columns:repeat(4,1fr); gap:12px; }
    .check{ display:flex; align-items:center; gap:9px; margin-top:16px; font-size:13.5px; color:var(--ink-2); }
    .check input{ width:auto; }
    .resultados{ display:grid; grid-template-columns:repeat(auto-fit,minmax(160px,1fr)); gap:12px; margin-top:16px; }
    .cifra{ font-family:"Archivo",sans-serif; font-weight:800; font-size:clamp(20px,3.6vw,27px);
            letter-spacing:-.03em; font-variant-numeric:tabular-nums; margin:8px 0 0; }
    .orden{ list-style:none; padding:0; margin:0; }
    .orden li{ display:grid; grid-template-columns:28px 1fr auto auto; gap:12px; align-items:baseline;
               padding:11px 0; border-bottom:1px solid var(--hair-soft); font-size:14px; }
    .n{ font-family:"Archivo",sans-serif; font-weight:800; color:var(--a); }
    .cuanto,.cuando{ font-size:12.5px; color:var(--ink-2); }
    @media (max-width:560px){
      .campos{ grid-template-columns:1fr 1fr; }
      .orden li{ grid-template-columns:24px 1fr auto; }
      .cuando{ grid-column:2 / -1; color:var(--ink-3); }
    }
  `],
})
export class PlanComponent implements OnInit {
  private api = inject(Api);
  // En pesos: se convierten a centavos al llamar.
  capacidad = 13352;
  apoyo = 10000;
  mesesApoyo = 6;
  metodo = 'avalancha';
  sinQuitas = false;

  s = signal<Simulacion | null>(null);
  error = signal('');
  private temporizador?: ReturnType<typeof setTimeout>;

  ngOnInit(): void { this.calcular(); }

  calcular(): void {
    clearTimeout(this.temporizador);
    this.temporizador = setTimeout(() => {
      this.api.plan({
        capacidad: Math.round((this.capacidad || 0) * 100),
        metodo: this.metodo,
        apoyo_mensual: Math.round((this.apoyo || 0) * 100),
        meses_apoyo: Math.max(0, Math.round(this.mesesApoyo || 0)),
        sin_quitas: this.sinQuitas,
      }).subscribe({
        next: (d) => { this.s.set(d); this.error.set(''); },
        error: () => this.error.set('No se pudo calcular. ¿Está corriendo el backend?'),
      });
    }, 300);
  }
}
