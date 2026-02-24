//! RustCLI — Entry point principal de la aplicación.
//!
//! Parseo de la CLI inicial y redirección al Dashboard.
//!
//! # Arquitectura
//! - **`cli`**: Definición de la interfaz usando Clap (derive).
//! - **`commands`**: Lógica de cada módulo (Scaffold, Snippet, Git).
//! - **`db`**: Capa de abstracción SQLite para persistencia.
//! - **`tui`**: Dashboard central que consume los módulos anteriores interactuando con Crossterm/Ratatui.
//! - **`error`**: Tipos de error `thiserror` centralizados.

mod cli;
mod commands;
mod db;
mod error;
mod tui;

use clap::Parser;
use colored::Colorize;

use crate::cli::Cli;
use crate::error::Result;

/// Entry point principal. Inicia el CLI y captura errores root.
fn main() {
    let _cli = Cli::parse(); // Parseamos por si agregamos flags globales a futuro

    // Iniciar el dashboard (UI maestro)
    if let Err(e) = run() {
        eprintln!(
            "\n{} {}\n",
            "❌ Fallo crítico en el Dashboard:".red().bold(),
            e.to_string().red()
        );
        std::process::exit(1);
    }
}

/// Función auxiliar para propagar el `Result` globalmente.
fn run() -> Result<()> {
    commands::dashboard::execute()?;
    Ok(())
}
