import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-raiz',
  standalone: true,
  imports: [RouterOutlet, RouterLink, RouterLinkActive],
  template: `
    <div class="marco">
      <header>
        <p class="kicker">Finanzas personales</p>
        <h1>Finanzas</h1>
        <nav>
          <a routerLink="/resumen" routerLinkActive="activo">Resumen</a>
          <a routerLink="/acreedores" routerLinkActive="activo">Acreedores</a>
          <a routerLink="/plan" routerLinkActive="activo">Plan</a>
          <a routerLink="/pendientes" routerLinkActive="activo">Pendientes</a>
        </nav>
      </header>
      <main><router-outlet /></main>
    </div>
  `,
  styles: [`
    .marco{ max-width:900px; margin:0 auto; padding:24px 16px 64px; }
    h1{ font-size:clamp(28px,6vw,38px); font-weight:800; font-stretch:110%;
        letter-spacing:-.03em; line-height:1; margin:2px 0 0; }
    nav{ display:flex; gap:6px; margin-top:18px; flex-wrap:wrap; }
    nav a{ font-size:13.5px; text-decoration:none; color:var(--ink-2);
           border:1px solid var(--hair); border-radius:999px; padding:7px 14px; }
    nav a:hover{ color:var(--ink); border-color:var(--ink-3); }
    nav a.activo{ background:var(--ink); border-color:var(--ink); color:var(--surface); font-weight:600; }
    main{ margin-top:26px; }
  `],
})
export class AppComponent {}
