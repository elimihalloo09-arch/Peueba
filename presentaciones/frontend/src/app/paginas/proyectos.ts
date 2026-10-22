import { Component, OnInit, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';

import { Api } from '../nucleo/api';
import { Proyecto } from '../nucleo/modelos';

@Component({
  selector: 'app-proyectos',
  standalone: true,
  imports: [FormsModule],
  template: `
    <div class="sec-head">
      <h2>Proyectos</h2>
      <span class="hint">El orden es el de las diapositivas. Se dan de alta solos al sumar avances.</span>
    </div>

    <div class="tarjeta lista">
      @for (p of lista(); track p.id) {
        <div class="fila" [class.inactivo]="!p.activo">
          <input class="orden" type="number" [ngModel]="p.orden" (change)="guardar(p, { orden: +$any($event.target).value })"
                 aria-label="Orden">
          <input type="text" [ngModel]="p.nombre" (change)="guardar(p, { nombre: $any($event.target).value })"
                 aria-label="Nombre del proyecto">
          <input type="text" [ngModel]="p.equipo" placeholder="Equipo" (change)="guardar(p, { equipo: $any($event.target).value })"
                 aria-label="Equipo">
          <label class="activo" [title]="p.activo ? 'Activo' : 'Inactivo: queda al final de la lista y conserva su historia'">
            <input type="checkbox" [ngModel]="p.activo" (ngModelChange)="guardar(p, { activo: $event })"> activo
          </label>
          <button type="button" class="quitar" aria-label="Borrar proyecto" (click)="quitar(p)">×</button>
        </div>
      } @empty {
        <p class="cargando vacio">Sin proyectos todavía.</p>
      }
    </div>

    <div class="nuevo">
      <input type="text" [(ngModel)]="nombre" placeholder="Nombre del proyecto" aria-label="Nombre del proyecto nuevo">
      <input type="text" [(ngModel)]="equipo" placeholder="Equipo (opcional)" aria-label="Equipo del proyecto nuevo"
             (keyup.enter)="agregar()">
      <button class="btn" type="button" (click)="agregar()" [disabled]="!nombre.trim()">Agregar</button>
    </div>
    @if (error()) { <p class="error">{{ error() }}</p> }
  `,
  styles: [`
    .lista{ padding:0; }
    .fila{ display:grid; grid-template-columns:64px 1.2fr 1fr auto 32px; gap:8px; align-items:center;
           padding:10px 14px; border-bottom:1px solid var(--hair-soft); }
    .fila:last-child{ border-bottom:0; }
    .fila.inactivo input[type=text]{ color:var(--ink-3); }
    .orden{ text-align:right; font-variant-numeric:tabular-nums; }
    .activo{ display:flex; gap:6px; align-items:center; font-size:12.5px; color:var(--ink-3); }
    .activo input{ width:auto; accent-color:var(--guinda); }
    .nuevo{ display:grid; grid-template-columns:1.2fr 1fr auto; gap:8px; margin-top:14px; }
    .vacio{ padding:18px; }
    @media (max-width:640px){
      .fila{ grid-template-columns:56px 1fr 32px; }
      .fila input[placeholder=Equipo]{ grid-column:2 / 3; }
      .activo{ grid-column:2 / 3; }
      .nuevo{ grid-template-columns:1fr; }
    }
  `],
})
export class ProyectosComponent implements OnInit {
  private api = inject(Api);
  lista = signal<Proyecto[]>([]);
  error = signal('');
  nombre = '';
  equipo = '';

  ngOnInit(): void { this.cargar(); }

  private cargar(): void {
    this.api.proyectos().subscribe({
      next: (l) => this.lista.set(l),
      error: () => this.error.set('No se pudo conectar con el backend.'),
    });
  }

  guardar(p: Proyecto, cambio: Partial<Proyecto>): void {
    this.error.set('');
    this.api.guardarProyecto(p.id, cambio).subscribe({
      next: () => this.cargar(),
      error: (e) => { this.error.set(e?.error?.error ?? 'No se pudo guardar.'); this.cargar(); },
    });
  }

  agregar(): void {
    const n = this.nombre.trim();
    if (!n) return;
    this.api.crearProyecto(n, this.equipo.trim()).subscribe({
      next: () => { this.nombre = ''; this.equipo = ''; this.cargar(); },
      error: (e) => this.error.set(e?.error?.error ?? 'No se pudo agregar.'),
    });
  }

  quitar(p: Proyecto): void {
    if (!confirm(`¿Borrar «${p.nombre}»? Se borran también sus avances en todos los cortes. Si solo ya no se presenta, mejor desmárcalo como activo.`)) return;
    this.api.borrarProyecto(p.id).subscribe({
      next: () => this.cargar(),
      error: () => this.error.set('No se pudo borrar.'),
    });
  }
}
