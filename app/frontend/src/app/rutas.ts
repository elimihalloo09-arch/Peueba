import { Routes } from '@angular/router';

export const RUTAS: Routes = [
  { path: '', pathMatch: 'full', redirectTo: 'resumen' },
  { path: 'resumen', loadComponent: () => import('./paginas/resumen').then((m) => m.ResumenComponent) },
  { path: 'acreedores', loadComponent: () => import('./paginas/acreedores').then((m) => m.AcreedoresComponent) },
  { path: 'plan', loadComponent: () => import('./paginas/plan').then((m) => m.PlanComponent) },
  { path: 'pendientes', loadComponent: () => import('./paginas/pendientes').then((m) => m.PendientesComponent) },
  { path: '**', redirectTo: 'resumen' },
];
