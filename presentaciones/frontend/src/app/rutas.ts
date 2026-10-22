import { Routes } from '@angular/router';

export const RUTAS: Routes = [
  { path: '', pathMatch: 'full', redirectTo: 'cortes' },
  { path: 'cortes', loadComponent: () => import('./paginas/cortes').then((m) => m.CortesComponent) },
  { path: 'cortes/:id', loadComponent: () => import('./paginas/corte').then((m) => m.CorteComponent) },
  { path: 'proyectos', loadComponent: () => import('./paginas/proyectos').then((m) => m.ProyectosComponent) },
  { path: '**', redirectTo: 'cortes' },
];
