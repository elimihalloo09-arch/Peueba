//! Fechas escritas como se dicen: "24 de septiembre de 2026".

use chrono::{Datelike, NaiveDate, Weekday};

const MESES: [&str; 12] = [
    "enero", "febrero", "marzo", "abril", "mayo", "junio",
    "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre",
];

pub fn larga(d: NaiveDate) -> String {
    format!("{} de {} de {}", d.day(), MESES[d.month0() as usize], d.year())
}

pub fn con_dia(d: NaiveDate) -> String {
    let dia = match d.weekday() {
        Weekday::Mon => "lunes",
        Weekday::Tue => "martes",
        Weekday::Wed => "miércoles",
        Weekday::Thu => "jueves",
        Weekday::Fri => "viernes",
        Weekday::Sat => "sábado",
        Weekday::Sun => "domingo",
    };
    format!("{dia} {}", larga(d))
}

pub fn corta(d: NaiveDate) -> String {
    d.format("%d/%m/%Y").to_string()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn escribe_las_fechas_en_espanol() {
        let d = NaiveDate::from_ymd_opt(2026, 9, 24).unwrap();
        assert_eq!(larga(d), "24 de septiembre de 2026");
        assert_eq!(corta(d), "24/09/2026");
        let s = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
        assert_eq!(con_dia(s), "jueves 1 de octubre de 2026");
    }
}
