//! Estado central de la aplicación TUI.
//!
//! Contiene la estructura `App` que mantiene el estado global del dashboard,
//! incluyendo tabs, tareas, pomodoro, scaffold, snippets, git y skills.

use crate::commands::git_utils::{ChangelogEntry, GitStats};
use crate::commands::skills::Skill;
use crate::db::storage::{Snippet, Storage, Task};
use std::time::Duration;

/// Pestañas disponibles en el dashboard TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Tasks,
    Pomodoro,
    Scaffold,
    Snippets,
    Git,
    Skills,
}

/// Modos de entrada del usuario (modales de texto)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    None,
    TaskAdd,
    ProjectName,
    SnippetTitle,
    SnippetLanguage,
    SnippetDescription,
    SkillSearch,
}

/// Estados del timer Pomodoro
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PomodoroState {
    Work,
    ShortBreak,
    LongBreak,
}

/// Estado central de la aplicación que contiene todos los datos del dashboard.
pub struct App {
    pub active_tab: Tab,
    pub should_quit: bool,
    pub input_mode: InputMode,
    pub input_buffer: String,

    // ── Overview data ──
    pub project_name: String,
    pub _file_count: usize,

    // ── Task data ──
    pub tasks: Vec<Task>,
    pub selected_task_index: usize,

    // ── Pomodoro data ──
    pub pomodoro: PomodoroState,
    pub pomodoro_timer: Duration,
    pub pomodoro_running: bool,
    pub pomodoros_completed: u32,

    // ── Scaffold data ──
    pub selected_template_index: usize,
    pub scaffold_logs: Option<Vec<String>>,

    // ── Snippets data ──
    pub snippets: Vec<Snippet>,
    pub selected_snippet_index: usize,
    pub new_snippet_draft: (String, String, String), // (Title, Lang, Desc)

    // ── Git data ──
    pub git_stats: Option<GitStats>,
    pub git_changelog: Option<Vec<ChangelogEntry>>,
    pub git_merged_branches: Option<Vec<String>>,
    pub git_error: Option<String>,

    // ── Skills data (skills.sh) ──
    /// Lista de skills obtenidas del scraping de skills.sh
    pub skills_results: Vec<Skill>,
    /// Índice de la skill seleccionada en la lista
    pub selected_skill_index: usize,
    /// Último query de búsqueda utilizado
    pub skills_search_query: String,
    /// Mensaje de estado de la operación de skills
    pub skills_status: Option<String>,
    /// Logs de la última instalación de skill
    pub skills_install_logs: Option<Vec<String>>,
    /// Indica si se está cargando una operación de skills
    pub skills_loading: bool,

    // ── Storage connection ──
    storage: Option<Storage>,
}

impl App {
    /// Crea una nueva instancia de App con valores por defecto.
    ///
    /// Inicializa la conexión a la base de datos SQLite y carga
    /// las tareas y snippets almacenados.
    pub fn new() -> Self {
        let mut app = Self {
            active_tab: Tab::Overview,
            should_quit: false,
            input_mode: InputMode::None,
            input_buffer: String::new(),

            project_name: std::env::current_dir()
                .unwrap_or_default()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            _file_count: crate::commands::git_utils::get_stats()
                .map(|s| s.tracked_files)
                .unwrap_or(0),

            tasks: Vec::new(),
            selected_task_index: 0,

            pomodoro: PomodoroState::Work,
            pomodoro_timer: Duration::from_secs(25 * 60),
            pomodoro_running: false,
            pomodoros_completed: 0,

            selected_template_index: 0,
            scaffold_logs: None,

            snippets: Vec::new(),
            selected_snippet_index: 0,
            new_snippet_draft: (String::new(), String::new(), String::new()),

            git_stats: None,
            git_changelog: None,
            git_merged_branches: None,
            git_error: None,

            // Skills inicializados vacíos
            skills_results: Vec::new(),
            selected_skill_index: 0,
            skills_search_query: String::new(),
            skills_status: None,
            skills_install_logs: None,
            skills_loading: false,

            storage: None,
        };

        // Inicializar base de datos y cargar datos persistidos
        match Storage::new() {
            Ok(storage) => {
                app.tasks = storage.list_tasks().unwrap_or_default();
                app.snippets = storage.list_snippets(None).unwrap_or_default();
                app.storage = Some(storage);
            }
            Err(e) => eprintln!("Error inicializando DB local: {}", e),
        }

        app
    }

    // ── Modos de entrada ─────────────────────────────────────────────────

