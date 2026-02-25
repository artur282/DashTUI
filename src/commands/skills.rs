//! Módulo de integración con skills.sh — búsqueda e instalación de AI skills.
//!
//! Realiza web scraping del sitio https://skills.sh/ para descubrir skills
//! y permite instalarlas mediante el CLI `npx skills add`.

use std::process::Command;

use scraper::{Html, Selector};

use crate::error::SkillsError;

/// Representa una skill obtenida del leaderboard de skills.sh.
#[derive(Debug, Clone)]
pub struct Skill {
    /// Posición en el ranking del leaderboard
    pub rank: usize,
    /// Nombre identificador de la skill (ej: "find-skills")
    pub name: String,
    /// Autor/organización que creó la skill (ej: "vercel-labs")
    pub author: String,
    /// Ruta completa para instalación (ej: "vercel-labs/skills/find-skills")
    pub install_path: String,
    /// Cantidad de instalaciones reportadas
    pub installs: String,
}

/// URL base del sitio de skills
const SKILLS_BASE_URL: &str = "https://skills.sh";

/// Busca skills en skills.sh mediante web scraping.
///
/// Realiza una petición HTTP GET al sitio, parsea el HTML resultante
/// y filtra los resultados con el `query` proporcionado (case-insensitive).
///
/// # Argumentos
/// * `query` - Texto de búsqueda para filtrar las skills por nombre/autor
///
/// # Retorna
/// Un vector de `Skill` que coinciden con la búsqueda
pub fn search_skills(query: &str) -> Result<Vec<Skill>, SkillsError> {
    // Hacer la petición HTTP al sitio de skills.sh
    let body = fetch_skills_page()?;

    // Parsear el HTML para extraer las skills del leaderboard
    let all_skills = parse_skills_from_html(&body)?;

    // Filtrar por query (búsqueda case-insensitive en nombre y autor)
    let query_lower = query.to_lowercase();
    let filtered: Vec<Skill> = all_skills
        .into_iter()
        .filter(|skill| {
            skill.name.to_lowercase().contains(&query_lower)
                || skill.author.to_lowercase().contains(&query_lower)
        })
        .collect();

    Ok(filtered)
}

/// Obtiene todas las skills sin filtro desde el leaderboard.
///
/// # Retorna
/// Un vector con todas las skills disponibles en la página principal
pub fn fetch_all_skills() -> Result<Vec<Skill>, SkillsError> {
    let body = fetch_skills_page()?;
    parse_skills_from_html(&body)
}

/// Realiza la petición HTTP GET a skills.sh y retorna el HTML como string.
fn fetch_skills_page() -> Result<String, SkillsError> {
    let response = ureq::get(SKILLS_BASE_URL)
        .call()
        .map_err(|e| SkillsError::Network(e.to_string()))?;

    response
        .into_string()
        .map_err(|e| SkillsError::Network(e.to_string()))
}

/// Parsea el HTML de skills.sh y extrae las skills del leaderboard.
///
/// Busca enlaces con formato `/author/repo/skill-name` en la tabla del leaderboard
/// y extrae la metadata de cada skill encontrada.
fn parse_skills_from_html(html: &str) -> Result<Vec<Skill>, SkillsError> {
    let document = Html::parse_document(html);

    // Selector para los enlaces de skills en el leaderboard
    // Los links de skills tienen la forma: /author/repo/skill-name
    let link_selector = Selector::parse("a[href]")
        .map_err(|e| SkillsError::ParseError(format!("Selector inválido: {e:?}")))?;

    let mut skills = Vec::new();
    let mut rank = 0_usize;

    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            // Filtrar solo enlaces con formato /author/repo/skill-name
            let parts: Vec<&str> = href.trim_start_matches('/').split('/').collect();
            if parts.len() >= 3 && !href.starts_with("http") && !is_navigation_link(href) {
                rank += 1;
                let author = parts[0].to_string();
                let skill_name = parts.last().unwrap_or(&"").to_string();
                let install_path = href.trim_start_matches('/').to_string();

                // Extraer el texto del enlace que puede contener nombre e installs
                let text = element.text().collect::<String>();

                // Intentar separar nombre de installs del texto
                let installs = extract_installs_from_text(&text);

                skills.push(Skill {
                    rank,
                    name: skill_name,
                    author,
                    install_path,
                    installs,
                });
            }
        }
    }

    if skills.is_empty() {
        return Err(SkillsError::ParseError(
            "No se encontraron skills en el HTML".to_string(),
        ));
    }

    Ok(skills)
}

/// Determina si un enlace es de navegación del sitio (no una skill).
fn is_navigation_link(href: &str) -> bool {
    let nav_paths = [
        "/docs",
        "/audits",
        "/trending",
        "/hot",
        "/faq",
    ];
    nav_paths.iter().any(|p| href == *p)
}

/// Extrae la cantidad de instalaciones del texto de un enlace de skill.
///
/// El texto puede ser algo como "find-skills319.5K" o simplemente "find-skills".
fn extract_installs_from_text(text: &str) -> String {
    // Buscar patrones numéricos al final (ej: "319.5K", "1.2K")
    let trimmed = text.trim();
    // Recorrer desde el final buscando donde empieza el número
    let mut install_start = trimmed.len();
    for (i, ch) in trimmed.char_indices().rev() {
        if ch.is_ascii_digit() || ch == '.' || ch == 'K' || ch == 'M' || ch == 'k' {
            install_start = i;
        } else {
            break;
        }
    }

    if install_start < trimmed.len() {
        trimmed[install_start..].to_string()
    } else {
        "N/A".to_string()
    }
}

/// Instala una skill ejecutando `npx skills add <install_path>`.
/// Ejecuta la instalación de una skill de forma interactiva.
/// 
/// Suspende el control del terminal por parte del TUI para permitir que el usuario
/// interactúe con los prompts y selecciones del comando `npx skills add`.
pub fn install_skill_interactive(install_path: &str) -> std::io::Result<()> {
    println!("\n\x1b[36m┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓\x1b[0m");
    println!("\x1b[36m┃\x1b[0m  🔌 Preparando instalación de Skill: \x1b[1;33m{:<14}\x1b[0m \x1b[36m┃\x1b[0m", install_path);
    println!("\x1b[36m┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛\x1b[0m\n");

    // Ejecutar npx skills add conectando stdin/stdout directamente al terminal del usuario
    let mut child = Command::new("npx")
        .args(["skills", "add", install_path])
        .spawn()?;

    let status = child.wait()?;

    if status.success() {
        println!("\n\x1b[32m✅ Instalación finalizada con éxito.\x1b[0m");
    } else {
        println!("\n\x1b[31m❌ El comando falló o fue cancelado.\x1b[0m");
    }

    println!("\n\x1b[2mPresiona \x1b[0m\x1b[1mEnter\x1b[0m\x1b[2m para volver a DashTUI...\x1b[0m");
    let mut pause = String::new();
    let _ = std::io::stdin().read_line(&mut pause);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_navigation_link_should_detect_nav_paths() {
        assert!(is_navigation_link("/docs"));
        assert!(is_navigation_link("/audits"));
        assert!(!is_navigation_link("/vercel-labs/skills/find-skills"));
    }

    #[test]
    fn extract_installs_should_parse_numeric_suffix() {
        assert_eq!(extract_installs_from_text("find-skills319.5K"), "319.5K");
        assert_eq!(extract_installs_from_text("my-skill1.2K"), "1.2K");
        assert_eq!(extract_installs_from_text("skill-only"), "N/A");
    }
}
