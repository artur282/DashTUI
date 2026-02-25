//! Módulo de comandos de DashTUI.
//!
//! Agrupa todos los subcomandos disponibles:
//! - `init`: Scaffold de proyectos desde templates con Docker Compose
//! - `git_utils`: Utilidades para repositorios Git
//! - `dashboard`: Dashboard interactivo en terminal
//! - `skills`: Integración con skills.sh (búsqueda e instalación)

pub mod dashboard;
pub mod git_utils;
pub mod init;
pub mod skills;
