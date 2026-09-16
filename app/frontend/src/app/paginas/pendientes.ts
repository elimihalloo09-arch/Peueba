import { Component, OnInit, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';

import { Api } from '../nucleo/api';
import { Pendiente } from '../nucleo/modelos';

@Component({
  selector: 'app-pendientes',
  standalone: true,
  imports: [FormsModule],
  template: `
    <div class="sec-head">
      <h2>Pendientes</h2>
      <span class="hint">{{ faltan() }} sin hacer</span>
    </div>

    <div class="tarjeta lista">
      @for (p of lista(); track p.id) {
        <div class="fila" [class.ok]="p.hecho">
          <button type="button" class="check" [attr.aria-pressed]="p.hecho"
                  [attr.aria-label]="(p.hecho ? 'Desmarcar: ' : 'Marcar como hecho: ') + p.texto"
                  (click)="alternar(p)">{{ p.hecho ? '✓' : '' }}</button>
          <span class="txt">{{ p.texto }}</span>
          @if (p.vence) {
            <span class="cuando" [class.urge]="!p.hecho && diasPara(p.vence) <= 1"
                  [class.pronto]="!p.hecho && diasPara(p.vence) > 1 && diasPara(p.vence) <= 7">
              {{ etiqueta(p) }}
            </span>
          } @else { <span></span> }
          <button type="button" class="quitar" aria-label="Quitar pendiente" (click)="quitar(p)">×</button>
        </div>
      } @empty {
        <p class="cargando">Sin pendientes. Agrega el primero abajo.</p>
      }
    </div>

    <div class="nuevo">
      <input type="text" [(ngModel)]="texto" placeholder="Qué falta hacer"
             (keyup.enter)="agregar()" aria-label="Nuevo pendiente">
      <input type="date" [(ngModel)]="vence" aria-label="Cuándo vence">
      <button class="btn" type="button" (click)="agregar()" [disabled]="!texto.trim()">Agregar</button>
    </div>

    @if (error()) { <p class="error">{{ error() }}</p> }
  `,
  styles: [`
    .lista{ padding:0; }
    .fila{ display:grid; grid-template-columns:28px 1fr auto 28px; gap:12px; align-items:center;
           padding:13px 16px; border-bottom:1px solid var(--hair-soft); }
    .fila:last-child{ border-bottom:0; }
    .fila.ok .txt{ text-decoration:line-through; color:var(--ink-3); }
    .check{ width:24px; height:24px; border-radius:6px; border:2px solid var(--hair); background:transparent;
            cursor:pointer; display:flex; align-items:center; justify-content:center; padding:0;
            color:#fff; font-size:14px; line-height:1; }
    .check[aria-pressed=true]{ background:var(--good); border-color:var(--good); }
    .txt{ font-size:14.5px; }
    .cuando{ font-family:"IBM Plex Mono",monospace; font-size:11px; padding:4px 9px; border-radius:999px;
             border:1px solid var(--hair); color:var(--ink-3); white-space:nowrap; }
    .cuando.urge{ color:var(--deuda); border-color:var(--deuda); }
    .cuando.pronto{ color:var(--b); border-color:var(--b); }
    .quitar{ font:inherit; font-size:16px; color:var(--ink-3); background:transparent; border:0; cursor:pointer; }
    .nuevo{ display:grid; grid-template-columns:1fr 150px auto; gap:8px; margin-top:14px; }
    @media (max-width:560px){ .nuevo{ grid-template-columns:1fr 130px; } .nuevo .btn{ grid-column:1 / -1; } }
  `],
})
export class PendientesComponent implements OnInit {
  private api = inject(Api);
  lista = signal<Pendiente[]>([]);
  error = signal('');
  texto = '';
  vence = '';

  ngOnInit(): void { this.cargar(); }

  faltan(): number { return this.lista().filter((p) => !p.hecho).length; }

  diasPara(fecha: string): number {
    const hoy = new Date();
    const d = new Date(fecha + 'T00:00:00');
    const base = new Date(hoy.getFullYear(), hoy.getMonth(), hoy.getDate());
    return Math.round((d.getTime() - base.getTime()) / 86400000);
  }

  etiqueta(p: Pendiente): string {
    if (!p.vence) return '';
    if (p.hecho) return p.vence;
    const d = this.diasPara(p.vence);
    if (d < 0) return 'vencido';
    if (d === 0) return 'hoy';
    if (d === 1) return 'mañana';
    return 'en ' + d + ' d';
  }

  private cargar(): void {
    this.api.pendientes().subscribe({
      next: (l) => this.lista.set(l),
      error: () => this.error.set('No se pudo conectar con el backend.'),
    });
  }

  alternar(p: Pendiente): void {
    const hecho = !p.hecho;
    this.api.guardarPendiente(p.id, { hecho }).subscribe({
      next: () => this.cargar(),
      error: () => this.error.set('No se pudo guardar.'),
    });
  }

  agregar(): void {
    const t = this.texto.trim();
    if (!t) return;
    this.api.crearPendiente(t, this.vence || null).subscribe({
      next: () => { this.texto = ''; this.vence = ''; this.cargar(); },
      error: () => this.error.set('No se pudo agregar.'),
    });
  }

  quitar(p: Pendiente): void {
    this.api.borrarPendiente(p.id).subscribe({
      next: () => this.cargar(),
      error: () => this.error.set('No se pudo quitar.'),
    });
  }
}
