use crate::types::ProxyState;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs},
    Frame, Terminal,
};
use std::io;
use std::sync::Arc;
use std::time::Duration;

enum Tab {
    Overview,
    Requests,
    Help,
}

struct App {
    state: Arc<ProxyState>,
    current_tab: Tab,
    should_quit: bool,
}

impl App {
    fn new(state: Arc<ProxyState>) -> Self {
        Self {
            state,
            current_tab: Tab::Overview,
            should_quit: false,
        }
    }

    fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Overview => Tab::Requests,
            Tab::Requests => Tab::Help,
            Tab::Help => Tab::Overview,
        };
    }

    fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Help => Tab::Requests,
            Tab::Requests => Tab::Overview,
            Tab::Overview => Tab::Help,
        };
    }
}

pub async fn run_tui(state: Arc<ProxyState>) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(state);

    // Run app
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                    }
                    KeyCode::Tab => {
                        app.next_tab();
                    }
                    KeyCode::BackTab => {
                        app.previous_tab();
                    }
                    KeyCode::Char('1') => {
                        app.current_tab = Tab::Overview;
                    }
                    KeyCode::Char('2') => {
                        app.current_tab = Tab::Requests;
                    }
                    KeyCode::Char('3') => {
                        app.current_tab = Tab::Help;
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Header
    render_header(f, chunks[0]);

    // Tabs
    let tab_titles = vec!["Overview (1)", "Requests (2)", "Help (3)"];
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(match app.current_tab {
            Tab::Overview => 0,
            Tab::Requests => 1,
            Tab::Help => 2,
        })
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, chunks[1]);

    // Content area
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0)])
        .split(chunks[1]);

    match app.current_tab {
        Tab::Overview => render_overview(f, app, content_chunks[0]),
        Tab::Requests => render_requests(f, app, content_chunks[0]),
        Tab::Help => render_help(f, content_chunks[0]),
    }

    // Footer
    render_footer(f, chunks[2]);
}

fn render_header(f: &mut Frame, area: Rect) {
    let title = Paragraph::new("Charles - Reverse Proxy with TUI")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, area);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let footer = Paragraph::new(
        "Press 'q' to quit | Tab/Shift+Tab to navigate | 1/2/3 for direct tab access",
    )
    .style(Style::default().fg(Color::Gray))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, area);
}

fn render_overview(f: &mut Frame, app: &App, area: Rect) {
    let stats = app.state.get_stats();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(0)])
        .split(area);

    // Stats
    let stats_text = vec![
        Line::from(vec![
            Span::styled("Total Requests: ", Style::default().fg(Color::Yellow)),
            Span::raw(stats.total_requests.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Successful: ", Style::default().fg(Color::Green)),
            Span::raw(stats.successful_requests.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Failed: ", Style::default().fg(Color::Red)),
            Span::raw(stats.failed_requests.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Active Connections: ", Style::default().fg(Color::Cyan)),
            Span::raw(stats.active_connections.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Bytes Sent: ", Style::default().fg(Color::Magenta)),
            Span::raw(format_bytes(stats.total_bytes_sent)),
        ]),
        Line::from(vec![
            Span::styled("Bytes Received: ", Style::default().fg(Color::Magenta)),
            Span::raw(format_bytes(stats.total_bytes_received)),
        ]),
    ];

    let stats_widget = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title("Statistics"))
        .style(Style::default().fg(Color::White));

    f.render_widget(stats_widget, chunks[0]);

    // Recent activity
    let transactions = app.state.get_recent_transactions(10);
    let rows: Vec<Row> = transactions
        .iter()
        .map(|t| {
            let status_style = if t.status >= 200 && t.status < 300 {
                Style::default().fg(Color::Green)
            } else if t.status >= 400 {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Yellow)
            };

            Row::new(vec![
                Cell::from(t.timestamp.format("%H:%M:%S").to_string()),
                Cell::from(t.method.clone()),
                Cell::from(t.path.clone()),
                Cell::from(t.status.to_string()).style(status_style),
                Cell::from(format!("{}ms", t.duration_ms)),
            ])
        })
        .collect();

    let table = Table::new(rows)
        .widths(&[
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Min(30),
            Constraint::Length(8),
            Constraint::Length(10),
        ])
        .header(
            Row::new(vec!["Time", "Method", "Path", "Status", "Duration"])
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Recent Activity"),
        );

    f.render_widget(table, chunks[1]);
}

fn render_requests(f: &mut Frame, app: &App, area: Rect) {
    let transactions = app.state.get_recent_transactions(100);

    let rows: Vec<Row> = transactions
        .iter()
        .map(|t| {
            let status_style = if t.status >= 200 && t.status < 300 {
                Style::default().fg(Color::Green)
            } else if t.status >= 400 {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Yellow)
            };

            Row::new(vec![
                Cell::from(t.id.to_string()),
                Cell::from(t.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()),
                Cell::from(t.method.clone()),
                Cell::from(t.path.clone()),
                Cell::from(t.status.to_string()).style(status_style),
                Cell::from(format!("{}ms", t.duration_ms)),
            ])
        })
        .collect();

    let table = Table::new(rows)
        .widths(&[
            Constraint::Length(6),
            Constraint::Length(20),
            Constraint::Length(8),
            Constraint::Min(20),
            Constraint::Length(8),
            Constraint::Length(10),
        ])
        .header(
            Row::new(vec![
                "ID",
                "Timestamp",
                "Method",
                "Path",
                "Status",
                "Duration",
            ])
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Request History"),
        );

    f.render_widget(table, area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Charles - Reverse Proxy with TUI",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("Keyboard Shortcuts:"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  q          ", Style::default().fg(Color::Yellow)),
            Span::raw("- Quit the application"),
        ]),
        Line::from(vec![
            Span::styled("  Tab        ", Style::default().fg(Color::Yellow)),
            Span::raw("- Next tab"),
        ]),
        Line::from(vec![
            Span::styled("  Shift+Tab  ", Style::default().fg(Color::Yellow)),
            Span::raw("- Previous tab"),
        ]),
        Line::from(vec![
            Span::styled("  1          ", Style::default().fg(Color::Yellow)),
            Span::raw("- Overview tab"),
        ]),
        Line::from(vec![
            Span::styled("  2          ", Style::default().fg(Color::Yellow)),
            Span::raw("- Requests tab"),
        ]),
        Line::from(vec![
            Span::styled("  3          ", Style::default().fg(Color::Yellow)),
            Span::raw("- Help tab"),
        ]),
        Line::from(""),
        Line::from("Features:"),
        Line::from(""),
        Line::from("  • Real-time request monitoring"),
        Line::from("  • Statistics dashboard"),
        Line::from("  • Request/Response logging"),
        Line::from("  • Windows MSVC compatible"),
        Line::from(""),
        Line::from("Command Line Usage:"),
        Line::from(""),
        Line::from("  charles --port 8080 --target localhost:3000"),
        Line::from("  charles --no-tui  # Run without TUI"),
        Line::from(""),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .style(Style::default().fg(Color::White));

    f.render_widget(help, area);
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_idx])
}
