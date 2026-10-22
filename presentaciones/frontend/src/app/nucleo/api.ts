import { HttpClient, HttpHeaders, HttpResponse } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { Avance, Corte, CorteEnLista, Estado, Lectura, Paso, Proyecto } from './modelos';

/** Cambia esto si el backend corre en otro lado. */
export const BASE = 'http://localhost:3001/api';
/** Si pusiste APP_API_KEY en el backend, la misma va aquí. */
export const LLAVE = '';

@Injectable({ providedIn: 'root' })
export class Api {
  private http = inject(HttpClient);

  private get opciones() {
    return LLAVE ? { headers: new HttpHeaders({ 'X-API-Key': LLAVE }) } : {};
  }

  cortes(): Observable<CorteEnLista[]> {
    return this.http.get<CorteEnLista[]>(`${BASE}/cortes`, this.opciones);
  }

  corte(id: string): Observable<Corte> {
    return this.http.get<Corte>(`${BASE}/cortes/${id}`, this.opciones);
  }

  crearCorte(c: { fecha: string; siguiente: string | null; arrastrar: boolean }): Observable<Corte> {
    return this.http.post<Corte>(`${BASE}/cortes`, c, this.opciones);
  }

  guardarCorte(id: string, c: { fecha: string; titulo: string; presenta: string; siguiente: string | null }): Observable<Corte> {
    return this.http.put<Corte>(`${BASE}/cortes/${id}`, c, this.opciones);
  }

  borrarCorte(id: string): Observable<unknown> {
    return this.http.delete(`${BASE}/cortes/${id}`, this.opciones);
  }

  leer(texto: string, proyecto: string | null): Observable<Lectura> {
    return this.http.post<Lectura>(`${BASE}/leer`, { texto, proyecto }, this.opciones);
  }

  sumar(id: string, l: Pick<Lectura, 'bloques' | 'pasos'>): Observable<Corte> {
    return this.http.post<Corte>(`${BASE}/cortes/${id}/sumar`, l, this.opciones);
  }

  /** `estado: ''` quita el estado. */
  guardarAvance(id: string, cambio: { texto?: string; estado?: Estado | '' }): Observable<Avance> {
    return this.http.put<Avance>(`${BASE}/avances/${id}`, cambio, this.opciones);
  }

  borrarAvance(id: string): Observable<unknown> {
    return this.http.delete(`${BASE}/avances/${id}`, this.opciones);
  }

  crearPaso(corte: string, proyecto_id: string | null, texto: string): Observable<Paso> {
    return this.http.post<Paso>(`${BASE}/cortes/${corte}/pasos`, { proyecto_id, texto }, this.opciones);
  }

  guardarPaso(id: string, texto: string): Observable<Paso> {
    return this.http.put<Paso>(`${BASE}/pasos/${id}`, { texto }, this.opciones);
  }

  borrarPaso(id: string): Observable<unknown> {
    return this.http.delete(`${BASE}/pasos/${id}`, this.opciones);
  }

  proyectos(): Observable<Proyecto[]> {
    return this.http.get<Proyecto[]>(`${BASE}/proyectos`, this.opciones);
  }

  crearProyecto(nombre: string, equipo: string): Observable<Proyecto> {
    return this.http.post<Proyecto>(`${BASE}/proyectos`, { nombre, equipo }, this.opciones);
  }

  guardarProyecto(id: string, cambio: Partial<Proyecto>): Observable<Proyecto> {
    return this.http.put<Proyecto>(`${BASE}/proyectos/${id}`, cambio, this.opciones);
  }

  borrarProyecto(id: string): Observable<unknown> {
    return this.http.delete(`${BASE}/proyectos/${id}`, this.opciones);
  }

  documento(id: string, tipo: 'pdf' | 'pptx'): Observable<HttpResponse<Blob>> {
    return this.http.get(`${BASE}/cortes/${id}/${tipo}`, { ...this.opciones, responseType: 'blob', observe: 'response' });
  }
}

/** Baja el archivo con el nombre que manda el backend. */
export function guardarArchivo(r: HttpResponse<Blob>, porOmision: string): void {
  const disp = r.headers.get('Content-Disposition') ?? '';
  const nombre = /filename="([^"]+)"/.exec(disp)?.[1] ?? porOmision;
  const url = URL.createObjectURL(r.body!);
  const a = document.createElement('a');
  a.href = url;
  a.download = nombre;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 2000);
}

/** "2026-09-24" → "24/09/2026". */
export function fechaCorta(iso: string | null): string {
  if (!iso) return '';
  const [a, m, d] = iso.split('-');
  return `${d}/${m}/${a}`;
}

/** Hoy en la zona del navegador, como "AAAA-MM-DD". */
export function hoyIso(): string {
  const d = new Date();
  return new Date(d.getTime() - d.getTimezoneOffset() * 60000).toISOString().slice(0, 10);
}

export function sumarDias(iso: string, dias: number): string {
  const d = new Date(iso + 'T12:00:00');
  d.setDate(d.getDate() + dias);
  return d.toISOString().slice(0, 10);
}
