export type Estado = 'concluido' | 'en_proceso' | 'pendiente';

export const ESTADOS: { clave: Estado; etiqueta: string }[] = [
  { clave: 'concluido', etiqueta: 'Concluido' },
  { clave: 'en_proceso', etiqueta: 'En proceso' },
  { clave: 'pendiente', etiqueta: 'Pendiente' },
];

export function etiquetaEstado(e: Estado | null): string {
  return ESTADOS.find((x) => x.clave === e)?.etiqueta ?? 'Sin estado';
}

export interface Proyecto {
  id: string;
  nombre: string;
  equipo: string;
  orden: number;
  activo: boolean;
}

export interface CorteEnLista {
  id: string;
  fecha: string;
  titulo: string;
  presenta: string;
  siguiente: string | null;
  puntos: number;
  proyectos: number;
}

export interface Avance {
  id: string;
  proyecto_id: string;
  texto: string;
  estado: Estado | null;
  orden: number;
}

export interface Paso {
  id: string;
  proyecto_id: string | null;
  texto: string;
  orden: number;
}

export interface ProyectoEnCorte extends Proyecto {
  avances: Avance[];
}

export interface Corte {
  id: string;
  fecha: string;
  titulo: string;
  presenta: string;
  siguiente: string | null;
  proyectos: ProyectoEnCorte[];
  pasos: Paso[];
}

/** Lo que el backend entendió del texto, antes de sumarlo. */
export interface PuntoLeido {
  texto: string;
  estado: Estado | null;
  inferido: boolean;
}

export interface BloqueLeido {
  proyecto: string;
  equipo: string;
  nuevo: boolean;
  puntos: PuntoLeido[];
}

export interface PasoLeido {
  proyecto: string | null;
  texto: string;
}

export interface Lectura {
  bloques: BloqueLeido[];
  pasos: PasoLeido[];
  avisos: string[];
}
