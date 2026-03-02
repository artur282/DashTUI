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

/// Obtiene el comando de instalación de una skill específica desde su página.
/// Hace scraping de https://skills.sh/author/repo/skill para obtener el comando exacto.
pub fn get_install_command(install_path: &str) -> Result<String, SkillsError> {
    let url = format!("{}/{}", SKILLS_BASE_URL, install_path);

    let body = fetch_page(&url)?;

    // Buscar el comando en el HTML - típicamente está en un bloque de código o texto
    let document = Html::parse_document(&body);

    // Buscar elementos que contengan el comando de instalación
    let text_selector = Selector::parse("p, code, pre")
        .map_err(|e| SkillsError::ParseError(format!("Selector inválido: {e:?}")))?;

    for element in document.select(&text_selector) {
        let text = element.text().collect::<String>();
        // Buscar patrones como "npx skills add ..."
        if text.contains("npx skills add") {
            // Extraer el comando completo
            if let Some(start) = text.find("npx skills add") {
                let rest = &text[start..];
                // Tomar hasta el final de la línea o hasta 100 caracteres
                let cmd: String = rest.chars().take(100).collect();
                let cmd = cmd.trim().to_string();
                if !cmd.is_empty() {
                    return Ok(cmd);
                }
            }
        }
    }

    // Si no encontramos el comando, intentar formato por defecto
    Err(SkillsError::ParseError(
        "No se encontró comando de instalación".to_string(),
    ))
}

/// Fetch una página específica
fn fetch_page(url: &str) -> Result<String, SkillsError> {
    let response = ureq::get(url)
        .set("User-Agent", "DashTUI/1.0 (Rust CLI Tool)")
        .call()
        .map_err(|e| SkillsError::Network(e.to_string()))?;

    response
        .into_string()
        .map_err(|e| SkillsError::Network(e.to_string()))
}

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
        .set("User-Agent", "DashTUI/1.0 (Rust CLI Tool)")
        .call()
        .map_err(|e| SkillsError::Network(e.to_string()))?;

    response
        .into_string()
        .map_err(|e| SkillsError::Network(e.to_string()))
}

/// Parsea el HTML de skills.sh y extrae las skills del leaderboard.
///
/// La estructura del HTML es:
/// [
/// ### skill-name
/// author/repo
/// installs
/// ](/author/repo/skill-name)
fn parse_skills_from_html(html: &str) -> Result<Vec<Skill>, SkillsError> {
    let document = Html::parse_document(html);

    // Buscar todos los enlaces que parecen ser skills
    let link_selector = Selector::parse("a[href]")
        .map_err(|e| SkillsError::ParseError(format!("Selector inválido: {e:?}")))?;

    let mut skills = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            // Los enlaces de skills empiezan con / y tienen formato /author/repo/skill
            if href.starts_with('/') && !href.starts_with("http") {
                let path = href.trim_start_matches('/');
                let parts: Vec<&str> = path.split('/').collect();

                // Debe tener al menos 3 partes: author/repo/skill
                if parts.len() >= 3 {
                    let author = parts[0].to_string();
                    let skill_name = parts.last().unwrap_or(&"").to_string();

                    // Ignorar rutas de navegación
                    if is_navigation_link(href) {
                        continue;
                    }

                    // Crear un identificador único para evitar duplicados
                    let unique_id = format!("{}:{}", author, skill_name);
                    if seen.contains(&unique_id) {
                        continue;
                    }
                    seen.insert(unique_id);

                    // Si no encontramos installs en el enlace, usar N/A
                    // (el scraping de installs es complejo por la estructura del HTML)
                    let installs = "N/A".to_string();

                    let install_path = path.to_string();
                    let rank = skills.len() + 1;

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
    let nav_paths = ["/docs", "/audits", "/trending", "/hot", "/faq"];
    nav_paths.contains(&href)
}

/// Instala una skill buscando el comando de instalación desde la página de la skill.
/// Ejecuta la instalación de una skill de forma interactiva.
///
/// Suspende el control del terminal por parte del TUI para permitir que el usuario
/// interactúe con los prompts y selecciones del comando `npx skills add`.
pub fn install_skill_interactive(install_path: &str) -> std::io::Result<()> {
    println!("\n\x1b[36m┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓\x1b[0m");
    println!("\x1b[36m┃\x1b[0m  🔌 Preparando instalación de Skill: \x1b[1;33m{:<14}\x1b[0m \x1b[36m┃\x1b[0m", install_path);
    println!("\x1b[36m┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛\x1b[0m\n");

    // Obtener el comando de instalación desde la página de la skill
    let install_cmd = match get_install_command(install_path) {
        Ok(cmd) => {
            println!("Comando encontrado: {}\n", cmd);
            cmd
        }
        Err(e) => {
            eprintln!("No se pudo obtener comando automáticamente: {}", e);
            // Intentar formato por defecto como fallback
            let parts: Vec<&str> = install_path.split('/').collect();
            if parts.len() < 3 {
                eprintln!("Formato de skill inválido: {}", install_path);
                return Ok(());
            }
            format!(
                "npx skills add https://github.com/{}/{} --skill {}",
                parts[0], parts[1], parts[2]
            )
        }
    };

    // Extraer los argumentos del comando
    // El comando puede ser: npx skills add https://github.com/author/repo --skill name
    // o: npx skills add author@repo --skill name
    let args: Vec<&str> = install_cmd.split_whitespace().collect();

    // Reconstruir argumentos
    let mut final_args = vec!["skills", "add"];
    let mut skip_next = false;
    let mut has_skill = false;

    for (i, arg) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if *arg == "add" {
            continue;
        }
        if *arg == "skills" {
            continue;
        }
        if *arg == "npx" {
            continue;
        }

        // Para --skill, incluir el valor siguiente
        if *arg == "--skill" {
            final_args.push(arg);
            has_skill = true;
            // Agregar el siguiente argumento (el nombre de la skill)
            if i + 1 < args.len() {
                let next = args[i + 1];
                if !next.starts_with("--") && !next.is_empty() {
                    final_args.push(next);
                    skip_next = true;
                }
            }
        } else if arg.starts_with("http") || arg.contains('@') {
            final_args.push(arg);
        } else if arg.starts_with("--") {
            final_args.push(arg);
        }
    }

    // Si tiene --skill, también agregar -y para saltarse la selección de skills
    // pero mantener el selector de agentes
    if has_skill {
        final_args.push("-y");
    }

    println!("Ejecutando: npx {}\n", final_args.join(" "));

    // Ejecutar el comando
    let output = Command::new("npx").args(&final_args).output()?;

    // Mostrar output del comando
    if !output.stdout.is_empty() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if output.status.success() {
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
