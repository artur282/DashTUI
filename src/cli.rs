//! Módulo CLI — Definición de la interfaz de línea de comandos.
//!
//! Dado que la herramienta ahora es 100% orientada al dashboard (TUI),
//! la CLI base simplemente inicializa la aplicación.

use clap::Parser;

/// 🦀 DashTUI - Herramienta interactiva TUI de alto rendimiento
#[derive(Parser, Debug)]
#[command(
    name = "dashtui",
    version,
    about = "Dashboard TUI interactivo para desarrolladores. Launching dashtui opens the dash.",
    author = "Luis"
)]
pub struct Cli {
    // Sin subcomandos, el flujo se centralizó en el Dashboard TUI.
    // Futuras flags locales (tipo --verbose) pueden ubicarse aquí.
}
