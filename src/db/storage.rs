//! Operaciones de almacenamiento SQLite para snippets y tareas.
//!
//! Gestiona la conexión a la base de datos, la creación de tablas
//! y todas las operaciones CRUD sobre snippets y tareas del dashboard.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::DatabaseError;

/// Representa un snippet de código almacenado en la base de datos.
///
/// Contiene el nombre identificador, el código fuente, el lenguaje
/// y una descripción opcional del snippet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    /// Identificador único del snippet
    pub id: Option<i64>,
    /// Nombre descriptivo del snippet
    pub name: String,
    /// Código fuente del snippet
    pub code: String,
    /// Lenguaje de programación
    pub language: String,
    /// Descripción breve del propósito del snippet
    pub description: String,
    /// Fecha de creación en formato ISO 8601
    pub created_at: String,
}

/// Representa una tarea del dashboard interactivo.
///
/// Las tareas se almacenan persistentemente y se gestionan
/// desde la interfaz TUI con navegación por teclado.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Identificador único de la tarea
    pub id: Option<i64>,
    /// Título descriptivo de la tarea
    pub title: String,
    /// Indica si la tarea está completada
    pub completed: bool,
    /// Fecha de creación en formato ISO 8601
    pub created_at: String,
}

/// Almacenamiento persistente usando SQLite embebido.
///
/// Encapsula la conexión a la base de datos y expone operaciones
/// CRUD para snippets y tareas del dashboard.
pub struct Storage {
    /// Conexión activa a la base de datos SQLite
    conn: Connection,
}

impl Storage {
    /// Crea una nueva instancia de Storage y garantiza que las tablas existan.
    ///
    /// # Errors
    ///
    /// Retorna `DatabaseError` si no se puede abrir la base de datos
    /// o si falla la creación de las tablas.
    pub fn new() -> std::result::Result<Self, DatabaseError> {
        let db_path = get_database_path()?;
        let conn = Connection::open(&db_path)?;
        let storage = Self { conn };
        storage.initialize_tables()?;
        Ok(storage)
    }

    /// Crea una instancia de Storage usando una base de datos en memoria.
    ///
    /// Útil para testing — los datos no se persisten al disco.
    ///
    /// # Errors
    ///
    /// Retorna `DatabaseError` si falla la creación de las tablas en memoria.
    #[cfg(test)]
    pub fn new_in_memory() -> std::result::Result<Self, DatabaseError> {
        let conn = Connection::open_in_memory()?;
        let storage = Self { conn };
        storage.initialize_tables()?;
        Ok(storage)
    }

    /// Inicializa las tablas necesarias si no existen.
    ///
    /// Crea las tablas `snippets` y `tasks` con sus esquemas completos.
    fn initialize_tables(&self) -> std::result::Result<(), DatabaseError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS snippets (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL UNIQUE,
                code        TEXT NOT NULL,
                language    TEXT NOT NULL DEFAULT 'text',
                description TEXT NOT NULL DEFAULT '',
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                title       TEXT NOT NULL,
                completed   INTEGER NOT NULL DEFAULT 0,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;
        Ok(())
    }

    // ─── Operaciones CRUD de Snippets ───────────────────────────────

    /// Inserta un nuevo snippet en la base de datos.
    ///
    /// # Errors
    ///
    /// Retorna `DatabaseError::Sqlite` si el nombre ya existe (UNIQUE constraint).
    pub fn add_snippet(
        &self,
        name: &str,
        code: &str,
        language: &str,
        description: &str,
    ) -> std::result::Result<(), DatabaseError> {
        self.conn.execute(
            "INSERT INTO snippets (name, code, language, description)
             VALUES (?1, ?2, ?3, ?4)",
            params![name, code, language, description],
        )?;
        Ok(())
    }

