//! Módulo de comandos de RustCLI.
//!
//! Agrupa todos los subcomandos disponibles:
//! - `init`: Scaffold de proyectos desde templates
//! - `snippet`: Gestión de fragmentos de código
//! - `git_utils`: Utilidades para repositorios Git
//! - `dashboard`: Dashboard interactivo en terminal

pub mod dashboard;
pub mod git_utils;
pub mod init;
