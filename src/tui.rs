use std::io;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

use crate::check::{check_all, DocState};
use crate::index::{doc_id, load_index, project_root};

enum Focus {
    Valid,
    Stale,
}

struct App {
    valid: Vec<DocState>,
    stale: Vec<DocState>,
    focus: Focus,
    v_idx: usize,
    s_idx: usize,
}

impl App {
    fn new() -> Result<Self> {
        let root = project_root()?;
        let (valid, stale) = check_all(&root, None)?;
        Ok(App {
            valid,
            stale,
            focus: Focus::Valid,
            v_idx: 0,
            s_idx: 0,
        })
    }

    fn refresh(&mut self) {
        if let Ok(root) = project_root() {
            if let Ok((valid, stale)) = check_all(&root, None) {
                self.valid = valid;
                self.stale = stale;
            }
        }
        self.clamp();
    }

    fn clamp(&mut self) {
        let vmax = self.valid.len().saturating_sub(1);
        let smax = self.stale.len().saturating_sub(1);
        if self.v_idx > vmax {
            self.v_idx = vmax;
        }
        if self.s_idx > smax {
            self.s_idx = smax;
        }
    }

    fn selected(&self) -> Option<&DocState> {
        match self.focus {
            Focus::Valid => self.valid.get(self.v_idx),
            Focus::Stale => self.stale.get(self.s_idx),
        }
    }

    fn doc_path(&self, name: &str) -> Option<PathBuf> {
        let root = project_root().ok()?;
        let idx = load_index(&root).ok()?;
        let id = doc_id(name);
        idx.docs.get(&id).map(|entry| root.join(&entry.file))
    }
}

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let app = App::new().context("no .idocs found (run 'idocs init' first)")?;
    let result = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| draw(f, &app))?;

        let event = match event::read() {
            Ok(e) => e,
            Err(_) => break,
        };

        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('r') => app.refresh(),
                KeyCode::Tab => {
                    app.focus = match app.focus {
                        Focus::Valid => Focus::Stale,
                        Focus::Stale => Focus::Valid,
                    };
                    app.clamp();
                }
                KeyCode::Up | KeyCode::Char('k') => match app.focus {
                    Focus::Valid => {
                        if app.v_idx > 0 {
                            app.v_idx -= 1;
                        }
                    }
                    Focus::Stale => {
                        if app.s_idx > 0 {
                            app.s_idx -= 1;
                        }
                    }
                },
                KeyCode::Down | KeyCode::Char('j') => match app.focus {
                    Focus::Valid => {
                        if app.v_idx + 1 < app.valid.len() {
                            app.v_idx += 1;
                        }
                    }
                    Focus::Stale => {
                        if app.s_idx + 1 < app.stale.len() {
                            app.s_idx += 1;
                        }
                    }
                },
                KeyCode::Enter => open_in_editor(terminal, &mut app),
                _ => {}
            }
        }
    }

    Ok(())
}

fn open_in_editor(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) {
    let doc = match app.selected() {
        Some(d) => d,
        None => return,
    };
    let path = match app.doc_path(&doc.name) {
        Some(p) => p,
        None => return,
    };

    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);

    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".into());
    let parts: Vec<&str> = editor.split_whitespace().collect();
    let mut cmd = Command::new(parts[0]);
    for part in &parts[1..] {
        cmd.arg(part);
    }
    cmd.arg(&path);

    if let Err(e) = cmd.status() {
        eprintln!("Error launching editor '{}': {}", editor, e);
        eprintln!("Press Enter to continue...");
        let _ = io::stdin().read_line(&mut String::new());
    }

    let _ = enable_raw_mode();
    let _ = execute!(terminal.backend_mut(), EnterAlternateScreen);

    app.refresh();
}

fn draw(f: &mut Frame, app: &App) {
    let area = f.area();

    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main[0]);

    let is_valid = matches!(app.focus, Focus::Valid);
    let is_stale = matches!(app.focus, Focus::Stale);

    draw_list(
        f,
        chunks[0],
        &app.valid,
        app.v_idx,
        " Valid ",
        Color::Green,
        is_valid,
    );
    draw_list(
        f,
        chunks[1],
        &app.stale,
        app.s_idx,
        " Stale ",
        Color::Red,
        is_stale,
    );

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " ↑↓/jk navigate  Tab switch  Enter open in $EDITOR  r refresh  q quit ",
            Style::new().dim(),
        ))),
        main[1],
    );
}

fn draw_list(
    f: &mut Frame,
    area: Rect,
    docs: &[DocState],
    selected: usize,
    title: &str,
    color: Color,
    active: bool,
) {
    let border_style = if active {
        Style::new().fg(color).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(Color::DarkGray)
    };

    let items: Vec<ListItem> = docs
        .iter()
        .map(|doc| {
            if doc.bad.is_empty() {
                ListItem::new(Line::from(vec![
                    Span::styled("  ", Style::new().fg(Color::Green)),
                    Span::raw(&doc.name),
                ]))
            } else {
                let mut lines = vec![Line::from(vec![
                    Span::styled("  ", Style::new().fg(color)),
                    Span::styled(&doc.name, Style::new().fg(color)),
                ])];
                for (file, why) in &doc.bad {
                    lines.push(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(why.clone(), Style::new().fg(Color::Yellow)),
                        Span::raw(" "),
                        Span::styled(file.clone(), Style::new().dim()),
                    ]));
                }
                ListItem::new(lines)
            }
        })
        .collect();

    let symbol = if active { "▸ " } else { "  " };
    let list = List::new(items)
        .block(
            Block::default()
                .title(format!("{} ({})", title, docs.len()))
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::new()
                .bg(if active {
                    Color::DarkGray
                } else {
                    Color::Reset
                })
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(symbol);

    let mut state = ListState::default();
    state.select(Some(selected.min(docs.len().saturating_sub(1))));
    f.render_stateful_widget(list, area, &mut state);
}
