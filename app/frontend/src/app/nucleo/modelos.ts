/** Todo el dinero viaja en CENTAVOS, igual que en la base. */
export type Etapa =
  | 'sin_contactar' | 'contactado' | 'con_oferta'
  | 'convenio' | 'pagado' | 'finiquito';

export const ETAPAS: { valor: Etapa; texto: string }[] = [
  { valor: 'sin_contactar', texto: 'Sin contactar' },
  { valor: 'contactado', texto: 'Contactado' },
  { valor: 'con_oferta', texto: 'Con oferta' },
  { valor: 'convenio', texto: 'Convenio' },
  { valor: 'pagado', texto: 'Pagado' },
  { valor: 'finiquito', texto: 'Finiquito' },
];

export interface Acreedor {
  id: string;
  nombre: string;
  saldo_original: number;
  monto_a_pagar: number;
  tasa_anual: number;
  pago_mensual: number;
  etapa: Etapa;
  admite_quita: boolean;
  por_nomina: boolean;
  tiene_convenio: boolean;
  tiene_comprobante: boolean;
  tiene_finiquito: boolean;
  nota: string;
}

export interface Pendiente {
  id: string;
  texto: string;
  vence: string | null;
  hecho: boolean;
}

export interface MovimientoPersonal {
  id: string;
  persona: string;
  concepto: string;
  monto: number;
  tipo: 'entrega' | 'pago';
  fecha: string;
}

export interface CuentaPersona {
  persona: string;
  entregado: number;
  pagado: number;
  saldo: number;
  movimientos: MovimientoPersonal[];
}

export interface Resumen {
  ingreso_mensual: number;
  gasto_mensual: number;
  gasto_recortable: number;
  pagos_mensuales_deuda: number;
  deuda_total: number;
  deuda_liquidada: number;
  deuda_por_pagar: number;
  ahorro_por_quitas: number;
  capacidad_mensual: number;
  saldo_con_personas: number;
}

export interface DeudaLiquidada { nombre: string; mes: number; total_pagado: number; }
export interface PuntoCurva { mes: number; saldo: number; }
export interface Simulacion {
  meses: number;
  intereses: number;
  total_pagado: number;
  orden: DeudaLiquidada[];
  curva: PuntoCurva[];
  inalcanzable: boolean;
  atoradas: string[];
}

export function liquidada(e: Etapa): boolean {
  return e === 'pagado' || e === 'finiquito';
}
