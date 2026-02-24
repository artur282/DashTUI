//! Estado central de la aplicación TUI.
use crate::commands::git_utils::{ChangelogEntry, GitStats};
use crate::db::storage::{Snippet, Storage, Task};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Tasks,
    Pomodoro,
    Scaffold,
    Snippets,
    Git,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    None,
    TaskAdd,
    ProjectName,
    SnippetTitle,
    SnippetLanguage,
    SnippetDescription,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PomodoroState {
    Work,
    ShortBreak,
    LongBreak,
}

pub struct App {
    pub active_tab: Tab,
    pub should_quit: bool,
    pub input_mode: InputMode,
    pub input_buffer: String,

    // Overview data
    pub project_name: String,
    pub _file_count: usize,

    // Task data
    pub tasks: Vec<Task>,
    pub selected_task_index: usize,

    // Pomodoro data
    pub pomodoro: PomodoroState,
    pub pomodoro_timer: Duration,
    pub pomodoro_running: bool,
    pub pomodoros_completed: u32,

    // Scaffold data
    pub selected_template_index: usize,
    pub scaffold_logs: Option<Vec<String>>,

    // Snippets data
    pub snippets: Vec<Snippet>,
    pub selected_snippet_index: usize,
    pub new_snippet_draft: (String, String, String), // (Title, Lang, Desc)

    // Git data
    pub git_stats: Option<GitStats>,
    pub git_changelog: Option<Vec<ChangelogEntry>>,
    pub git_merged_branches: Option<Vec<String>>,
    pub git_error: Option<String>,

    // Storage connection
    storage: Option<Storage>,
}

impl App {
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

            storage: None,
        };

        match Storage::new() {
            Ok(ref mut storage) => {
                app.tasks = storage.list_tasks().unwrap_or_default();
                app.snippets = storage.list_snippets(None).unwrap_or_default();
                // Move instead of clone ref
                app.storage = Storage::new().ok();
            }
            Err(e) => eprintln!("Error inicializando DB local: {}", e),
        }

        app
    }

    pub fn enter_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
        self.input_buffer.clear();
    }

    pub fn exit_input_mode(&mut self) {
        self.input_mode = InputMode::None;
        self.input_buffer.clear();
    }

    pub fn refresh_data(&mut self) {
        if let Some(ref storage) = self.storage {
            self.tasks = storage.list_tasks().unwrap_or_default();
            self.snippets = storage.list_snippets(None).unwrap_or_default();
        }
    }

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

    pub fn on_tick(&mut self) {
        if self.pomodoro_running {
            if self.pomodoro_timer.as_secs() > 0 {
                self.pomodoro_timer -= Duration::from_secs(1);
            } else {
                self.pomodoro_running = false;
                self.pomodoros_completed += 1;
                // Transition state
                match self.pomodoro {
                    PomodoroState::Work => {
                        if self.pomodoros_completed % 4 == 0 {
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

    pub fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::Overview => Tab::Tasks,
            Tab::Tasks => Tab::Pomodoro,
            Tab::Pomodoro => Tab::Scaffold,
            Tab::Scaffold => Tab::Snippets,
            Tab::Snippets => Tab::Git,
            Tab::Git => Tab::Overview,
        };
        if self.active_tab == Tab::Git && self.git_stats.is_none() {
            self.load_git_data();
        }
    }

    pub fn prev_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::Overview => Tab::Git,
            Tab::Tasks => Tab::Overview,
            Tab::Pomodoro => Tab::Tasks,
            Tab::Scaffold => Tab::Pomodoro,
            Tab::Snippets => Tab::Scaffold,
            Tab::Git => Tab::Snippets,
        };
        if self.active_tab == Tab::Git && self.git_stats.is_none() {
            self.load_git_data();
        }
    }

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
                    &self.new_snippet_draft.1,
                    &self.new_snippet_draft.2,
                    &code,
                )
                .is_ok()
            {
                self.refresh_data();
            }
        }
        self.exit_input_mode();
    }

    pub fn clean_git_branches(&mut self) {
        if let Some(ref branches) = self.git_merged_branches {
            let clones = branches.clone();
            let _ = crate::commands::git_utils::delete_branches(&clones);
            self.load_git_data(); // reload
        }
    }
}
