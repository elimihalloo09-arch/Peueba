import { Component, OnInit, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Router, RouterLink } from '@angular/router';

import { Api, fechaCorta, hoyIso, sumarDias } from '../nucleo/api';
import { CorteEnLista } from '../nucleo/modelos';

@Component({
  selector: 'app-cortes',
  standalone: true,
  imports: [FormsModule, RouterLink],
  template: `
    <section class="tarjeta nuevo">
      <div class="sec-head">
        <h2>Nuevo corte</h2>
        <span class="hint">Una presentación por fecha de corte</span>
      </div>
      <div class="campos">
        <div>
          <label class="lab" for="fecha">Corte al</label>
          <input id="fecha" type="date" [(ngModel)]="fecha" (ngModelChange)="siguiente = sumarDias($event, 7)">
        </div>
        <div>
          <label class="lab" for="siguiente">Siguiente corte</label>
          <input id="siguiente" type="date" [(ngModel)]="siguiente">
        </div>
        <button class="btn" type="button" (click)="crear()" [disabled]="!fecha || creando()">Crear corte</button>
      </div>
      @if (lista().length) {
        <label class="arrastra">
          <input type="checkbox" [(ngModel)]="arrastrar">
          <span>Traer lo que quedó <b>en proceso</b> o <b>pendiente</b> del corte anterior, para solo actualizarlo</span>
        </label>
      }
    </section>

    <div class="sec-head">
      <h2>Cortes</h2>
      <span class="hint">{{ lista().length }} en total</span>
    </div>
    <div class="tarjeta lista">
      @for (c of lista(); track c.id) {
        <a class="fila" [routerLink]="['/cortes', c.id]">
          <span class="fecha">{{ fechaCorta(c.fecha) }}</span>
          <span class="txt">
            <b>{{ c.titulo }}</b>
            <small>{{ c.proyectos }} {{ c.proyectos === 1 ? 'proyecto' : 'proyectos' }} · {{ c.puntos }} {{ c.puntos === 1 ? 'punto' : 'puntos' }}
              @if (c.presenta) { · {{ c.presenta }} }</small>
          </span>
          <span class="ir" aria-hidden="true">›</span>
        </a>
      } @empty {
        @if (!error()) { <p class="cargando vacio">Todavía no hay cortes. Crea el primero arriba.</p> }
      }
    </div>
    @if (error()) { <p class="error">{{ error() }}</p> }
  `,
  styles: [`
    .nuevo{ margin-bottom:28px; }
    .campos{ display:grid; grid-template-columns:1fr 1fr auto; gap:12px; align-items:end; }
    @media (max-width:560px){ .campos{ grid-template-columns:1fr 1fr; } .campos .btn{ grid-column:1 / -1; } }
    .arrastra{ display:flex; gap:10px; align-items:flex-start; margin-top:14px; font-size:13.5px; color:var(--ink-2); cursor:pointer; }
    .arrastra input{ width:auto; margin-top:3px; accent-color:var(--guinda); }
    .lista{ padding:0; }
    .fila{ display:grid; grid-template-columns:110px 1fr 20px; gap:14px; align-items:center;
           padding:14px 18px; border-bottom:1px solid var(--hair-soft); text-decoration:none; color:inherit; }
    .fila:last-child{ border-bottom:0; }
    .fila:hover{ background:var(--sunken); }
    .fecha{ font-weight:700; color:var(--guinda); font-variant-numeric:tabular-nums; }
    .txt{ display:flex; flex-direction:column; min-width:0; }
    .txt small{ color:var(--ink-3); font-size:12.5px; }
    .ir{ color:var(--ink-3); font-size:20px; }
    .vacio{ padding:18px; }
  `],
})
export class CortesComponent implements OnInit {
  private api = inject(Api);
  private router = inject(Router);
  lista = signal<CorteEnLista[]>([]);
  error = signal('');
  creando = signal(false);
  fecha = hoyIso();
  siguiente = sumarDias(hoyIso(), 7);
  arrastrar = true;
  fechaCorta = fechaCorta;
  sumarDias = sumarDias;

  ngOnInit(): void {
    this.api.cortes().subscribe({
      next: (l) => this.lista.set(l),
      error: () => this.error.set('No se pudo conectar con el backend. ¿Está corriendo en el puerto 3001?'),
    });
  }

  crear(): void {
    this.creando.set(true);
    this.api.crearCorte({ fecha: this.fecha, siguiente: this.siguiente || null, arrastrar: this.arrastrar }).subscribe({
      next: (c) => this.router.navigate(['/cortes', c.id]),
      error: () => { this.creando.set(false); this.error.set('No se pudo crear el corte.'); },
    });
  }
}
