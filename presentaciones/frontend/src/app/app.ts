import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-raiz',
  standalone: true,
  imports: [RouterOutlet, RouterLink, RouterLinkActive],
  template: `
    <header class="banda">
      <div class="marco">
        <p class="kicker">Seguimiento de proyectos</p>
        <h1>Avance de proyectos</h1>
        <nav>
          <a routerLink="/cortes" routerLinkActive="activo">Cortes</a>
          <a routerLink="/proyectos" routerLinkActive="activo">Proyectos</a>
        </nav>
      </div>
    </header>
    <main class="marco"><router-outlet /></main>
  `,
  styles: [`
    .banda{ background:#691B32; color:#fff; border-bottom:4px solid #AE8320; }
    .banda .marco{ padding-top:22px; padding-bottom:0; }
    .banda .kicker{ color:#E8D9DF; }
    .marco{ max-width:960px; margin:0 auto; padding:24px 16px 64px; }
    h1{ font-size:clamp(24px,5vw,32px); font-weight:800; letter-spacing:-.02em; line-height:1.1; margin:4px 0 0; }
    nav{ display:flex; gap:4px; margin-top:16px; }
    nav a{ font-size:13.5px; font-weight:600; text-decoration:none; color:#E8D9DF;
           padding:9px 14px 11px; border-radius:8px 8px 0 0; }
    nav a:hover{ color:#fff; background:rgba(255,255,255,.08); }
    nav a.activo{ background:var(--ground); color:#A02142; }
    @media (prefers-color-scheme: dark){ nav a.activo{ color:var(--guinda); } }
  `],
})
export class AppComponent {}
