//! Dashboard TUI interactivo maestro — event loop y manejo de teclas.
//!
//! Controla el ciclo principal de renderizado, captura de eventos
//! y despacho de acciones a cada pestaña activa.

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::error::DashboardError;
use crate::tui::app::{App, InputMode, PomodoroState, Tab};
use crate::tui::ui::render;

/// Punto de entrada del dashboard — configura terminal y ejecuta el event loop.
pub fn execute() -> Result<(), DashboardError> {
    enable_raw_mode().map_err(DashboardError::Terminal)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(DashboardError::Terminal)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| DashboardError::Render(e.to_string()))?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

    // Restaurar terminal al estado original
    disable_raw_mode().map_err(DashboardError::Terminal)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(DashboardError::Terminal)?;
    terminal
        .show_cursor()
        .map_err(|e| DashboardError::Render(e.to_string()))?;

    if let Err(err) = res {
        eprintln!("Error durante el dashboard: {err}");
    }

    Ok(())
}

/// Event loop principal del dashboard.
///
/// Renderiza la UI, captura eventos de teclado y despacha acciones
/// según la pestaña activa y el modo de entrada actual.
fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> io::Result<()> {
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        // Renderizar frame actual
        terminal.draw(|f| render(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Ctrl+C para forzar salida siempre
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    app.should_quit = true;
                } else if app.input_mode != InputMode::None {
                    // ── Modo de entrada activo ──
                    handle_input_mode(&mut app, key.code);
                } else {
                    // ── Modo normal ──
                    handle_normal_mode(&mut app, key.code);
                }
            }
        }

        // Avanzar timer del pomodoro
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

/// Maneja teclas mientras hay un modal de entrada activo.
fn handle_input_mode(app: &mut App, key_code: KeyCode) {
    match key_code {
        KeyCode::Enter => match app.input_mode {
            InputMode::TaskAdd => app.add_task(),
            InputMode::ProjectName => app.execute_scaffold(),
            InputMode::SnippetTitle => {
                app.new_snippet_draft.0 = app.input_buffer.clone();
                app.enter_input_mode(InputMode::SnippetLanguage);
            }
            InputMode::SnippetLanguage => {
                app.new_snippet_draft.1 = app.input_buffer.clone();
                app.enter_input_mode(InputMode::SnippetDescription);
            }
            InputMode::SnippetDescription => {
                app.new_snippet_draft.2 = app.input_buffer.clone();
                app.save_snippet_from_clipboard();
            }
            InputMode::SkillSearch => app.search_skills(),
            _ => app.exit_input_mode(),
        },
        KeyCode::Char(c) => app.input_buffer.push(c),
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Esc => app.exit_input_mode(),
        _ => {}
    }
}