    /// Obtiene todos los snippets, opcionalmente filtrados por búsqueda.
    ///
    /// Realiza una búsqueda fuzzy en el nombre y la descripción
    /// del snippet usando LIKE con comodines.
    ///
    /// # Errors
    ///
    /// Retorna `DatabaseError::Sqlite` si falla la consulta.
    pub fn list_snippets(
        &self,
        query: Option<&str>,
    ) -> std::result::Result<Vec<Snippet>, DatabaseError> {
        let mut stmt = match query {
            Some(q) => {
                let pattern = format!("%{q}%");
                let mut stmt = self.conn.prepare(
                    "SELECT id, name, code, language, description, created_at
                     FROM snippets
                     WHERE name LIKE ?1 OR description LIKE ?1
                     ORDER BY created_at DESC",
                )?;
                let snippets = stmt
                    .query_map(params![pattern], map_snippet_row)?
                    .filter_map(|r| r.ok())
                    .collect();
                return Ok(snippets);
            }
            None => self.conn.prepare(
                "SELECT id, name, code, language, description, created_at
                 FROM snippets
                 ORDER BY created_at DESC",
            )?,
        };

        let snippets = stmt
            .query_map([], map_snippet_row)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(snippets)
    }

    /// Obtiene un snippet específico por su nombre.
    ///
    /// # Errors
    ///
    /// Retorna `DatabaseError::Sqlite` si el snippet no existe.
    #[allow(dead_code)]
    pub fn get_snippet(&self, name: &str) -> std::result::Result<Option<Snippet>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, code, language, description, created_at
             FROM snippets
             WHERE name = ?1",
        )?;

        let snippet = stmt
            .query_map(params![name], map_snippet_row)?
            .filter_map(|r| r.ok())
            .next();

        Ok(snippet)
    }

    /// Elimina un snippet por su nombre.
    ///
    /// # Errors
    ///
    /// Retorna `DatabaseError::Sqlite` si falla la operación DELETE.
    pub fn delete_snippet(&self, name: &str) -> std::result::Result<bool, DatabaseError> {
        let rows_affected = self
            .conn
            .execute("DELETE FROM snippets WHERE name = ?1", params![name])?;
        Ok(rows_affected > 0)
    }

    // ─── Operaciones CRUD de Tareas ─────────────────────────────────

    /// Inserta una nueva tarea en la base de datos.
    ///
    /// # Errors
    ///
    /// Retorna `DatabaseError::Sqlite` si falla la inserción.
    pub fn add_task(&self, title: &str) -> std::result::Result<(), DatabaseError> {
        self.conn
            .execute("INSERT INTO tasks (title) VALUES (?1)", params![title])?;
        Ok(())
    }

    /// Obtiene todas las tareas ordenadas por fecha de creación.
    ///
    /// # Errors
    ///
    /// Retorna `DatabaseError::Sqlite` si falla la consulta.
    pub fn list_tasks(&self) -> std::result::Result<Vec<Task>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, completed, created_at
             FROM tasks
             ORDER BY completed ASC, created_at DESC",
        )?;

        let tasks = stmt
            .query_map([], |row| {
                Ok(Task {
                    id: Some(row.get(0)?),
                    title: row.get(1)?,
                    completed: row.get::<_, i32>(2)? != 0,
                    created_at: row.get(3)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(tasks)
    }

    /// Alterna el estado de completado de una tarea.
    ///
    /// # Errors
    ///
    /// Retorna `DatabaseError::Sqlite` si la tarea no existe.
    pub fn toggle_task(&self, task_id: i64) -> std::result::Result<(), DatabaseError> {
        self.conn.execute(
            "UPDATE tasks SET completed = NOT completed WHERE id = ?1",
            params![task_id],
        )?;
        Ok(())
    }

    /// Elimina una tarea por su ID.
    ///
    /// # Errors
    ///
    /// Retorna `DatabaseError::Sqlite` si falla la operación DELETE.
    pub fn delete_task(&self, task_id: i64) -> std::result::Result<bool, DatabaseError> {
        let rows_affected = self
            .conn
            .execute("DELETE FROM tasks WHERE id = ?1", params![task_id])?;
        Ok(rows_affected > 0)
    }
}

/// Mapea una fila de la tabla `snippets` a la estructura `Snippet`.
fn map_snippet_row(row: &rusqlite::Row) -> rusqlite::Result<Snippet> {
    Ok(Snippet {
        id: Some(row.get(0)?),
        name: row.get(1)?,
        code: row.get(2)?,
        language: row.get(3)?,
        description: row.get(4)?,
        created_at: row.get(5)?,
    })
}

/// Determina la ruta del archivo de base de datos SQLite.
///
/// Usa `dirs::data_dir()` para almacenar la DB en el directorio
/// estándar de datos del usuario (~/.local/share/rustcli/).
fn get_database_path() -> std::result::Result<std::path::PathBuf, DatabaseError> {
    let data_dir = dirs::data_dir().ok_or(DatabaseError::NoDataDir)?;

    let app_dir = data_dir.join("rustcli");
    std::fs::create_dir_all(&app_dir)?;

    Ok(app_dir.join("rustcli.db"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_should_create_tables_on_init() {
        let storage = Storage::new_in_memory().expect("Fallo al crear storage en memoria");

        // Verificar que las tablas existen consultándolas
        let snippet_count: i64 = storage
            .conn
            .query_row("SELECT COUNT(*) FROM snippets", [], |row| row.get(0))
            .expect("Tabla snippets no existe");
        assert_eq!(snippet_count, 0);

        let task_count: i64 = storage
            .conn
            .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
            .expect("Tabla tasks no existe");
        assert_eq!(task_count, 0);
    }

    #[test]
    fn add_snippet_should_persist_data() {
        let storage = Storage::new_in_memory().expect("Fallo al crear storage");

        storage
            .add_snippet("test", "fn main() {}", "rust", "Test snippet")
            .expect("Fallo al agregar snippet");

        let snippets = storage
            .list_snippets(None)
            .expect("Fallo al listar snippets");
        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].name, "test");
        assert_eq!(snippets[0].language, "rust");
    }

    #[test]
    fn get_snippet_should_return_matching_snippet() {
        let storage = Storage::new_in_memory().expect("Fallo al crear storage");

        storage
            .add_snippet("hello", "println!(\"hi\")", "rust", "Saludo")
            .expect("Fallo al agregar snippet");

        let snippet = storage
            .get_snippet("hello")
            .expect("Fallo al buscar snippet");
        assert!(snippet.is_some());
        assert_eq!(snippet.as_ref().map(|s| s.name.as_str()), Some("hello"));
    }

    #[test]
    fn get_snippet_should_return_none_for_missing() {
        let storage = Storage::new_in_memory().expect("Fallo al crear storage");

        let result = storage
            .get_snippet("no_existe")
            .expect("Fallo al buscar snippet");
        assert!(result.is_none());
    }

    #[test]
    fn delete_snippet_should_remove_entry() {
        let storage = Storage::new_in_memory().expect("Fallo al crear storage");

        storage
            .add_snippet("temporal", "let x = 1;", "rust", "Temp")
            .expect("Fallo al agregar snippet");

        let deleted = storage
            .delete_snippet("temporal")
            .expect("Fallo al eliminar snippet");
        assert!(deleted);

        let result = storage
            .get_snippet("temporal")
            .expect("Fallo al buscar snippet");
        assert!(result.is_none());
    }

    #[test]
    fn list_snippets_should_filter_by_query() {
        let storage = Storage::new_in_memory().expect("Fallo al crear storage");

        storage
            .add_snippet("rust-hello", "fn main() {}", "rust", "Saludo en Rust")
            .expect("Fallo al agregar snippet");
        storage
            .add_snippet("python-hello", "print('hi')", "python", "Saludo en Python")
            .expect("Fallo al agregar snippet");

        let results = storage
            .list_snippets(Some("rust"))
            .expect("Fallo al filtrar snippets");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "rust-hello");
    }

    #[test]
    fn add_task_should_persist_data() {
        let storage = Storage::new_in_memory().expect("Fallo al crear storage");

        storage
            .add_task("Implementar feature X")
            .expect("Fallo al agregar tarea");

        let tasks = storage.list_tasks().expect("Fallo al listar tareas");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Implementar feature X");
        assert!(!tasks[0].completed);
    }

    #[test]
    fn toggle_task_should_switch_completed_state() {
        let storage = Storage::new_in_memory().expect("Fallo al crear storage");

        storage
            .add_task("Tarea toggle")
            .expect("Fallo al agregar tarea");

        let tasks = storage.list_tasks().expect("Fallo al listar tareas");
        let task_id = tasks[0].id.expect("Tarea sin ID");
        assert!(!tasks[0].completed);

        storage.toggle_task(task_id).expect("Fallo al toggle tarea");

        let tasks = storage.list_tasks().expect("Fallo al listar tareas");
        assert!(tasks[0].completed);
    }
}
