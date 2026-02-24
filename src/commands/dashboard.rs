//! Dashboard TUI interactivo maestro que absorbió todas las demás CLIs.
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

pub fn execute() -> Result<(), DashboardError> {
    enable_raw_mode().map_err(DashboardError::Terminal)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(DashboardError::Terminal)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| DashboardError::Render(e.to_string()))?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

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

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> io::Result<()> {
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
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
                    match key.code {
                        KeyCode::Enter => match app.input_mode {
                            InputMode::TaskAdd => app.add_task(),
                            InputMode::ProjectName => {
                                app.execute_scaffold();
                            }
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
                            _ => app.exit_input_mode(),
                        },
                        KeyCode::Char(c) => app.input_buffer.push(c),
                        KeyCode::Backspace => {
                            app.input_buffer.pop();
                        }
                        KeyCode::Esc => app.exit_input_mode(),
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Right => app.next_tab(),
                        KeyCode::Left => app.prev_tab(),

                        // Generales por Tab
                        KeyCode::Down | KeyCode::Char('j') => match app.active_tab {
                            Tab::Tasks => {
                                let len = app.tasks.len();
                                if len > 0 {
                                    app.selected_task_index =
                                        (app.selected_task_index + 1).min(len - 1);
                                }
                            }
                            Tab::Scaffold => {
                                let len = crate::commands::init::AVAILABLE_TEMPLATES.len();
                                if len > 0 {
                                    app.selected_template_index =
                                        (app.selected_template_index + 1).min(len - 1);
                                }
                            }
                            Tab::Snippets => {
                                let len = app.snippets.len();
                                if len > 0 {
                                    app.selected_snippet_index =
                                        (app.selected_snippet_index + 1).min(len - 1);
                                }
                            }
                            _ => {}
                        },
                        KeyCode::Up | KeyCode::Char('k') => match app.active_tab {
                            Tab::Tasks => {
                                if app.selected_task_index > 0 {
                                    app.selected_task_index -= 1
                                }
                            }
                            Tab::Scaffold => {
                                if app.selected_template_index > 0 {
                                    app.selected_template_index -= 1
                                }
                            }
                            Tab::Snippets => {
                                if app.selected_snippet_index > 0 {
                                    app.selected_snippet_index -= 1
                                }
                            }
                            _ => {}
                        },

                        // Acciones en Tasks Tab
                        KeyCode::Char('a') if app.active_tab == Tab::Tasks => {
                            app.enter_input_mode(InputMode::TaskAdd);
                        }
                        KeyCode::Char('d') if app.active_tab == Tab::Tasks => {
                            app.delete_task();
                        }
                        KeyCode::Char('x') if app.active_tab == Tab::Tasks => {
                            app.toggle_task();
                        }

                        // Acciones en Pomodoro Tab
                        KeyCode::Char('s') if app.active_tab == Tab::Pomodoro => {
                            app.pomodoro_running = !app.pomodoro_running;
                        }
                        KeyCode::Char('r') if app.active_tab == Tab::Pomodoro => {
                            app.pomodoro_running = false;
                            app.pomodoro = PomodoroState::Work;
                            app.pomodoro_timer = Duration::from_secs(25 * 60);
                        }

                        // Acciones en Scaffold Tab
                        KeyCode::Enter if app.active_tab == Tab::Scaffold => {
                            app.enter_input_mode(InputMode::ProjectName);
                        }

                        // Acciones en Snippets Tab
                        KeyCode::Char('a') if app.active_tab == Tab::Snippets => {
                            app.enter_input_mode(InputMode::SnippetTitle);
                        }
                        KeyCode::Char('d') if app.active_tab == Tab::Snippets => {
                            app.delete_snippet();
                        }
                        KeyCode::Char('c') if app.active_tab == Tab::Snippets => {
                            app.copy_snippet();
                        }

                        // Acciones en Git
                        KeyCode::Char('c') if app.active_tab == Tab::Git => {
                            app.clean_git_branches();
                        }

                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
