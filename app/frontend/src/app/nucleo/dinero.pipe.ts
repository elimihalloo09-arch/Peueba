import { Pipe, PipeTransform } from '@angular/core';

/** Recibe centavos y los muestra en pesos. */
@Pipe({ name: 'dinero', standalone: true })
export class DineroPipe implements PipeTransform {
  transform(centavos: number | null | undefined, decimales = 0): string {
    const v = (centavos ?? 0) / 100;
    return '$' + new Intl.NumberFormat('es-MX', {
      minimumFractionDigits: decimales,
      maximumFractionDigits: decimales,
    }).format(v);
  }
}
