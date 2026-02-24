//! Renderizado de la TUI
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::tui::app::{App, InputMode, PomodoroState, Tab};

pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

    render_tabs(f, app, chunks[0]);

    match app.active_tab {
        Tab::Overview => render_overview(f, app, chunks[1]),
        Tab::Tasks => render_tasks(f, app, chunks[1]),
        Tab::Pomodoro => render_pomodoro(f, app, chunks[1]),
        Tab::Scaffold => render_scaffold(f, app, chunks[1]),
        Tab::Snippets => render_snippets(f, app, chunks[1]),
        Tab::Git => render_git(f, app, chunks[1]),
    }

    render_footer(f, chunks[2]);

    if app.input_mode != InputMode::None {
        render_input_modal(f, app, size);
    }
}

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = vec![
        "General", "Tareas", "Pomodoro", "Scaffold", "Snippets", "Git",
    ]
    .into_iter()
    .map(Line::from)
    .collect();

    let tab_index = match app.active_tab {
        Tab::Overview => 0,
        Tab::Tasks => 1,
        Tab::Pomodoro => 2,
        Tab::Scaffold => 3,
        Tab::Snippets => 4,
        Tab::Git => 5,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" 🦀 DashTUI "))
        .select(tab_index)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" | ");

    f.render_widget(tabs, area);
}

fn render_overview(f: &mut Frame, app: &App, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw(" Proyecto: "),
            Span::styled(
                &app.project_name,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(" Total Tareas: "),
            Span::styled(
                app.tasks.len().to_string(),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw(" Tareas Completadas: "),
            Span::styled(
                app.tasks.iter().filter(|t| t.completed).count().to_string(),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::raw(" Pomodoros Completados: "),
            Span::styled(
                app.pomodoros_completed.to_string(),
                Style::default().fg(Color::LightMagenta),
            ),
        ]),
    ];

    let p = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" Resumen "))
        .alignment(Alignment::Left);

    f.render_widget(p, area);
}

fn render_tasks(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let prefix = if task.completed { "✅ " } else { "☐ " };
            let style = if i == app.selected_task_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if task.completed {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(format!("{prefix}{}", task.title)).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Tareas (a: add, x: toggle, d: del) "),
    );

    f.render_widget(list, area);
}

fn render_pomodoro(f: &mut Frame, app: &App, area: Rect) {
    let secs = app.pomodoro_timer.as_secs();
    let mins = secs / 60;
    let rem_secs = secs % 60;

    let time_str = format!("{mins:02}:{rem_secs:02}");

    let main_color = match app.pomodoro {
        PomodoroState::Work => Color::Red,
        PomodoroState::ShortBreak => Color::Green,
        PomodoroState::LongBreak => Color::Blue,
    };

    let state_str = match app.pomodoro {
        PomodoroState::Work => "Enfoque",
        PomodoroState::ShortBreak => "Descanso Corto",
        PomodoroState::LongBreak => "Descanso Largo",
    };

    let status = if app.pomodoro_running {
        "Corriendo "
    } else {
        "Pausado (s: start/stop) "
    };

    let text = vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            time_str,
            Style::default().fg(main_color).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            state_str,
            Style::default().fg(main_color),
        )]),
    ];

    let p = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(status))
        .alignment(Alignment::Center);

    f.render_widget(p, area);
}

fn render_scaffold(f: &mut Frame, app: &App, area: Rect) {
    let templates = crate::commands::init::AVAILABLE_TEMPLATES;

    let items: Vec<ListItem> = templates
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let style = if i == app.selected_template_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("  {}  ", t)).style(style)
        })
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Templates (Enter: init) "),
    );

    f.render_widget(list, chunks[0]);

    if let Some(ref logs) = app.scaffold_logs {
        let text: Vec<Line> = logs.iter().map(|l| Line::from(l.as_str())).collect();
        let p =
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(" Salida "));
        f.render_widget(p, chunks[1]);
    } else {
        let p = Paragraph::new("Selecciona un template y presiona Enter para generar un proyecto.\nEl proyecto se genarará en el CWD actual.")
            .block(Block::default().borders(Borders::ALL).title(" Instrucciones "))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, chunks[1]);
    }
}

