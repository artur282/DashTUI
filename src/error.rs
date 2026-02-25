//! Tipos de error personalizados para DashTUI.
//!
//! Define una jerarquía de errores específicos por dominio,
//! usando `thiserror` para implementar `Display` y `From` automáticamente.

use thiserror::Error;

/// Error principal de la aplicación que agrupa todos los errores de dominio.
///
/// Cada variante envuelve un error específico del módulo correspondiente,
/// permitiendo propagación idiomática con el operador `?`.
#[derive(Debug, Error)]
pub enum AppError {
    /// Error relacionado con operaciones de scaffold/init
    #[error("Error de scaffold: {0}")]
    Scaffold(#[from] ScaffoldError),

    /// Error relacionado con operaciones de snippets
    #[error("Error de snippet: {0}")]
    Snippet(#[from] SnippetError),

    /// Error relacionado con operaciones Git
    #[error("Error de Git: {0}")]
    Git(#[from] GitError),

    /// Error relacionado con la base de datos SQLite
    #[error("Error de base de datos: {0}")]
    Database(#[from] DatabaseError),

    /// Error relacionado con el dashboard TUI
    #[error("Error de dashboard: {0}")]
    Dashboard(#[from] DashboardError),

    /// Error relacionado con las skills de skills.sh
    #[error("Error de skills: {0}")]
    Skills(#[from] SkillsError),

    /// Error de entrada/salida del sistema de archivos
    #[error("Error de IO: {0}")]
    Io(#[from] std::io::Error),
}

/// Errores específicos del comando `init` (scaffold de proyectos).
#[derive(Debug, Error)]
pub enum ScaffoldError {
    /// El directorio de destino ya existe y no está vacío
    #[error("El directorio '{0}' ya existe y no está vacío")]
    DirectoryExists(String),

    /// Template no reconocido o no disponible
    #[error("Template '{0}' no encontrado. Usa los templates disponibles del catálogo.")]
    TemplateNotFound(String),

    /// Error de IO durante el scaffold
    #[error("Error de IO: {0}")]
    Io(#[from] std::io::Error),
}

/// Errores específicos del módulo de snippets.
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum SnippetError {
    /// El snippet con ese nombre ya existe
    #[error("El snippet '{0}' ya existe")]
    AlreadyExists(String),

    /// El snippet solicitado no fue encontrado
    #[error("Snippet '{0}' no encontrado")]
    NotFound(String),

    /// Error al acceder al clipboard del sistema
    #[error("Error de clipboard: {0}")]
    ClipboardError(String),

    /// Error de la base de datos subyacente
    #[error("Error de base de datos: {0}")]
    Database(#[from] DatabaseError),
}

/// Errores específicos de las utilidades Git.
#[derive(Debug, Error)]
pub enum GitError {
    /// No se encontró un repositorio Git en el directorio actual
    #[error("No se encontró repositorio Git en el directorio actual")]
    NoRepository,

    /// Error de la librería git2
    #[error("Error de Git: {0}")]
    LibGit(#[from] git2::Error),

    /// No hay commits convencionales para generar changelog
    #[error("No se encontraron commits convencionales para el changelog")]
    NoConventionalCommits,
}

/// Errores específicos de la base de datos SQLite.
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// Error de rusqlite durante operaciones SQL
    #[error("Error SQLite: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// No se pudo determinar el directorio de datos del usuario
    #[error("No se pudo determinar el directorio de datos de la aplicación")]
    NoDataDir,

    /// Error de IO al acceder al archivo de base de datos
    #[error("Error de IO en base de datos: {0}")]
    Io(#[from] std::io::Error),
}

/// Errores específicos del dashboard TUI.
#[derive(Debug, Error)]
pub enum DashboardError {
    /// Error del backend de terminal (crossterm)
    #[error("Error de terminal: {0}")]
    Terminal(#[from] std::io::Error),

    /// Error al renderizar widgets de Ratatui
    #[error("Error de renderizado: {0}")]
    Render(String),
}

/// Errores específicos de la integración con skills.sh.
#[derive(Debug, Error)]
pub enum SkillsError {
    /// Error de conexión HTTP al hacer scraping de skills.sh
    #[error("Error al hacer la solicitud de web scraping: {0}")]
    Network(String),

    /// Error al parsear el HTML de la respuesta
    #[error("Error al parsear el HTML: {0}")]
    ParseError(String),
}

/// Tipo Result genérico de la aplicación para simplificar firmas de funciones.
pub type Result<T> = std::result::Result<T, AppError>;
