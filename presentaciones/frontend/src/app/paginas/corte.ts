import { Component, OnDestroy, OnInit, computed, inject, input, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { DomSanitizer, SafeResourceUrl } from '@angular/platform-browser';
import { Router, RouterLink } from '@angular/router';

import { Api, fechaCorta, guardarArchivo } from '../nucleo/api';
import { Avance, Corte, ESTADOS, Estado, Lectura, Paso, Proyecto } from '../nucleo/modelos';

const EJEMPLO = `PROYECTO: NOMBRE DEL PROYECTO
Equipo que reporta
• Lo que se avanzó esta semana. Estado: En proceso.
• Lo que está detenido y por qué. Estado: Pendiente.

Otro proyecto - Equipo
- Se liberó el módulo (concluido)

Próximos pasos:
- Nombre del proyecto: lo que sigue`;

@Component({
  selector: 'app-corte',
  standalone: true,
  imports: [FormsModule, RouterLink],
  template: `
    <a routerLink="/cortes" class="volver">‹ Cortes</a>

    @if (corte(); as c) {
      <div class="cabeza">
        <div>
          <p class="kicker">Corte al {{ fechaCorta(c.fecha) }}</p>
          <h2 class="titulo">{{ c.titulo }}</h2>
          <p class="cuentas">
            {{ c.proyectos.length }} {{ c.proyectos.length === 1 ? 'proyecto' : 'proyectos' }} · {{ totalPuntos() }} puntos
            @for (e of ESTADOS; track e.clave) {
              @if (cuenta(e.clave); as n) { <span class="estado" [class]="e.clave">{{ n }} {{ e.etiqueta.toLowerCase() }}</span> }
            }
          </p>
        </div>
        <div class="acciones">
          <button class="btn" type="button" (click)="descargar('pdf')" [disabled]="!totalPuntos() || bajando()">PDF</button>
          <button class="btn" type="button" (click)="descargar('pptx')" [disabled]="!totalPuntos() || bajando()">PowerPoint</button>
          <button class="btn ghost" type="button" (click)="alternarVista()" [disabled]="!totalPuntos()">
            {{ vista() ? 'Cerrar vista' : 'Vista previa' }}
          </button>
        </div>
      </div>

      @if (vista()) {
        <iframe class="vista" [src]="vista()" title="Vista previa de la presentación"></iframe>
      }

      <!-- 1. Pegar y leer lo que mandó el equipo -->
      <section class="tarjeta">
        <div class="sec-head">
          <h2>Sumar avances</h2>
          <span class="hint">Pega el texto tal como lo mandó el equipo</span>
        </div>
        @if (!lectura()) {
          <textarea rows="9" [(ngModel)]="texto" [placeholder]="ejemplo" aria-label="Texto del equipo"></textarea>
          <div class="fila-leer">
            <div class="omision">
              <label class="lab" for="omision">Si el texto no dice de qué proyecto es</label>
              <input id="omision" type="text" list="catalogo" [(ngModel)]="proyectoPorOmision" placeholder="Proyecto (opcional)">
              <datalist id="catalogo">
                @for (p of catalogo(); track p.id) { <option [value]="p.nombre"></option> }
              </datalist>
            </div>
            <button class="btn" type="button" (click)="leer()" [disabled]="!texto.trim() || leyendo()">Leer texto</button>
          </div>
        } @else {
          @let l = lectura()!;
          @for (a of l.avisos; track a) { <p class="aviso">{{ a }}</p> }
          @for (b of l.bloques; track $index; let bi = $index) {
            <div class="bloque">
              <div class="bloque-cabeza">
                <input class="nombre" type="text" list="catalogo2" [(ngModel)]="b.proyecto" aria-label="Proyecto">
                @if (b.nuevo) { <span class="nuevo" title="No está en el catálogo: se dará de alta">nuevo</span> }
                <input class="equipo" type="text" [(ngModel)]="b.equipo" placeholder="Equipo" aria-label="Equipo">
                <button type="button" class="quitar" aria-label="Quitar este proyecto" (click)="l.bloques.splice(bi, 1)">×</button>
              </div>
              @for (p of b.puntos; track $index; let pi = $index) {
                <div class="punto">
                  <textarea rows="2" [(ngModel)]="p.texto" aria-label="Punto de avance"></textarea>
                  <div class="lado">
                    <select [(ngModel)]="p.estado" (ngModelChange)="p.inferido = false" aria-label="Estado">
                      <option [ngValue]="null">Sin estado</option>
                      @for (e of ESTADOS; track e.clave) { <option [ngValue]="e.clave">{{ e.etiqueta }}</option> }
                    </select>
                    @if (p.inferido) { <span class="adivinado" title="El texto no lo decía: revísalo">adivinado</span> }
                  </div>
                  <button type="button" class="quitar" aria-label="Quitar punto" (click)="b.puntos.splice(pi, 1)">×</button>
                </div>
              }
            </div>
          } @empty {
            <p class="cargando">No encontré puntos de avance en el texto.</p>
          }
          @if (l.pasos.length) {
            <p class="lab sub">Próximos pasos</p>
            @for (p of l.pasos; track $index; let i = $index) {
              <div class="paso-leido">
                <span class="de">{{ p.proyecto ?? 'General' }}</span>
                <input type="text" [(ngModel)]="p.texto" aria-label="Paso">
                <button type="button" class="quitar" aria-label="Quitar paso" (click)="l.pasos.splice(i, 1)">×</button>
              </div>
            }
          }
          <div class="fila-sumar">
            <button class="btn ghost" type="button" (click)="lectura.set(null)">Volver al texto</button>
            <button class="btn" type="button" (click)="sumar()" [disabled]="sumando() || !puntosLeidos()">
              Sumar {{ puntosLeidos() }} {{ puntosLeidos() === 1 ? 'punto' : 'puntos' }} al corte
            </button>
          </div>
        }
      </section>

      <!-- 2. Lo que ya lleva el corte -->
      <div class="sec-head separado">
        <h2>Contenido de la presentación</h2>
        <span class="hint">Una diapositiva por proyecto. Los cambios se guardan solos.</span>
      </div>
      @for (p of c.proyectos; track p.id) {
        <section class="tarjeta proyecto">
          <h3>PROYECTO: {{ p.nombre.toUpperCase() }}</h3>
          @if (p.equipo) { <p class="equipo-txt">{{ p.equipo }}</p> }
          @for (a of p.avances; track a.id) {
            <div class="avance">
              <span class="vineta" aria-hidden="true">•</span>
              <textarea rows="1" class="txt" [ngModel]="a.texto" (change)="guardarTexto(a, $any($event.target).value)"
                        aria-label="Texto del avance"></textarea>
              <select [class]="'sel ' + (a.estado ?? '')" [ngModel]="a.estado ?? ''" (ngModelChange)="guardarEstado(a, $event)"
                      aria-label="Estado">
                <option value="">Sin estado</option>
                @for (e of ESTADOS; track e.clave) { <option [value]="e.clave">{{ e.etiqueta }}</option> }
              </select>
              <button type="button" class="quitar" aria-label="Quitar avance" (click)="quitarAvance(a)">×</button>
            </div>
          }
        </section>
      } @empty {
        <p class="cargando">El corte está vacío: pega arriba el primer texto.</p>
      }

      <section class="tarjeta proyecto">
        <h3>Próximos pasos</h3>
        @if (!c.pasos.length && totalPuntos()) {
          <p class="equipo-txt">Si no escribes pasos, la diapositiva lista lo que sigue en proceso o pendiente de cada proyecto.</p>
        }
        @for (p of c.pasos; track p.id) {
          <div class="avance">
            <span class="vineta" aria-hidden="true">•</span>
            <span class="txt-paso">
              @if (nombreDe(p.proyecto_id); as n) { <b>{{ n }}:</b> }
              <input type="text" [ngModel]="p.texto" (change)="guardarPaso(p, $any($event.target).value)" aria-label="Paso">
            </span>
            <button type="button" class="quitar" aria-label="Quitar paso" (click)="quitarPaso(p)">×</button>
          </div>
        }
        <div class="nuevo-paso">
          <select [(ngModel)]="pasoProyecto" aria-label="Proyecto del paso">
            <option [ngValue]="null">General</option>
            @for (p of c.proyectos; track p.id) { <option [ngValue]="p.id">{{ p.nombre }}</option> }
          </select>
          <input type="text" [(ngModel)]="pasoTexto" placeholder="Lo que sigue" (keyup.enter)="agregarPaso()" aria-label="Nuevo paso">
          <button class="btn ghost" type="button" (click)="agregarPaso()" [disabled]="!pasoTexto.trim()">Agregar</button>
        </div>
      </section>

      <!-- 3. Portada -->
      <section class="tarjeta">
        <div class="sec-head">
          <h2>Portada</h2>
          <span class="hint">Lo que dice la primera y la última diapositiva</span>
        </div>
        <div class="portada">
          <div class="ancho"><label class="lab" for="titulo">Título</label><input id="titulo" type="text" [(ngModel)]="titulo"></div>
          <div class="ancho"><label class="lab" for="presenta">Presenta</label><input id="presenta" type="text" [(ngModel)]="presenta" placeholder="Nombre y cargo"></div>
          <div><label class="lab" for="f">Corte al</label><input id="f" type="date" [(ngModel)]="fecha"></div>
          <div><label class="lab" for="s">Siguiente corte</label><input id="s" type="date" [(ngModel)]="siguiente"></div>
        </div>
        <div class="fila-sumar">
          <button class="btn ghost peligro" type="button" (click)="borrarCorte()">Borrar corte</button>
          <button class="btn" type="button" (click)="guardarPortada()" [disabled]="!cambioPortada()">Guardar portada</button>
        </div>
      </section>
    } @else if (!error()) {
      <p class="cargando">Cargando…</p>
    }
    @if (error()) { <p class="error" role="alert">{{ error() }}</p> }

    <datalist id="catalogo2">
      @for (p of catalogo(); track p.id) { <option [value]="p.nombre"></option> }
    </datalist>
  `,
  styles: [`
    .volver{ font-size:13px; font-weight:600; text-decoration:none; }
    .cabeza{ display:flex; justify-content:space-between; align-items:flex-end; gap:16px; flex-wrap:wrap; margin:10px 0 18px; }
    .titulo{ font-size:24px; color:var(--guinda); }
    .cuentas{ margin:6px 0 0; display:flex; gap:8px; align-items:center; flex-wrap:wrap; font-size:13px; color:var(--ink-3); }
    .acciones{ display:flex; gap:8px; flex-wrap:wrap; }
    .vista{ width:100%; aspect-ratio:16/10; border:1px solid var(--hair); border-radius:8px; background:#fff; margin-bottom:18px; }
    section.tarjeta{ margin-bottom:16px; }
    .fila-leer{ display:flex; gap:12px; align-items:flex-end; margin-top:12px; flex-wrap:wrap; }
    .omision{ flex:1; min-width:220px; }
    .aviso{ margin:0 0 10px; padding:9px 12px; border-left:3px solid var(--dorado); background:var(--sunken);
            font-size:13px; color:var(--ink-2); border-radius:0 6px 6px 0; }
    .bloque{ border:1px solid var(--hair-soft); border-radius:8px; padding:12px; margin-bottom:12px; }
    .bloque-cabeza{ display:grid; grid-template-columns:1.3fr auto 1fr 32px; gap:8px; align-items:center; margin-bottom:10px; }
    .bloque-cabeza .nombre{ font-weight:700; color:var(--guinda); }
    .nuevo{ font-size:10.5px; font-weight:700; text-transform:uppercase; letter-spacing:.08em; color:var(--dorado);
            border:1px solid var(--dorado); border-radius:999px; padding:2px 8px; }
    .punto{ display:grid; grid-template-columns:1fr 150px 32px; gap:8px; align-items:start; margin-top:6px; }
    .lado{ display:flex; flex-direction:column; gap:4px; }
    .adivinado{ font-size:11px; color:var(--dorado); font-weight:600; }
    .sub{ margin:16px 0 8px; }
    .paso-leido{ display:grid; grid-template-columns:auto 1fr 32px; gap:8px; align-items:center; margin-top:6px; }
    .de{ font-size:12.5px; font-weight:700; color:var(--guinda); }
    .fila-sumar{ display:flex; justify-content:space-between; gap:8px; margin-top:14px; flex-wrap:wrap; }
    .separado{ margin-top:28px; }
    .proyecto h3{ font-size:16px; color:var(--guinda); }
    .equipo-txt{ margin:2px 0 8px; color:var(--ink-3); font-size:13px; }
    .avance{ display:grid; grid-template-columns:14px 1fr 140px 32px; gap:8px; align-items:start; padding:6px 0;
             border-bottom:1px solid var(--hair-soft); }
    .avance:last-of-type{ border-bottom:0; }
    .vineta{ color:var(--guinda); font-weight:700; padding-top:8px; }
    .avance .txt{ background:transparent; field-sizing:content; min-height:38px; resize:none; }
    .avance .txt:hover{ background:var(--sunken); }
    .sel{ font-size:13px; font-weight:600; }
    .sel.concluido{ color:var(--concluido); }
    .sel.en_proceso{ color:var(--proceso); }
    .sel.pendiente{ color:var(--pendiente); }
    .avance:has(.txt-paso){ grid-template-columns:14px 1fr 32px; }
    .txt-paso{ display:flex; gap:6px; align-items:center; flex-wrap:wrap; }
    .txt-paso input{ flex:1; min-width:200px; }
    .txt-paso b{ color:var(--guinda); white-space:nowrap; }
    .txt-paso input{ background:transparent; }
    .nuevo-paso{ display:grid; grid-template-columns:180px 1fr auto; gap:8px; margin-top:12px; }
    .portada{ display:grid; grid-template-columns:1fr 1fr; gap:12px; }
    .peligro{ color:var(--pendiente); }
    @media (max-width:640px){
      .bloque-cabeza{ grid-template-columns:1fr 32px; }
      .bloque-cabeza .equipo{ grid-column:1 / 2; }
      .punto{ grid-template-columns:1fr 32px; }
      .punto .lado{ grid-column:1 / 2; }
      .avance{ grid-template-columns:14px 1fr 32px; }
      .avance select{ grid-column:2 / 3; }
      .nuevo-paso, .portada{ grid-template-columns:1fr; }
    }
  `],
})
export class CorteComponent implements OnInit, OnDestroy {
  private api = inject(Api);
  private router = inject(Router);
  private sanitizer = inject(DomSanitizer);

  /** Viene de la ruta /cortes/:id. */
  id = input.required<string>();

  corte = signal<Corte | null>(null);
  catalogo = signal<Proyecto[]>([]);
  lectura = signal<Lectura | null>(null);
  vista = signal<SafeResourceUrl | null>(null);
  error = signal('');
  leyendo = signal(false);
  sumando = signal(false);
  bajando = signal(false);

  texto = '';
  proyectoPorOmision = '';
  pasoProyecto: string | null = null;
  pasoTexto = '';
  titulo = '';
  presenta = '';
  fecha = '';
  siguiente = '';
  private urlVista: string | null = null;

  readonly ESTADOS = ESTADOS;
  readonly ejemplo = EJEMPLO;
  fechaCorta = fechaCorta;

  totalPuntos = computed(() => this.corte()?.proyectos.reduce((s, p) => s + p.avances.length, 0) ?? 0);

  ngOnInit(): void {
    this.cargar();
    this.api.proyectos().subscribe({ next: (l) => this.catalogo.set(l) });
  }

  ngOnDestroy(): void {
    if (this.urlVista) URL.revokeObjectURL(this.urlVista);
  }

  private cargar(): void {
    this.api.corte(this.id()).subscribe({
      next: (c) => this.poner(c),
      error: () => this.error.set('No se pudo cargar el corte.'),
    });
  }

  private poner(c: Corte): void {
    this.corte.set(c);
    this.titulo = c.titulo;
    this.presenta = c.presenta;
    this.fecha = c.fecha;
    this.siguiente = c.siguiente ?? '';
    if (this.vista()) this.refrescarVista();
  }

  cuenta(e: Estado): number {
    return this.corte()?.proyectos.reduce((s, p) => s + p.avances.filter((a) => a.estado === e).length, 0) ?? 0;
  }

  puntosLeidos(): number {
    return this.lectura()?.bloques.reduce((s, b) => s + b.puntos.filter((p) => p.texto.trim()).length, 0) ?? 0;
  }

  nombreDe(id: string | null): string | null {
    const p = this.corte()?.proyectos.find((x) => x.id === id);
    return p ? p.nombre.split(' (')[0] : null;
  }

  cambioPortada(): boolean {
    const c = this.corte();
    return !!c && !!this.fecha && !!this.titulo.trim() &&
      (c.titulo !== this.titulo || c.presenta !== this.presenta || c.fecha !== this.fecha || (c.siguiente ?? '') !== this.siguiente);
  }

  leer(): void {
    this.error.set('');
    this.leyendo.set(true);
    this.api.leer(this.texto, this.proyectoPorOmision.trim() || null).subscribe({
      next: (l) => { this.lectura.set(l); this.leyendo.set(false); },
      error: (e) => { this.error.set(e?.error?.error ?? 'No se pudo leer el texto.'); this.leyendo.set(false); },
    });
  }

  sumar(): void {
    const l = this.lectura();
    if (!l) return;
    this.sumando.set(true);
    this.api.sumar(this.id(), { bloques: l.bloques, pasos: l.pasos }).subscribe({
      next: (c) => {
        this.poner(c);
        this.lectura.set(null);
        this.texto = '';
        this.sumando.set(false);
        this.api.proyectos().subscribe({ next: (x) => this.catalogo.set(x) });
      },
      error: (e) => { this.error.set(e?.error?.error ?? 'No se pudo sumar.'); this.sumando.set(false); },
    });
  }

  guardarTexto(a: Avance, texto: string): void {
    if (!texto.trim() || texto === a.texto) return;
    this.api.guardarAvance(a.id, { texto }).subscribe({ next: () => this.cargar(), error: () => this.error.set('No se pudo guardar.') });
  }

  guardarEstado(a: Avance, estado: Estado | ''): void {
    this.api.guardarAvance(a.id, { estado }).subscribe({ next: () => this.cargar(), error: () => this.error.set('No se pudo guardar.') });
  }

  quitarAvance(a: Avance): void {
    this.api.borrarAvance(a.id).subscribe({ next: () => this.cargar(), error: () => this.error.set('No se pudo quitar.') });
  }

  agregarPaso(): void {
    const t = this.pasoTexto.trim();
    if (!t) return;
    this.api.crearPaso(this.id(), this.pasoProyecto, t).subscribe({
      next: () => { this.pasoTexto = ''; this.cargar(); },
      error: () => this.error.set('No se pudo agregar el paso.'),
    });
  }

  guardarPaso(p: Paso, texto: string): void {
    if (!texto.trim() || texto === p.texto) return;
    this.api.guardarPaso(p.id, texto).subscribe({ next: () => this.cargar(), error: () => this.error.set('No se pudo guardar.') });
  }

  quitarPaso(p: Paso): void {
    this.api.borrarPaso(p.id).subscribe({ next: () => this.cargar(), error: () => this.error.set('No se pudo quitar.') });
  }

  guardarPortada(): void {
    this.api.guardarCorte(this.id(), {
      fecha: this.fecha, titulo: this.titulo, presenta: this.presenta, siguiente: this.siguiente || null,
    }).subscribe({ next: (c) => this.poner(c), error: (e) => this.error.set(e?.error?.error ?? 'No se pudo guardar.') });
  }

  borrarCorte(): void {
    if (!confirm('¿Borrar este corte con todos sus avances? No se puede deshacer.')) return;
    this.api.borrarCorte(this.id()).subscribe({
      next: () => this.router.navigate(['/cortes']),
      error: () => this.error.set('No se pudo borrar.'),
    });
  }

  descargar(tipo: 'pdf' | 'pptx'): void {
    this.bajando.set(true);
    this.api.documento(this.id(), tipo).subscribe({
      next: (r) => { guardarArchivo(r, `avance.${tipo}`); this.bajando.set(false); },
      error: () => { this.error.set('No se pudo generar el archivo.'); this.bajando.set(false); },
    });
  }

  alternarVista(): void {
    if (this.vista()) {
      this.vista.set(null);
      return;
    }
    this.refrescarVista();
  }

  private refrescarVista(): void {
    this.api.documento(this.id(), 'pdf').subscribe({
      next: (r) => {
        if (this.urlVista) URL.revokeObjectURL(this.urlVista);
        this.urlVista = URL.createObjectURL(r.body!);
        this.vista.set(this.sanitizer.bypassSecurityTrustResourceUrl(this.urlVista));
      },
      error: () => this.error.set('No se pudo generar la vista previa.'),
    });
  }
}