fn render_snippets(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let items: Vec<ListItem> = app
        .snippets
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == app.selected_snippet_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!(" [{}] {}", s.language, s.name)).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Snippets (a: add, d: del, c: copy) "),
    );

    f.render_widget(list, chunks[0]);

    if let Some(s) = app.snippets.get(app.selected_snippet_index) {
        let description = format!("Descripción: {}", s.description);
        let text = vec![
            Line::from(description.as_str()),
            Line::from(""),
            Line::from(Span::styled(&s.code, Style::default().fg(Color::Cyan))),
        ];

        let p = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Código "))
            .wrap(Wrap { trim: false });
        f.render_widget(p, chunks[1]);
    } else {
        let p = Paragraph::new(
            "No hay snippets disponibles. Presiona 'a' para agregar desde el portapapeles.",
        )
        .block(Block::default().borders(Borders::ALL).title(" Código "));
        f.render_widget(p, chunks[1]);
    }
}

fn render_git(f: &mut Frame, app: &App, area: Rect) {
    if let Some(err) = &app.git_error {
        let p = Paragraph::new(format!("⚠️ Error Git: {}", err))
            .block(Block::default().borders(Borders::ALL).title(" Git "));
        f.render_widget(p, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Stats Side
    if let Some(stats) = &app.git_stats {
        let mut lines = vec![
            Line::from(vec![
                Span::raw(" 📝 Commits Totales: "),
                Span::styled(
                    stats.total_commits.to_string(),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::raw(" 🌿 Ramas Locales: "),
                Span::styled(
                    stats.local_branches.to_string(),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::raw(" 📁 Archivos Tracked: "),
                Span::styled(
                    stats.tracked_files.to_string(),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                " 🏆 Top Contribuidores:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ];

        for (i, (name, count)) in stats.contributors.iter().take(5).enumerate() {
            let bar = "█".repeat((*count as usize).min(20));
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {}. ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(format!("{name:<15} "), Style::default().fg(Color::Green)),
                Span::styled(format!("{}", count), Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled(bar, Style::default().fg(Color::Blue)),
            ]));
        }

        if let Some(b) = &app.git_merged_branches {
            lines.push(Line::from(""));
            if b.is_empty() {
                lines.push(Line::from("  ✔ No hay ramas fusionadas para limpiar."));
            } else {
                lines.push(Line::from(Span::styled(
                    "  🧹 Ramas para limpiar (presiona 'c'):",
                    Style::default().fg(Color::Yellow),
                )));
                for branch in b {
                    lines.push(Line::from(vec![Span::raw("    • "), Span::raw(branch)]));
                }
            }
        }

        let p = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Estadísticas y Ramas "),
        );
        f.render_widget(p, chunks[0]);
    } else {
        f.render_widget(
            Paragraph::new("Cargando...").block(Block::default().borders(Borders::ALL)),
            chunks[0],
        );
    }

    // Changelog Side
    if let Some(ch) = &app.git_changelog {
        let mut lines = Vec::new();
        for entry in ch {
            let emoji = crate::commands::git_utils::get_type_emoji(&entry.commit_type);
            let label = crate::commands::git_utils::format_type_label(&entry.commit_type);
            lines.push(Line::from(vec![
                Span::raw(emoji),
                Span::raw(" "),
                Span::styled(label, Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!(" ({})", entry.messages.len()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
            for msg in &entry.messages {
                lines.push(Line::from(vec![
                    Span::styled("  • ", Style::default().fg(Color::DarkGray)),
                    Span::raw(msg),
                ]));
            }
            lines.push(Line::from(""));
        }

        let p = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Changelog "))
            .wrap(Wrap { trim: false });
        f.render_widget(p, chunks[1]);
    }
}

fn render_footer(f: &mut Frame, area: Rect) {
    let mode = "NORMAL";
    let instructions = " q: Salir | ←/→: Tabs | ↑/↓: Navegar ";

    let footer = Line::from(vec![
        Span::styled(
            format!(" {mode} "),
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(instructions),
    ]);

    let block = Block::default()
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .title_alignment(Alignment::Center);

    let p = Paragraph::new(footer).block(block);
    f.render_widget(p, area);
}

fn render_input_modal(f: &mut Frame, app: &App, size: Rect) {
    let title = match app.input_mode {
        InputMode::TaskAdd => " Nueva Tarea ",
        InputMode::ProjectName => " Nombre del Proyecto ",
        InputMode::SnippetTitle => " Título del Snippet (El código se leerá del Clipboard) ",
        InputMode::SnippetLanguage => " Lenguaje (ej: rust, js, py) ",
        InputMode::SnippetDescription => " Descripción del Snippet ",
        _ => " Input ",
    };

    let p = Paragraph::new(app.input_buffer.as_str())
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(Color::Yellow));

    let area = centered_rect(60, 20, size);
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