    /// Entra en un modo de entrada específico y limpia el buffer.
    pub fn enter_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
        self.input_buffer.clear();
    }

    /// Sale del modo de entrada actual y limpia el buffer.
    pub fn exit_input_mode(&mut self) {
        self.input_mode = InputMode::None;
        self.input_buffer.clear();
    }

    // ── Datos ────────────────────────────────────────────────────────────

    /// Recarga tareas y snippets desde la base de datos.
    pub fn refresh_data(&mut self) {
        if let Some(ref storage) = self.storage {
            self.tasks = storage.list_tasks().unwrap_or_default();
            self.snippets = storage.list_snippets(None).unwrap_or_default();
        }
    }

    /// Carga estadísticas y changelog desde el repositorio Git local.
    pub fn load_git_data(&mut self) {
        match crate::commands::git_utils::get_stats() {
            Ok(s) => self.git_stats = Some(s),
            Err(e) => {
                self.git_error = Some(e.to_string());
                return;
            }
        }
        if let Ok(c) = crate::commands::git_utils::get_changelog(100) {
            self.git_changelog = Some(c);
        }
        if let Ok(b) = crate::commands::git_utils::get_merged_branches() {
            self.git_merged_branches = Some(b);
        }
    }

    // ── Pomodoro ─────────────────────────────────────────────────────────

    /// Avanza el timer del pomodoro en un tick (1 segundo).
    ///
    /// Cuando el timer llega a cero, transiciona automáticamente
    /// entre estados Work → ShortBreak/LongBreak → Work.
    pub fn on_tick(&mut self) {
        if self.pomodoro_running {
            if self.pomodoro_timer.as_secs() > 0 {
                self.pomodoro_timer -= Duration::from_secs(1);
            } else {
                self.pomodoro_running = false;
                self.pomodoros_completed += 1;
                match self.pomodoro {
                    PomodoroState::Work => {
                        if self.pomodoros_completed.is_multiple_of(4) {
                            self.pomodoro = PomodoroState::LongBreak;
                            self.pomodoro_timer = Duration::from_secs(15 * 60);
                        } else {
                            self.pomodoro = PomodoroState::ShortBreak;
                            self.pomodoro_timer = Duration::from_secs(5 * 60);
                        }
                    }
                    _ => {
                        self.pomodoro = PomodoroState::Work;
                        self.pomodoro_timer = Duration::from_secs(25 * 60);
                    }
                }
            }
        }
    }

    // ── Navegación entre tabs ────────────────────────────────────────────

    /// Avanza a la siguiente pestaña (circular).
    pub fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::Overview => Tab::Tasks,
            Tab::Tasks => Tab::Pomodoro,
            Tab::Pomodoro => Tab::Scaffold,
            Tab::Scaffold => Tab::Snippets,
            Tab::Snippets => Tab::Git,
            Tab::Git => Tab::Skills,
            Tab::Skills => Tab::Overview,
        };
        // Carga lazy de datos de Git
        if self.active_tab == Tab::Git && self.git_stats.is_none() {
            self.load_git_data();
        }
    }

    /// Retrocede a la pestaña anterior (circular).
    pub fn prev_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::Overview => Tab::Skills,
            Tab::Tasks => Tab::Overview,
            Tab::Pomodoro => Tab::Tasks,
            Tab::Scaffold => Tab::Pomodoro,
            Tab::Snippets => Tab::Scaffold,
            Tab::Git => Tab::Snippets,
            Tab::Skills => Tab::Git,
        };
        if self.active_tab == Tab::Git && self.git_stats.is_none() {
            self.load_git_data();
        }
    }

    // ── Acciones de Tasks ────────────────────────────────────────────────

    /// Agrega una nueva tarea desde el buffer de entrada.
    pub fn add_task(&mut self) {
        let desc = self.input_buffer.trim().to_string();
        if !desc.is_empty() {
            if let Some(ref storage) = self.storage {
                if storage.add_task(&desc).is_ok() {
                    self.refresh_data();
                }
            }
        }
        self.exit_input_mode();
    }

    /// Alterna el estado completado/pendiente de la tarea seleccionada.
    pub fn toggle_task(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        if let Some(task) = self.tasks.get(self.selected_task_index) {
            if let Some(ref storage) = self.storage {
                if let Some(id) = task.id {
                    if storage.toggle_task(id).is_ok() {
                        self.refresh_data();
                    }
                }
            }
        }
    }

    /// Elimina la tarea seleccionada.
    pub fn delete_task(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        if let Some(task) = self.tasks.get(self.selected_task_index) {
            if let Some(ref storage) = self.storage {
                if let Some(id) = task.id {
                    if storage.delete_task(id).is_ok() {
                        self.refresh_data();
                        if self.selected_task_index >= self.tasks.len()
                            && self.selected_task_index > 0
                        {
                            self.selected_task_index -= 1;
                        }
                    }
                }
            }
        }
    }

    // ── Acciones de Snippets ─────────────────────────────────────────────

    /// Elimina el snippet seleccionado.
    pub fn delete_snippet(&mut self) {
        if self.snippets.is_empty() {
            return;
        }
        if let Some(snippet) = self.snippets.get(self.selected_snippet_index) {
            if let Some(ref storage) = self.storage {
                if storage.delete_snippet(&snippet.name).is_ok() {
                    self.refresh_data();
                    if self.selected_snippet_index >= self.snippets.len()
                        && self.selected_snippet_index > 0
                    {
                        self.selected_snippet_index -= 1;
                    }
                }
            }
        }
    }

    /// Copia el código del snippet seleccionado al clipboard del sistema.
    pub fn copy_snippet(&mut self) {
        if self.snippets.is_empty() {
            return;
        }
        if let Some(snippet) = self.snippets.get(self.selected_snippet_index) {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(snippet.code.clone());
            }
        }
    }

    // ── Acciones de Scaffold ─────────────────────────────────────────────

    /// Ejecuta el scaffold del template seleccionado con el nombre del buffer.
    pub fn execute_scaffold(&mut self) {
        let name = if self.input_buffer.trim().is_empty() {
            None
        } else {
            Some(self.input_buffer.trim())
        };

        let template = crate::commands::init::AVAILABLE_TEMPLATES[self.selected_template_index];
        match crate::commands::init::execute(template, name) {
            Ok(logs) => self.scaffold_logs = Some(logs),
            Err(e) => self.scaffold_logs = Some(vec![format!("❌ Error: {}", e)]),
        }
        self.exit_input_mode();
    }

    /// Guarda un snippet leyendo el código desde el clipboard del sistema.
    pub fn save_snippet_from_clipboard(&mut self) {
        let code = if let Ok(mut clipboard) = arboard::Clipboard::new() {
            clipboard.get_text().unwrap_or_default()
        } else {
            String::new()
        };

        if code.is_empty() {
            self.exit_input_mode();
            return;
        }

        if let Some(ref storage) = self.storage {
            if storage
                .add_snippet(
                    &self.new_snippet_draft.0,
                    &code,
                    &self.new_snippet_draft.1,
                    &self.new_snippet_draft.2,
                )
                .is_ok()
            {
                self.refresh_data();
            }
        }
        self.exit_input_mode();
    }

    // ── Acciones de Git ──────────────────────────────────────────────────

    /// Limpia las ramas locales ya mergeadas.
    pub fn clean_git_branches(&mut self) {
        if let Some(ref branches) = self.git_merged_branches {
            let clones = branches.clone();
            let _ = crate::commands::git_utils::delete_branches(&clones);
            self.load_git_data();
        }
    }

    // ── Acciones de Skills ───────────────────────────────────────────────

    /// Busca skills en skills.sh usando el query del buffer de entrada.
    ///
    /// Realiza web scraping del sitio y filtra los resultados
    /// por nombre/autor según el query proporcionado.
    pub fn search_skills(&mut self) {
        let query = self.input_buffer.trim().to_string();
        self.skills_search_query = query.clone();
        self.skills_loading = true;
        self.skills_install_logs = None;

        if query.is_empty() {
            // Sin query: cargar todas las skills del leaderboard
            match crate::commands::skills::fetch_all_skills() {
                Ok(skills) => {
                    self.skills_status = Some(format!("✅ {} skills encontradas", skills.len()));
                    self.skills_results = skills;
                    self.selected_skill_index = 0;
                }
                Err(e) => {
                    self.skills_status = Some(format!("❌ Error: {}", e));
                    self.skills_results.clear();
                }
            }
        } else {
            // Con query: buscar y filtrar
            match crate::commands::skills::search_skills(&query) {
                Ok(skills) => {
                    if skills.is_empty() {
                        self.skills_status = Some(format!(
                            "❓ 0 skills para '{}'. Presiona 'l' para leaderboard",
                            query
                        ));
                    } else {
                        self.skills_status =
                            Some(format!("✅ {} skills para '{}'", skills.len(), query));
                    }
                    self.skills_results = skills;
                    self.selected_skill_index = 0;
                }
                Err(e) => {
                    self.skills_status = Some(format!("❌ Error: {}", e));
                    self.skills_results.clear();
                }
            }
        }

        self.skills_loading = false;
        self.exit_input_mode();
    }
}
