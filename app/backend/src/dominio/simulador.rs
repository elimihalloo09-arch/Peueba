//! Simulador de liquidación de deudas.
//!
//! Dos métodos: avalancha (primero la tasa más alta, paga menos intereses) y
//! bola de nieve (primero el saldo más chico, da victorias antes). El pago
//! mensual que se libera al saldar una deuda rueda automáticamente a la
//! siguiente.
//!
//! Todo en centavos, con enteros. Los intereses se redondean al centavo.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Metodo {
    #[default]
    Avalancha,
    BolaDeNieve,
}

#[derive(Debug, Clone)]
pub struct DeudaSimulada {
    pub nombre: String,
    /// Lo que de verdad hay que pagar: ya con quita si se negoció.
    pub saldo: i64,
    pub pago_mensual: i64,
    /// Tasa anual en porcentaje: 70.0 es 70%.
    pub tasa_anual: f64,
    /// Si es true, se ataca primero sin importar tasa ni saldo.
    pub urgente: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeudaLiquidada {
    pub nombre: String,
    pub mes: u32,
    pub total_pagado: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PuntoCurva {
    pub mes: u32,
    pub saldo: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Simulacion {
    pub meses: u32,
    pub intereses: i64,
    pub total_pagado: i64,
    pub orden: Vec<DeudaLiquidada>,
    pub curva: Vec<PuntoCurva>,
    /// True cuando el abono no alcanza ni para cubrir los intereses.
    pub inalcanzable: bool,
    pub atoradas: Vec<String>,
}

const TOPE_MESES: u32 = 360;
/// Debajo de un peso damos la deuda por saldada: son residuos de redondeo.
const UMBRAL_SALDADA: i64 = 100;

struct EnCurso {
    nombre: String,
    saldo: i64,
    pago_mensual: i64,
    tasa_anual: f64,
    urgente: bool,
    pagado: i64,
    mes_fin: Option<u32>,
}

/// Simula la liquidación.
///
/// * `capacidad` — TODO el dinero que se destina cada mes a deuda, en centavos.
///   De aquí salen primero los pagos mínimos comprometidos y lo que sobra se
///   concentra en una sola deuda. Si la capacidad no alcanza para cubrir los
///   mínimos, se pagan hasta donde llegue y el resultado sale `inalcanzable`.
/// * `apoyo_mensual` / `meses_apoyo` — dinero prestado que entra los primeros meses.
pub fn simular(
    deudas: &[DeudaSimulada],
    capacidad: i64,
    metodo: Metodo,
    apoyo_mensual: i64,
    meses_apoyo: u32,
) -> Simulacion {
    let mut vivas: Vec<EnCurso> = deudas
        .iter()
        .filter(|d| d.saldo > 0)
        .map(|d| EnCurso {
            nombre: d.nombre.clone(),
            saldo: d.saldo,
            pago_mensual: d.pago_mensual.max(0),
            tasa_anual: d.tasa_anual.max(0.0),
            urgente: d.urgente,
            pagado: 0,
            mes_fin: None,
        })
        .collect();

    let saldo_total = |v: &Vec<EnCurso>| -> i64 { v.iter().map(|d| d.saldo.max(0)).sum() };

    let mut curva = vec![PuntoCurva { mes: 0, saldo: saldo_total(&vivas) }];
    let mut orden: Vec<DeudaLiquidada> = Vec::new();
    let mut intereses: i64 = 0;
    let mut total_pagado: i64 = 0;
    let mut liberado: i64 = 0;
    let mut mes: u32 = 0;

    while mes < TOPE_MESES && vivas.iter().any(|d| d.saldo > UMBRAL_SALDADA) {
        mes += 1;

        // Interés del mes sobre lo que sigue vivo.
        for d in vivas.iter_mut().filter(|d| d.saldo > UMBRAL_SALDADA) {
            let interes = ((d.saldo as f64) * d.tasa_anual / 100.0 / 12.0).round() as i64;
            d.saldo += interes;
            intereses += interes;
        }

        let apoyo = if mes <= meses_apoyo { apoyo_mensual } else { 0 };
        let mut bolsa = capacidad + apoyo + liberado;

        // Primero los pagos mínimos comprometidos.
        for d in vivas.iter_mut().filter(|d| d.saldo > UMBRAL_SALDADA) {
            let pago = d.pago_mensual.min(d.saldo).min(bolsa.max(0));
            if pago > 0 {
                d.saldo -= pago;
                d.pagado += pago;
                bolsa -= pago;
                total_pagado += pago;
            }
        }

        // Y el resto se concentra en una sola deuda, según el método.
        let mut indices: Vec<usize> = (0..vivas.len())
            .filter(|&i| vivas[i].saldo > UMBRAL_SALDADA)
            .collect();
        indices.sort_by(|&a, &b| {
            let (x, y) = (&vivas[a], &vivas[b]);
            // Lo urgente va antes que cualquier criterio financiero.
            y.urgente
                .cmp(&x.urgente)
                .then_with(|| match metodo {
                    Metodo::Avalancha => y
                        .tasa_anual
                        .partial_cmp(&x.tasa_anual)
                        .unwrap_or(std::cmp::Ordering::Equal),
                    Metodo::BolaDeNieve => x.saldo.cmp(&y.saldo),
                })
        });
        for i in indices {
            if bolsa <= 0 {
                break;
            }
            let golpe = bolsa.min(vivas[i].saldo);
            if golpe > 0 {
                vivas[i].saldo -= golpe;
                vivas[i].pagado += golpe;
                bolsa -= golpe;
                total_pagado += golpe;
            }
        }

        // Cierre de las que quedaron saldadas este mes.
        for d in vivas.iter_mut() {
            if d.saldo <= UMBRAL_SALDADA && d.mes_fin.is_none() {
                d.saldo = 0;
                d.mes_fin = Some(mes);
                liberado += d.pago_mensual;
                orden.push(DeudaLiquidada {
                    nombre: d.nombre.clone(),
                    mes,
                    total_pagado: d.pagado,
                });
            }
        }

        curva.push(PuntoCurva { mes, saldo: saldo_total(&vivas) });
    }

    let atoradas: Vec<String> = vivas
        .iter()
        .filter(|d| d.saldo > UMBRAL_SALDADA)
        .map(|d| d.nombre.clone())
        .collect();

    Simulacion {
        meses: mes,
        intereses,
        total_pagado,
        orden,
        curva,
        inalcanzable: !atoradas.is_empty(),
        atoradas,
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn deuda(nombre: &str, saldo: i64, pago: i64, tasa: f64) -> DeudaSimulada {
        DeudaSimulada {
            nombre: nombre.to_string(),
            saldo,
            pago_mensual: pago,
            tasa_anual: tasa,
            urgente: false,
        }
    }

    #[test]
    fn sin_intereses_se_liquida_en_los_meses_justos() {
        let deudas = vec![deuda("una", 10_000_00, 0, 0.0)];
        let s = simular(&deudas, 2_500_00, Metodo::Avalancha, 0, 0);
        assert_eq!(s.meses, 4);
        assert_eq!(s.intereses, 0);
        assert!(!s.inalcanzable);
    }

    #[test]
    fn la_avalancha_paga_menos_intereses_que_la_bola() {
        let deudas = vec![
            deuda("cara y grande", 30_000_00, 500_00, 90.0),
            deuda("barata y chica", 5_000_00, 200_00, 20.0),
        ];
        let a = simular(&deudas, 3_000_00, Metodo::Avalancha, 0, 0);
        let b = simular(&deudas, 3_000_00, Metodo::BolaDeNieve, 0, 0);
        assert!(a.intereses <= b.intereses, "avalancha debería costar menos");
    }

    #[test]
    fn la_bola_de_nieve_cierra_antes_la_deuda_chica() {
        let deudas = vec![
            deuda("grande", 30_000_00, 0, 50.0),
            deuda("chica", 2_000_00, 0, 10.0),
        ];
        let b = simular(&deudas, 3_000_00, Metodo::BolaDeNieve, 0, 0);
        assert_eq!(b.orden.first().map(|d| d.nombre.as_str()), Some("chica"));
    }

    #[test]
    fn lo_urgente_se_paga_primero_aunque_no_cobre_intereses() {
        let mut urgente = deuda("tenencia", 10_000_00, 0, 0.0);
        urgente.urgente = true;
        let deudas = vec![deuda("tarjeta", 20_000_00, 0, 90.0), urgente];
        let s = simular(&deudas, 10_000_00, Metodo::Avalancha, 0, 0);
        assert_eq!(s.orden.first().map(|d| d.nombre.as_str()), Some("tenencia"));
    }

    #[test]
    fn el_apoyo_acorta_el_plazo() {
        let deudas = vec![deuda("tarjeta", 60_000_00, 1_000_00, 70.0)];
        let sin = simular(&deudas, 3_000_00, Metodo::Avalancha, 0, 0);
        let con = simular(&deudas, 3_000_00, Metodo::Avalancha, 10_000_00, 6);
        assert!(con.meses < sin.meses);
        assert!(con.intereses < sin.intereses);
    }

    #[test]
    fn si_el_abono_no_cubre_intereses_se_marca_inalcanzable() {
        let deudas = vec![deuda("imposible", 100_000_00, 100_00, 90.0)];
        let s = simular(&deudas, 0, Metodo::Avalancha, 0, 0);
        assert!(s.inalcanzable);
        assert_eq!(s.atoradas.len(), 1);
    }

    #[test]
    fn el_pago_liberado_rueda_a_la_siguiente_deuda() {
        let deudas = vec![
            deuda("chica", 1_000_00, 500_00, 0.0),
            deuda("grande", 10_000_00, 500_00, 0.0),
        ];
        // La capacidad es justo la suma de los mínimos: no hay extra propio.
        let s = simular(&deudas, 1_000_00, Metodo::Avalancha, 0, 0);
        // Sin rodar el pago liberado, la grande sola tardaría 20 meses.
        assert!(s.meses < 20, "el pago liberado debe acelerar, fueron {}", s.meses);
        assert_eq!(s.orden.first().map(|d| d.nombre.as_str()), Some("chica"));
    }

    #[test]
    fn sin_capacidad_no_se_paga_nada_y_se_avisa() {
        let deudas = vec![deuda("tarjeta", 10_000_00, 500_00, 0.0)];
        let s = simular(&deudas, 0, Metodo::Avalancha, 0, 0);
        assert!(s.inalcanzable, "sin dinero la deuda no puede bajar");
        assert_eq!(s.total_pagado, 0);
    }
}
