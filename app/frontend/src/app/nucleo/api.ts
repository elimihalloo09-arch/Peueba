import { HttpClient, HttpHeaders, HttpParams } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';

import { Acreedor, CuentaPersona, Pendiente, Resumen, Simulacion } from './modelos';

/** Cambia esto si el backend corre en otro lado. */
export const BASE = 'http://localhost:3000/api';
/** Si pusiste APP_API_KEY en el backend, la misma va aquí. */
export const LLAVE = '';

@Injectable({ providedIn: 'root' })
export class Api {
  private http = inject(HttpClient);

  private get opciones() {
    return LLAVE ? { headers: new HttpHeaders({ 'X-API-Key': LLAVE }) } : {};
  }

  resumen(): Observable<Resumen> {
    return this.http.get<Resumen>(`${BASE}/resumen`, this.opciones);
  }

  acreedores(): Observable<Acreedor[]> {
    return this.http.get<Acreedor[]>(`${BASE}/acreedores`, this.opciones);
  }

  guardarAcreedor(id: string, cambio: Partial<Acreedor>): Observable<Acreedor> {
    return this.http.put<Acreedor>(`${BASE}/acreedores/${id}`, cambio, this.opciones);
  }

  crearAcreedor(datos: Partial<Acreedor>): Observable<Acreedor> {
    return this.http.post<Acreedor>(`${BASE}/acreedores`, datos, this.opciones);
  }

  borrarAcreedor(id: string): Observable<unknown> {
    return this.http.delete(`${BASE}/acreedores/${id}`, this.opciones);
  }

  pendientes(): Observable<Pendiente[]> {
    return this.http.get<Pendiente[]>(`${BASE}/pendientes`, this.opciones);
  }

  crearPendiente(texto: string, vence: string | null): Observable<Pendiente> {
    return this.http.post<Pendiente>(`${BASE}/pendientes`, { texto, vence }, this.opciones);
  }

  guardarPendiente(id: string, cambio: Partial<Pendiente>): Observable<Pendiente> {
    return this.http.put<Pendiente>(`${BASE}/pendientes/${id}`, cambio, this.opciones);
  }

  borrarPendiente(id: string): Observable<unknown> {
    return this.http.delete(`${BASE}/pendientes/${id}`, this.opciones);
  }

  movimientos(): Observable<CuentaPersona[]> {
    return this.http.get<CuentaPersona[]>(`${BASE}/movimientos`, this.opciones);
  }

  crearMovimiento(m: {
    persona: string; concepto: string; monto: number; tipo: 'entrega' | 'pago';
  }): Observable<unknown> {
    return this.http.post(`${BASE}/movimientos`, m, this.opciones);
  }

  plan(p: {
    capacidad: number; metodo: string; apoyo_mensual: number; meses_apoyo: number; sin_quitas: boolean;
  }): Observable<Simulacion> {
    const params = new HttpParams()
      .set('capacidad', p.capacidad)
      .set('metodo', p.metodo)
      .set('apoyo_mensual', p.apoyo_mensual)
      .set('meses_apoyo', p.meses_apoyo)
      .set('sin_quitas', p.sin_quitas);
    return this.http.get<Simulacion>(`${BASE}/plan`, { params, ...this.opciones });
  }
}
