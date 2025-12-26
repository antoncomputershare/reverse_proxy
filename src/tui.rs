use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
};
use serde::Deserialize;
use std::io;
use std::time::Duration;
use tracing::{error, info};

#[derive(Debug, Deserialize)]
struct Metrics {
    total_requests: u64,
    active_requests: u64,
    total_errors: u64,
}

enum Tab {
    Stats,
    Requests,
}

pub struct TuiApp {
    control_url: String,
    selected_tab: Tab,
    requests: Vec<String>,
    list_state: ListState,
    should_quit: bool,
}

impl TuiApp {
    pub fn new(control_url: String) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            control_url,
            selected_tab: Tab::Stats,
            requests: Vec::new(),
            list_state,
            should_quit: false,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal).await;

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn run_app<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        loop {
            // Fetch data from control API
            if let Err(e) = self.fetch_data().await {
                error!("Failed to fetch data: {}", e);
            }

            terminal.draw(|f| self.ui(f))?;

            // Handle events with timeout
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                self.should_quit = true;
                            }
                            KeyCode::Char('c')
                                if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                            {
                                self.should_quit = true;
                            }
                            KeyCode::Char('1') => {
                                self.selected_tab = Tab::Stats;
                            }
                            KeyCode::Char('2') => {
                                self.selected_tab = Tab::Requests;
                            }
                            KeyCode::Down => {
                                self.next_request();
                            }
                            KeyCode::Up => {
                                self.previous_request();
                            }
                            KeyCode::Char('r') => {
                                self.replay_request();
                            }
                            _ => {}
                        }
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(f.area());

        // Tab bar
        let tab_titles = vec!["1. Stats", "2. Requests"];
        let tabs = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Charles Proxy"),
            )
            .select(match self.selected_tab {
                Tab::Stats => 0,
                Tab::Requests => 1,
            })
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(tabs, chunks[0]);

        // Content area
        match self.selected_tab {
            Tab::Stats => self.render_stats(f, chunks[1]),
            Tab::Requests => self.render_requests(f, chunks[1]),
        }
    }

    fn render_stats(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let text = vec![
            Line::from(vec![
                Span::styled(
                    "Control API: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(&self.control_url),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled("Connected", Style::default().fg(Color::Green)),
            ]),
            Line::from(""),
            Line::from("Use ↑/↓ to navigate, 'r' to replay, 'q'/Esc/Ctrl+C to quit"),
        ];

        let paragraph =
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Statistics"));

        f.render_widget(paragraph, area);
    }

    fn render_requests(&mut self, f: &mut Frame, area: ratatui::layout::Rect) {
        let items: Vec<ListItem> = self
            .requests
            .iter()
            .map(|r| ListItem::new(r.as_str()))
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Requests"))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn next_request(&mut self) {
        if self.requests.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.requests.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous_request(&mut self) {
        if self.requests.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.requests.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn replay_request(&self) {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.requests.len() {
                info!("Replaying request: {}", self.requests[selected]);
            }
        }
    }

    async fn fetch_data(&mut self) -> Result<()> {
        // Fetch metrics
        let metrics_url = format!("{}/metrics", self.control_url);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()?;

        match client.get(&metrics_url).send().await {
            Ok(response) => {
                if let Ok(metrics) = response.json::<Metrics>().await {
                    // Update requests list with dummy data for now
                    self.requests.clear();
                    self.requests
                        .push(format!("Total Requests: {}", metrics.total_requests));
                    self.requests
                        .push(format!("Active Requests: {}", metrics.active_requests));
                    self.requests
                        .push(format!("Total Errors: {}", metrics.total_errors));
                }
            }
            Err(_) => {
                self.requests.clear();
                self.requests
                    .push("Failed to connect to control API".to_string());
            }
        }

        Ok(())
    }
}
