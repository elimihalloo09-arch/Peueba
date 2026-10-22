//! Lee el texto que manda cada equipo, tal cual llega por correo o chat, y lo
//! separa en proyectos, puntos de avance con su estado y próximos pasos.
//!
//! Entiende el formato del deck ("PROYECTO: X", "Equipo …", "• … Estado: En
//! proceso.") y también lo informal: "X - Equipo Y", viñetas con guion o número,
//! "(pendiente)" al final, "Próximos pasos:". Lo que no trae estado explícito
//! se adivina por las palabras y queda marcado como `inferido` para revisarlo.

use serde::{Deserialize, Serialize};

use super::EstadoAvance;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PuntoLeido {
    pub texto: String,
    pub estado: Option<EstadoAvance>,
    #[serde(default)]
    pub inferido: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloqueLeido {
    pub proyecto: String,
    #[serde(default)]
    pub equipo: String,
    /// No coincide con ningún proyecto del catálogo: se dará de alta al sumar.
    #[serde(default)]
    pub nuevo: bool,
    pub puntos: Vec<PuntoLeido>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasoLeido {
    pub proyecto: Option<String>,
    pub texto: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Lectura {
    pub bloques: Vec<BloqueLeido>,
    pub pasos: Vec<PasoLeido>,
    pub avisos: Vec<String>,
}

const SIN_PROYECTO: &str = "Sin proyecto";

/// `conocidos` son los nombres del catálogo, para reconocerlos aunque vengan en
/// mayúsculas, sin acentos o abreviados. `por_omision` recibe los puntos que
/// llegan antes de cualquier encabezado.
pub fn leer(texto: &str, conocidos: &[String], por_omision: Option<&str>) -> Lectura {
    let renglones: Vec<&str> = texto.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    let mut l = Lectura::default();
    let mut en_pasos = false;
    // Si el último punto quedó sin cerrar, un renglón en minúscula lo continúa.
    let mut abierto = false;
    // El bloque al que van los puntos: no siempre el último, si un proyecto se repite.
    let mut actual: Option<usize> = None;

    for (i, crudo) in renglones.iter().enumerate() {
        let (vineta, cuerpo) = quitar_vineta(crudo);
        if cuerpo.is_empty() {
            continue;
        }
        let siguiente_es_contenido = renglones
            .get(i + 1)
            .map(|s| quitar_vineta(s).0 || linea_equipo(s).is_some())
            .unwrap_or(false);

        let p = plegar(cuerpo);
        if !vineta && es_cortesia(&p) {
            continue;
        }
        if let Some(valor) = quitar_prefijo(&p, cuerpo, &["estado", "estatus", "status"]) {
            if let (Some(e), Some(ultimo)) = (leer_estado(&valor), actual.and_then(|i| l.bloques[i].puntos.last_mut())) {
                ultimo.estado = Some(e);
                ultimo.inferido = false;
                continue;
            }
        }
        if let Some(resto) = quitar_prefijo(&p, cuerpo, &["proximo paso", "siguiente paso"]) {
            let proyecto = actual.map(|i| l.bloques[i].proyecto.clone());
            agregar_paso(&mut l, proyecto, &resto, conocidos);
            abierto = false;
            continue;
        }

        if !vineta {
            if let Some(resto) = encabezado_pasos(cuerpo) {
                en_pasos = true;
                abierto = false;
                if !resto.is_empty() {
                    agregar_paso(&mut l, None, resto, conocidos);
                }
                continue;
            }
            if let Some(eq) = linea_equipo(cuerpo) {
                if let Some(b) = actual.map(|i| &mut l.bloques[i]) {
                    if b.equipo.is_empty() && b.puntos.is_empty() {
                        b.equipo = eq;
                        continue;
                    }
                }
            }
            if abierto && empieza_en_minuscula(cuerpo) {
                if let Some(p) = actual.and_then(|i| l.bloques[i].puntos.last_mut()) {
                    let unido = format!("{} {}", p.texto.trim_end_matches('.'), cuerpo);
                    *p = punto(&unido);
                    abierto = !termina_cerrado(cuerpo);
                    continue;
                }
            }
            if let Some((nombre, equipo, resto)) = encabezado_proyecto(cuerpo, conocidos, siguiente_es_contenido) {
                en_pasos = false;
                abierto = false;
                let i = bloque(&mut l, &nombre, conocidos);
                actual = Some(i);
                let b = &mut l.bloques[i];
                if b.equipo.is_empty() {
                    b.equipo = equipo;
                }
                if !resto.is_empty() {
                    b.puntos.push(punto(&resto));
                    abierto = !termina_cerrado(&resto);
                }
                continue;
            }
        }

        if en_pasos {
            agregar_paso(&mut l, None, cuerpo, conocidos);
            abierto = false;
            continue;
        }

        if actual.is_none() {
            let nombre = por_omision.filter(|s| !s.trim().is_empty()).unwrap_or(SIN_PROYECTO);
            if nombre == SIN_PROYECTO {
                l.avisos.push(
                    "Hay puntos antes de cualquier proyecto: quedaron en «Sin proyecto». \
                     Elige a qué proyecto van o agrega el nombre arriba del texto."
                        .into(),
                );
            }
            actual = Some(bloque(&mut l, nombre, conocidos));
        }
        let b = &mut l.bloques[actual.expect("hay bloque")];
        b.puntos.push(punto(cuerpo));
        abierto = !termina_cerrado(cuerpo);
    }

    l.bloques.retain(|b| !b.puntos.is_empty() || !b.equipo.is_empty());
    for b in &l.bloques {
        if b.puntos.is_empty() {
            l.avisos.push(format!("«{}» no trae ningún punto de avance.", b.proyecto));
        }
    }
    let inferidos = l.bloques.iter().flat_map(|b| &b.puntos).filter(|p| p.inferido).count();
    if inferidos > 0 {
        l.avisos.push(format!(
            "{inferidos} {} sin estado explícito: lo adiviné por las palabras, revísalo.",
            if inferidos == 1 { "punto venía" } else { "puntos venían" }
        ));
    }
    l
}

/// Busca el bloque del proyecto o lo crea. Un proyecto mencionado dos veces
/// junta sus puntos en un solo bloque.
fn bloque(l: &mut Lectura, nombre: &str, conocidos: &[String]) -> usize {
    let (canonico, nuevo) = match coincidir(nombre, conocidos) {
        Some(c) => (c.to_string(), false),
        None => (nombre_bonito(nombre), nombre != SIN_PROYECTO),
    };
    let pos = l.bloques.iter().position(|b| clave(&b.proyecto) == clave(&canonico));
    pos.unwrap_or_else(|| {
        l.bloques.push(BloqueLeido { proyecto: canonico, equipo: String::new(), nuevo, puntos: vec![] });
        l.bloques.len() - 1
    })
}

fn agregar_paso(l: &mut Lectura, proyecto: Option<String>, texto: &str, conocidos: &[String]) {
    // "Proyecto: lo que sigue" asigna el paso a ese proyecto.
    if proyecto.is_none() {
        if let Some((antes, despues)) = texto.split_once(':') {
            let mismos: Vec<String> = l.bloques.iter().map(|b| b.proyecto.clone()).chain(conocidos.iter().cloned()).collect();
            if let Some(c) = coincidir(antes, &mismos) {
                if !despues.trim().is_empty() {
                    l.pasos.push(PasoLeido { proyecto: Some(c.to_string()), texto: limpiar(despues) });
                    return;
                }
            }
        }
    }
    l.pasos.push(PasoLeido { proyecto, texto: limpiar(texto) });
}

fn punto(texto: &str) -> PuntoLeido {
    let (limpio, estado, inferido) = extraer_estado(texto);
    PuntoLeido { texto: limpiar(&limpio), estado, inferido }
}

// ---------------------------------------------------------------- renglones

fn quitar_vineta(l: &str) -> (bool, &str) {
    let l = l.trim();
    let mut cs = l.char_indices();
    if let Some((_, c)) = cs.next() {
        if matches!(c, '-' | '*' | '•' | '·' | '–' | '—' | '‣' | '◦' | '▪' | '►' | '>' | '+') {
            let resto = &l[c.len_utf8()..];
            if resto.starts_with(' ') || resto.starts_with('\t') {
                return (true, resto.trim());
            }
        }
        // 1. / 1) / a)
        let fin_num = l.find(|c: char| !c.is_ascii_digit()).unwrap_or(0);
        let fin = if fin_num > 0 {
            fin_num
        } else if l.len() > 1 && l.as_bytes()[0].is_ascii_lowercase() && l.as_bytes()[1] == b')' {
            1
        } else {
            0
        };
        if fin > 0 {
            let resto = &l[fin..];
            if (resto.starts_with(". ") || resto.starts_with(") ")) && fin <= 2 {
                return (true, resto[1..].trim());
            }
        }
    }
    (false, l)
}

fn encabezado_pasos(cuerpo: &str) -> Option<&str> {
    let p = plegar(cuerpo);
    const FRASES: [&str; 6] = [
        "proximos pasos", "siguientes pasos", "pasos a seguir",
        "proximas acciones", "siguientes acciones", "acuerdos",
    ];
    let frase = FRASES.iter().find(|f| p.starts_with(*f))?;
    let resto = &cuerpo[cuerpo.char_indices().nth(frase.chars().count()).map(|(b, _)| b).unwrap_or(cuerpo.len())..];
    let resto = resto.trim_start_matches(|c: char| c == ':' || c == '-' || c == '—' || c.is_whitespace());
    // "Próximos pasos de la semana pasada fueron…" no es encabezado.
    if !resto.is_empty() && !cuerpo.contains(':') {
        return None;
    }
    Some(resto.trim())
}

fn linea_equipo(cuerpo: &str) -> Option<String> {
    let (_, cuerpo) = quitar_vineta(cuerpo);
    let p = plegar(cuerpo);
    if cuerpo.chars().count() > 80 {
        return None;
    }
    let limpio = cuerpo.trim_end_matches(['.', ':']).trim();
    if let Some(resto) = quitar_prefijo(&p, cuerpo, &["equipo"]) {
        if cuerpo.contains(':') && !plegar(&resto).starts_with("equipo") {
            return Some(format!("Equipo {}", resto.trim_end_matches('.')));
        }
        return Some(limpio.to_string());
    }
    if let Some(resto) = quitar_prefijo(&p, cuerpo, &["responsable", "area", "lider"]) {
        if cuerpo.contains(':') && !resto.is_empty() {
            return Some(limpio.to_string());
        }
    }
    None
}

/// Devuelve (nombre, equipo, resto del renglón) si el renglón abre un proyecto.
fn encabezado_proyecto(cuerpo: &str, conocidos: &[String], siguiente_es_contenido: bool) -> Option<(String, String, String)> {
    let p = plegar(cuerpo);
    let largo = cuerpo.chars().count();

    if let Some(resto) = quitar_prefijo(&p, cuerpo, &["proyecto"]) {
        let (cabeza, cola) = match resto.split_once(':') {
            Some((a, b)) if !a.trim().is_empty() && a.chars().count() <= 60 => (a.trim().to_string(), b.trim().to_string()),
            _ => (resto.clone(), String::new()),
        };
        let (nombre, equipo) = dividir_equipo(&cabeza);
        if !nombre.is_empty() {
            return Some((nombre, equipo, cola));
        }
    }

    if let Some((antes, despues)) = cuerpo.split_once(':') {
        let antes_t = antes.trim();
        if coincidir(antes_t, conocidos).is_some() {
            let (nombre, equipo) = dividir_equipo(antes_t);
            return Some((nombre, equipo, despues.trim().to_string()));
        }
        if despues.trim().is_empty() && largo <= 80 {
            let (nombre, equipo) = dividir_equipo(antes_t);
            return Some((nombre, equipo, String::new()));
        }
    }

    let palabras = cuerpo.split_whitespace().count();
    let corto = largo <= 60 && palabras <= 8 && !cuerpo.ends_with('.');
    if corto {
        let (nombre, equipo) = dividir_equipo(cuerpo);
        if coincidir(&nombre, conocidos).is_some() || siguiente_es_contenido {
            return Some((nombre, equipo, String::new()));
        }
    }
    None
}

/// "Portal - Equipo jurídico" o "Portal (Equipo jurídico)".
fn dividir_equipo(s: &str) -> (String, String) {
    let s = s.trim().trim_end_matches(':').trim();
    for sep in [" - ", " — ", " – ", " | ", " / "] {
        if let Some((a, b)) = s.split_once(sep) {
            return (a.trim().to_string(), b.trim().to_string());
        }
    }
    if s.ends_with(')') {
        if let Some(abre) = s.rfind('(') {
            let dentro = &s[abre + 1..s.len() - 1];
            if plegar(dentro).starts_with("equipo") {
                return (s[..abre].trim().to_string(), dentro.trim().to_string());
            }
        }
    }
    (s.to_string(), String::new())
}

/// "Hola, les paso el avance:", "Saludos", "Quedo atento": no son avances.
fn es_cortesia(p: &str) -> bool {
    const FRASES: [&str; 16] = [
        "hola", "buen dia", "buenos dias", "buenas tardes", "buenas noches", "saludos", "les paso",
        "les comparto", "te paso", "te comparto", "les envio", "adjunto", "quedo atent", "gracias",
        "estimad", "cualquier duda",
    ];
    FRASES.iter().any(|f| p.starts_with(f))
}

// ------------------------------------------------------------------ estado

/// Saca el estado del texto: "Estado: X", "(X)", "[X]" o "— X" al final. Si no
/// viene, lo adivina por las palabras y lo marca como inferido.
fn extraer_estado(texto: &str) -> (String, Option<EstadoAvance>, bool) {
    let chars: Vec<char> = texto.chars().collect();
    let pleg: String = chars.iter().map(|&c| plegar_char(c)).collect();
    let corta = |hasta_byte: usize| -> String {
        let n = pleg[..hasta_byte].chars().count();
        chars[..n].iter().collect::<String>()
    };

    for marca in ["estado:", "estatus:", "status:", "estado -", "estado ="] {
        if let Some(b) = pleg.rfind(marca) {
            if let Some(e) = leer_estado(&pleg[b + marca.len()..]) {
                return (corta(b), Some(e), false);
            }
        }
    }

    let sin_punto = pleg.trim_end_matches(['.', ' ']);
    for (abre, cierra) in [('(', ')'), ('[', ']')] {
        if sin_punto.ends_with(cierra) {
            if let Some(b) = sin_punto.rfind(abre) {
                if let Some(e) = leer_estado(&sin_punto[b + 1..sin_punto.len() - 1]) {
                    return (corta(b), Some(e), false);
                }
            }
        }
    }

    for sep in [" - ", " — ", " – ", " | ", ". ", ": ", ", "] {
        if let Some(b) = sin_punto.rfind(sep) {
            if let Some(e) = leer_estado(&sin_punto[b + sep.len()..]) {
                return (corta(b), Some(e), false);
            }
        }
    }

    match inferir(&pleg).or_else(|| preterito(texto)) {
        Some(e) => (texto.to_string(), Some(e), true),
        None => (texto.to_string(), None, false),
    }
}

/// Un valor corto como "En proceso", "pendiente", "concluido ✓".
fn leer_estado(valor: &str) -> Option<EstadoAvance> {
    let v = plegar(valor);
    let v = v.trim().trim_matches(|c: char| !c.is_alphanumeric());
    if v.is_empty() || v.chars().count() > 24 {
        return None;
    }
    const CONCLUIDO: [&str; 9] = ["conclu", "termin", "complet", "listo", "lista", "hecho", "liberad", "cerrad", "finaliz"];
    const EN_PROCESO: [&str; 6] = ["proceso", "curso", "avance", "desarrollo", "trabajando", "progreso"];
    const PENDIENTE: [&str; 7] = ["pendiente", "espera", "bloque", "detenid", "sin iniciar", "por iniciar", "atrasad"];
    if EN_PROCESO.iter().any(|k| v.contains(k)) {
        Some(EstadoAvance::EnProceso)
    } else if PENDIENTE.iter().any(|k| v.contains(k)) {
        Some(EstadoAvance::Pendiente)
    } else if CONCLUIDO.iter().any(|k| v.contains(k)) {
        Some(EstadoAvance::Concluido)
    } else {
        None
    }
}

/// Un verbo en pretérito ("se cargó", "QA liberó") cuenta como hecho.
fn preterito(texto: &str) -> Option<EstadoAvance> {
    texto
        .split(|c: char| !c.is_alphabetic())
        .any(|w| w.chars().count() > 3 && w.ends_with('ó'))
        .then_some(EstadoAvance::Concluido)
}

fn inferir(p: &str) -> Option<EstadoAvance> {
    const EN_PROCESO: [&str; 13] = [
        "continua", "en proceso", "se esta ", "se estan ", "estamos ", "trabajando", "atendiendo",
        "en curso", "avanza", "en desarrollo", "en pruebas", "seguimos", "sigue ",
    ];
    const PENDIENTE: [&str; 13] = [
        "en espera", "pendiente", "falta ", "faltan ", "por agendar", "por definir", "sin respuesta",
        "bloquead", "una vez ", "se compartira", "se enviara", "esperamos", "por confirmar",
    ];
    const CONCLUIDO: [&str; 13] = [
        "concluy", "conclui", "terminad", "se termino", "se libero", "liberad", "aprob", "quedo list",
        "completad", "entregad", "desplegad", "finaliz", "ya esta",
    ];
    if PENDIENTE.iter().any(|k| p.contains(k)) && !EN_PROCESO.iter().any(|k| p.contains(k)) {
        Some(EstadoAvance::Pendiente)
    } else if EN_PROCESO.iter().any(|k| p.contains(k)) {
        Some(EstadoAvance::EnProceso)
    } else if CONCLUIDO.iter().any(|k| p.contains(k)) {
        Some(EstadoAvance::Concluido)
    } else {
        None
    }
}

// ------------------------------------------------------------------- texto

/// Minúsculas y sin acentos, carácter por carácter: conserva las posiciones.
pub fn plegar(s: &str) -> String {
    s.chars().map(plegar_char).collect()
}

fn plegar_char(c: char) -> char {
    let c = c.to_lowercase().next().unwrap_or(c);
    match c {
        'á' | 'à' | 'ä' | 'â' | 'ā' | 'ã' => 'a',
        'é' | 'è' | 'ë' | 'ê' | 'ē' => 'e',
        'í' | 'ì' | 'ï' | 'î' | 'ī' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' | 'ō' | 'õ' => 'o',
        'ú' | 'ù' | 'ü' | 'û' | 'ū' => 'u',
        'ñ' => 'n',
        otro => otro,
    }
}

/// Para comparar nombres y frases: plegado y solo letras y números.
pub fn clave(s: &str) -> String {
    plegar(s)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Encuentra el nombre del catálogo al que se refiere `nombre`: igual sin
/// acentos ni mayúsculas, o con la misma primera palabra si solo hay uno así
/// ("Asistente" ↔ "Asistente virtual (Fase 2)").
pub fn coincidir<'a>(nombre: &str, conocidos: &'a [String]) -> Option<&'a str> {
    let k = clave(nombre);
    if k.is_empty() {
        return None;
    }
    if let Some(c) = conocidos.iter().find(|c| clave(c) == k) {
        return Some(c);
    }
    let primera = k.split(' ').next().unwrap_or("");
    if primera.chars().count() < 4 {
        return None;
    }
    let mismos: Vec<&String> = conocidos
        .iter()
        .filter(|c| clave(c).split(' ').next() == Some(primera))
        .collect();
    // Solo cuando la otra parte es un prefijo: "Asistente" sí, "Portal interno" vs "Portal ciudadano" no.
    match mismos.as_slice() {
        [uno] if clave(uno).starts_with(&k) || k.starts_with(&clave(uno)) => Some(uno.as_str()),
        _ => None,
    }
}

/// Si `p` (plegado) empieza con alguna palabra clave, devuelve lo que sigue en
/// el texto original, sin dos puntos ni guiones al inicio.
fn quitar_prefijo(p: &str, original: &str, claves: &[&str]) -> Option<String> {
    for k in claves {
        if p.starts_with(k) {
            let n = k.chars().count();
            // "proyectos" o "estadística" no son la palabra clave.
            if p.chars().nth(n).map(|c| c.is_alphanumeric()).unwrap_or(false) {
                continue;
            }
            let resto: String = original.chars().skip(n).collect();
            let resto = resto.trim_start_matches(|c: char| c == ':' || c == '-' || c == '—' || c == '=' || c.is_whitespace());
            return Some(resto.trim().to_string());
        }
    }
    None
}

/// Mayúscula inicial, sin separadores colgando y con punto final.
pub fn limpiar(t: &str) -> String {
    let t = t.trim().trim_end_matches([' ', '-', '—', '–', ',', ';', ':', '|']).trim();
    let mut cs = t.chars();
    let mut s = match cs.next() {
        Some(c) => c.to_uppercase().collect::<String>() + cs.as_str(),
        None => return String::new(),
    };
    if s.chars().last().map(|c| c.is_alphanumeric() || c == ')' || c == '"' || c == '»').unwrap_or(false) {
        s.push('.');
    }
    s
}

/// "PORTAL DE TRÁMITES (SAT)" → "Portal de Trámites (SAT)". Lo que ya trae
/// minúsculas se respeta tal cual.
fn nombre_bonito(s: &str) -> String {
    let s = s.trim();
    if s.chars().any(|c| c.is_lowercase()) {
        return s.to_string();
    }
    const MENORES: [&str; 12] = ["de", "del", "la", "las", "el", "los", "y", "e", "o", "en", "para", "con"];
    let mut dentro = false;
    s.split(' ')
        .enumerate()
        .map(|(i, w)| {
            if w.starts_with('(') {
                dentro = true;
            }
            let r = if dentro || w.chars().count() <= 1 {
                w.to_string()
            } else {
                let min = w.to_lowercase();
                if i > 0 && MENORES.contains(&min.as_str()) {
                    min
                } else {
                    let mut cs = min.chars();
                    cs.next().map(|c| c.to_uppercase().collect::<String>() + cs.as_str()).unwrap_or_default()
                }
            };
            if w.ends_with(')') {
                dentro = false;
            }
            r
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn empieza_en_minuscula(s: &str) -> bool {
    s.chars().next().map(|c| c.is_lowercase()).unwrap_or(false)
}

fn termina_cerrado(s: &str) -> bool {
    s.trim_end().ends_with(['.', '!', '?', ')'])
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use EstadoAvance::*;

    fn nombres(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn lee_el_formato_del_deck() {
        let t = "PROYECTO: PORTAL DE TRÁMITES\nEquipo jurídico\n\
                 • Continúan las pruebas del checklist. Estado: En proceso.\n\
                 • Estamos en espera de la retroalimentación de jurídica. Estado: Pendiente.\n";
        let l = leer(t, &[], None);
        assert_eq!(l.bloques.len(), 1);
        let b = &l.bloques[0];
        assert_eq!(b.proyecto, "Portal de Trámites");
        assert_eq!(b.equipo, "Equipo jurídico");
        assert!(b.nuevo);
        assert_eq!(b.puntos[0], PuntoLeido { texto: "Continúan las pruebas del checklist.".into(), estado: Some(EnProceso), inferido: false });
        assert_eq!(b.puntos[1].estado, Some(Pendiente));
        assert!(!b.puntos[1].texto.contains("Estado"));
    }

    #[test]
    fn lee_un_correo_informal() {
        let t = "Mesa de ayuda - Equipo de soporte\n\
                 - ya quedó lista la base de conocimiento (concluido)\n\
                 - falta la capacitación a los usuarios\n\
                 \n\
                 Próximos pasos:\n\
                 - Mesa de ayuda: agendar la capacitación\n\
                 - revisar accesos con seguridad\n";
        let l = leer(t, &[], None);
        let b = &l.bloques[0];
        assert_eq!(b.proyecto, "Mesa de ayuda");
        assert_eq!(b.equipo, "Equipo de soporte");
        assert_eq!(b.puntos[0].texto, "Ya quedó lista la base de conocimiento.");
        assert_eq!(b.puntos[0].estado, Some(Concluido));
        assert_eq!((b.puntos[1].estado, b.puntos[1].inferido), (Some(Pendiente), true));
        assert_eq!(l.pasos.len(), 2);
        assert_eq!(l.pasos[0].proyecto.as_deref(), Some("Mesa de ayuda"));
        assert_eq!(l.pasos[0].texto, "Agendar la capacitación.");
        assert_eq!(l.pasos[1].proyecto, None);
    }

    #[test]
    fn reconoce_proyectos_del_catalogo_aunque_vengan_distinto() {
        let cat = nombres(&["Asistente virtual (AOS)", "Tablero de indicadores"]);
        let t = "ASISTENTE VIRTUAL\n1. Se corrigió la conexión VPN - concluido\n\
                 Tablero de indicadores: se están cargando los datos de agosto\n";
        let l = leer(t, &cat, None);
        assert_eq!(l.bloques.len(), 2);
        assert_eq!(l.bloques[0].proyecto, "Asistente virtual (AOS)");
        assert!(!l.bloques[0].nuevo);
        assert_eq!(l.bloques[0].puntos[0].estado, Some(Concluido));
        assert_eq!(l.bloques[1].proyecto, "Tablero de indicadores");
        assert_eq!(l.bloques[1].puntos[0].texto, "Se están cargando los datos de agosto.");
        assert_eq!(l.bloques[1].puntos[0].estado, Some(EnProceso));
    }

    #[test]
    fn sin_encabezado_usa_el_proyecto_elegido() {
        let l = leer("- Se instaló el servidor.\n- Faltan las pruebas de carga", &[], Some("Portal"));
        assert_eq!(l.bloques[0].proyecto, "Portal");
        assert_eq!(l.bloques[0].puntos.len(), 2);
        let sin = leer("- Se instaló el servidor.", &[], None);
        assert_eq!(sin.bloques[0].proyecto, "Sin proyecto");
        assert!(sin.avisos.iter().any(|a| a.contains("Sin proyecto")));
    }

    #[test]
    fn une_renglones_cortados_y_junta_proyectos_repetidos() {
        let t = "Portal:\n- se migraron los usuarios\nque faltaban del piloto\nOtro:\n- algo. Estado: pendiente\nPortal:\n- QA aprobó la versión";
        let l = leer(t, &[], None);
        assert_eq!(l.bloques.len(), 2);
        assert_eq!(l.bloques[0].puntos[0].texto, "Se migraron los usuarios que faltaban del piloto.");
        assert_eq!(l.bloques[0].puntos.len(), 2);
        assert_eq!(l.bloques[0].puntos[1].estado, Some(Concluido));
    }

    #[test]
    fn ignora_saludos_y_entiende_el_preterito() {
        let t = "Hola, les paso el avance de esta semana:\nPortal\n- QA liberó el módulo de citas\n- Se cargó la base\nSaludos";
        let l = leer(t, &[], None);
        assert_eq!(l.bloques.len(), 1);
        assert_eq!(l.bloques[0].proyecto, "Portal");
        assert_eq!(l.bloques[0].puntos.len(), 2);
        assert!(l.bloques[0].puntos.iter().all(|p| p.estado == Some(Concluido) && p.inferido));
    }

    #[test]
    fn estado_en_renglon_aparte() {
        let l = leer("Portal:\n- Se cargó el catálogo\nEstado: en proceso", &[], None);
        assert_eq!(l.bloques[0].puntos.len(), 1);
        assert_eq!(l.bloques[0].puntos[0].estado, Some(EnProceso));
        assert!(!l.bloques[0].puntos[0].inferido);
    }

    #[test]
    fn coincide_por_primera_palabra_solo_si_no_hay_duda() {
        let cat = nombres(&["Portal ciudadano", "Portal interno", "Asistente virtual (Fase 2)"]);
        assert_eq!(coincidir("ASISTENTE", &cat), Some("Asistente virtual (Fase 2)"));
        assert_eq!(coincidir("portal ciudadano", &cat), Some("Portal ciudadano"));
        assert_eq!(coincidir("Portal", &cat), None);
    }
}
