import { bootstrapApplication } from '@angular/platform-browser';
import { provideHttpClient } from '@angular/common/http';
import { provideRouter, withComponentInputBinding } from '@angular/router';

import { AppComponent } from './app/app';
import { RUTAS } from './app/rutas';

bootstrapApplication(AppComponent, {
  providers: [provideHttpClient(), provideRouter(RUTAS, withComponentInputBinding())],
}).catch((e) => console.error(e));