/// Maneja teclas en modo normal (sin modal activo).
fn handle_normal_mode(app: &mut App, key_code: KeyCode) {
    match key_code {
        // ── Globales ──
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Right => app.next_tab(),
        KeyCode::Left => app.prev_tab(),

        // ── Navegación vertical por tab ──
        KeyCode::Down | KeyCode::Char('j') => handle_nav_down(app),
        KeyCode::Up | KeyCode::Char('k') => handle_nav_up(app),

        // ── Acciones en Tasks Tab ──
        KeyCode::Char('a') if app.active_tab == Tab::Tasks => {
            app.enter_input_mode(InputMode::TaskAdd);
        }
        KeyCode::Char('d') if app.active_tab == Tab::Tasks => app.delete_task(),
        KeyCode::Char('x') if app.active_tab == Tab::Tasks => app.toggle_task(),

        // ── Acciones en Pomodoro Tab ──
        KeyCode::Char('s') if app.active_tab == Tab::Pomodoro => {
            app.pomodoro_running = !app.pomodoro_running;
        }
        KeyCode::Char('r') if app.active_tab == Tab::Pomodoro => {
            app.pomodoro_running = false;
            app.pomodoro = PomodoroState::Work;
            app.pomodoro_timer = Duration::from_secs(25 * 60);
        }

        // ── Acciones en Scaffold Tab ──
        KeyCode::Enter if app.active_tab == Tab::Scaffold => {
            app.enter_input_mode(InputMode::ProjectName);
        }

        // ── Acciones en Snippets Tab ──
        KeyCode::Char('a') if app.active_tab == Tab::Snippets => {
            app.enter_input_mode(InputMode::SnippetTitle);
        }
        KeyCode::Char('d') if app.active_tab == Tab::Snippets => app.delete_snippet(),
        KeyCode::Char('c') if app.active_tab == Tab::Snippets => app.copy_snippet(),

        // ── Acciones en Git Tab ──
        KeyCode::Char('c') if app.active_tab == Tab::Git => app.clean_git_branches(),

        // ── Acciones en Skills Tab ──
        KeyCode::Char('s') if app.active_tab == Tab::Skills => {
            // Abrir modal de búsqueda de skills
            app.enter_input_mode(InputMode::SkillSearch);
        }
        KeyCode::Char('l') if app.active_tab == Tab::Skills => {
            // Cargar leaderboard completo (búsqueda vacía)
            app.input_buffer.clear();
            app.search_skills();
        }
        KeyCode::Enter if app.active_tab == Tab::Skills => {
            // Instalar la skill seleccionada de forma interactiva
            if let Some(skill) = app.skills_results.get(app.selected_skill_index) {
                let install_path = skill.install_path.clone();

                // 1. Salir temporalmente del modo TUI
                let _ = disable_raw_mode();
                let _ = execute!(io::stdout(), LeaveAlternateScreen);

                // 2. Ejecutar instalación interactiva
                let _ = crate::commands::skills::install_skill_interactive(&install_path);

                // 3. Restaurar el modo TUI
                let _ = enable_raw_mode();
                let _ = execute!(io::stdout(), EnterAlternateScreen);
                
                // Forzar un refresco completo de la pantalla
                // terminal.clear() se usará fuera de aquí si es necesario, 
                // pero Ratatui redibujará en el siguiente loop.
            }
        }

        _ => {}
    }
}

/// Maneja la navegación hacia abajo (↓/j) según la pestaña activa.
fn handle_nav_down(app: &mut App) {
    match app.active_tab {
        Tab::Tasks => {
            let len = app.tasks.len();
            if len > 0 {
                app.selected_task_index = (app.selected_task_index + 1).min(len - 1);
            }
        }
        Tab::Scaffold => {
            let len = crate::commands::init::AVAILABLE_TEMPLATES.len();
            if len > 0 {
                app.selected_template_index = (app.selected_template_index + 1).min(len - 1);
            }
        }
        Tab::Snippets => {
            let len = app.snippets.len();
            if len > 0 {
                app.selected_snippet_index = (app.selected_snippet_index + 1).min(len - 1);
            }
        }
        Tab::Skills => {
            let len = app.skills_results.len();
            if len > 0 {
                app.selected_skill_index = (app.selected_skill_index + 1).min(len - 1);
            }
        }
        _ => {}
    }
}

/// Maneja la navegación hacia arriba (↑/k) según la pestaña activa.
fn handle_nav_up(app: &mut App) {
    match app.active_tab {
        Tab::Tasks => {
            if app.selected_task_index > 0 {
                app.selected_task_index -= 1;
            }
        }
        Tab::Scaffold => {
            if app.selected_template_index > 0 {
                app.selected_template_index -= 1;
            }
        }
        Tab::Snippets => {
            if app.selected_snippet_index > 0 {
                app.selected_snippet_index -= 1;
            }
        }
        Tab::Skills => {
            if app.selected_skill_index > 0 {
                app.selected_skill_index -= 1;
            }
        }
        _ => {}
    }
}
