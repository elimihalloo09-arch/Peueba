import { Component, OnInit, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';

import { Api } from '../nucleo/api';
import { DineroPipe } from '../nucleo/dinero.pipe';
import { Acreedor, ETAPAS, Etapa, liquidada } from '../nucleo/modelos';

@Component({
  selector: 'app-acreedores',
  standalone: true,
  imports: [FormsModule, DineroPipe],
  template: `
    <div class="sec-head">
      <h2>Acreedores</h2>
      <span class="hint">{{ lista().length }} · toca la etapa en la que va cada uno</span>
    </div>

    @if (cargando()) { <p class="cargando">Cargando…</p> }
    @if (error()) { <p class="error">{{ error() }}</p> }

    @for (a of lista(); track a.id) {
      <div class="tarjeta acr" [class.lista-ok]="esLiquidada(a.etapa)" style="margin-bottom:12px">
        <div class="arriba">
          <div>
            <strong class="nom">{{ a.nombre }}</strong>
            <p class="pie">
              @if (a.por_nomina) { Se descuenta de nómina }
              @else if (!a.admite_quita) { Sin quita }
              @else if (a.saldo_original > a.monto_a_pagar) {
                @if (a.etapa === 'convenio' || esLiquidada(a.etapa)) {
                  Quita de {{ a.saldo_original - a.monto_a_pagar | dinero }}
                } @else {
                  Quita estimada de {{ a.saldo_original - a.monto_a_pagar | dinero }}, por negociar
                }
              } @else { Falta negociar la quita }
            </p>
          </div>
          <div class="montos">
            <p class="m" [style.color]="esLiquidada(a.etapa) ? 'var(--good)' : null">{{ a.monto_a_pagar | dinero }}</p>
            @if (a.saldo_original > a.monto_a_pagar) {
              <p class="antes">{{ a.saldo_original | dinero }}</p>
            }
          </div>
        </div>

        <div class="pasos">
          @for (e of etapas; track e.valor) {
            <button type="button" class="paso" [class.sel]="a.etapa === e.valor"
                    [attr.aria-pressed]="a.etapa === e.valor"
                    (click)="cambiarEtapa(a, e.valor)">{{ e.texto }}</button>
          }
        </div>

        <div class="docs">
          <button type="button" class="doc" [class.sel]="a.tiene_convenio"
                  (click)="alternar(a, 'tiene_convenio')">Convenio</button>
          <button type="button" class="doc" [class.sel]="a.tiene_comprobante"
                  (click)="alternar(a, 'tiene_comprobante')">Comprobante</button>
          <button type="button" class="doc" [class.sel]="a.tiene_finiquito"
                  (click)="alternar(a, 'tiene_finiquito')">Carta finiquito</button>
        </div>

        <div class="campos">
          <div>
            <label class="lab" [attr.for]="'saldo-' + a.id">Saldo original</label>
            <input type="number" [id]="'saldo-' + a.id" [ngModel]="a.saldo_original / 100"
                   (ngModelChange)="ponerPesos(a, 'saldo_original', $event)">
          </div>
          <div>
            <label class="lab" [attr.for]="'pago-' + a.id">Monto a pagar</label>
            <input type="number" [id]="'pago-' + a.id" [ngModel]="a.monto_a_pagar / 100"
                   (ngModelChange)="ponerPesos(a, 'monto_a_pagar', $event)">
          </div>
          <div>
            <label class="lab" [attr.for]="'quita-' + a.id">Quita %</label>
            <input type="number" [id]="'quita-' + a.id" [ngModel]="quita(a)"
                   (ngModelChange)="ponerQuita(a, $event)">
          </div>
        </div>

        <textarea rows="2" [ngModel]="a.nota" (ngModelChange)="ponerNota(a, $event)"
                  placeholder="Qué te dijeron, con quién hablaste, qué sigue"></textarea>
      </div>
    }
  `,
  styles: [`
    .acr.lista-ok{ border-left:4px solid var(--good); }
    .arriba{ display:flex; justify-content:space-between; gap:12px; align-items:flex-start; }
    .nom{ font-family:"Archivo",sans-serif; font-weight:700; font-size:16px; letter-spacing:-.015em; }
    .pie{ font-family:"IBM Plex Mono",monospace; font-size:11px; color:var(--ink-3); margin:4px 0 0; }
    .montos{ text-align:right; }
    .m{ font-family:"Archivo",sans-serif; font-weight:800; font-size:19px; letter-spacing:-.02em;
        font-variant-numeric:tabular-nums; margin:0; }
    .antes{ font-family:"IBM Plex Mono",monospace; font-size:11px; color:var(--ink-3);
            text-decoration:line-through; margin:2px 0 0; }
    .pasos{ display:flex; gap:4px; flex-wrap:wrap; margin-top:14px; }
    .paso{ font:inherit; font-size:11.5px; padding:6px 10px; border-radius:999px; cursor:pointer;
           border:1px solid var(--hair); background:transparent; color:var(--ink-3); flex:1 1 auto; }
    .paso.sel{ background:var(--ink); border-color:var(--ink); color:var(--surface); font-weight:600; }
    .docs{ display:flex; gap:8px; flex-wrap:wrap; margin-top:12px; }
    .doc{ font:inherit; font-size:11.5px; padding:6px 11px; border-radius:6px; cursor:pointer;
          border:1px dashed var(--hair); background:transparent; color:var(--ink-3); }
    .doc.sel{ border-style:solid; border-color:var(--good); color:var(--good); font-weight:600; }
    .campos{ display:grid; grid-template-columns:repeat(3,1fr); gap:10px; margin-top:14px; }
    textarea{ margin-top:12px; }
    @media (max-width:560px){ .campos{ grid-template-columns:1fr 1fr; } }
  `],
})
export class AcreedoresComponent implements OnInit {
  private api = inject(Api);
  etapas = ETAPAS;
  lista = signal<Acreedor[]>([]);
  cargando = signal(true);
  error = signal('');
  private temporizadores = new Map<string, ReturnType<typeof setTimeout>>();

  ngOnInit(): void { this.cargar(); }

  esLiquidada = liquidada;

  quita(a: Acreedor): number {
    if (a.saldo_original <= 0) return 0;
    return Math.round((1 - a.monto_a_pagar / a.saldo_original) * 100);
  }

  private cargar(): void {
    this.api.acreedores().subscribe({
      next: (l) => { this.lista.set(l); this.cargando.set(false); },
      error: () => { this.error.set('No se pudo conectar con el backend.'); this.cargando.set(false); },
    });
  }

  cambiarEtapa(a: Acreedor, etapa: Etapa): void {
    a.etapa = etapa;
    this.lista.set([...this.lista()]);
    this.guardar(a, { etapa });
  }

  alternar(a: Acreedor, campo: 'tiene_convenio' | 'tiene_comprobante' | 'tiene_finiquito'): void {
    a[campo] = !a[campo];
    // Con los tres documentos en mano, la cuenta está cerrada de verdad.
    const cambio: Partial<Acreedor> = { [campo]: a[campo] };
    if (a.tiene_convenio && a.tiene_comprobante && a.tiene_finiquito) {
      a.etapa = 'finiquito';
      cambio.etapa = 'finiquito';
    }
    this.lista.set([...this.lista()]);
    this.guardar(a, cambio);
  }

  ponerPesos(a: Acreedor, campo: 'saldo_original' | 'monto_a_pagar', pesos: number): void {
    const centavos = Math.max(0, Math.round((pesos || 0) * 100));
    a[campo] = centavos;
    this.guardar(a, { [campo]: centavos });
  }

  ponerQuita(a: Acreedor, porcentaje: number): void {
    const q = Math.min(100, Math.max(0, porcentaje || 0));
    a.monto_a_pagar = Math.round(a.saldo_original * (1 - q / 100));
    this.lista.set([...this.lista()]);
    this.guardar(a, { monto_a_pagar: a.monto_a_pagar });
  }

  ponerNota(a: Acreedor, nota: string): void {
    a.nota = nota;
    this.guardar(a, { nota });
  }

  /** Espera a que dejes de escribir antes de mandar al servidor. */
  private guardar(a: Acreedor, cambio: Partial<Acreedor>): void {
    clearTimeout(this.temporizadores.get(a.id));
    this.temporizadores.set(a.id, setTimeout(() => {
      this.api.guardarAcreedor(a.id, cambio).subscribe({
        error: () => this.error.set('No se pudo guardar el último cambio.'),
      });
    }, 500));
  }
}
