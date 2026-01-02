use crate::app::App;
use crate::ssh::ControlMasterStatus;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::io::{self, stdout};

/// Run the TUI application
pub fn run(app: &mut App) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop
    let result = run_loop(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    result
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|frame| {
            draw(frame, app);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events (not release)
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(c) => app.on_key(c),
                        KeyCode::Esc => app.should_quit = true,
                        KeyCode::Up => app.select_prev(),
                        KeyCode::Down => app.select_next(),
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn draw(frame: &mut ratatui::Frame, app: &App) {
    let area = frame.area();

    // Create layout with header, main content, and footer
    let chunks = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Min(0),    // Main content
        Constraint::Length(1), // Footer
    ])
    .split(area);

    // Header
    let status_text = match &app.master_status {
        ControlMasterStatus::Running { pid } => {
            Span::styled(format!("Running (PID: {})", pid), Style::default().fg(Color::Green))
        }
        ControlMasterStatus::NotRunning => {
            Span::styled("Not Running", Style::default().fg(Color::Red))
        }
    };

    let header = Paragraph::new(Line::from(vec![
        Span::raw("Host: "),
        Span::styled(&app.hostname, Style::default().fg(Color::Cyan)),
        Span::raw(" | Control Master: "),
        status_text,
    ]))
    .block(Block::default().borders(Borders::ALL).title("MuxWarden"));

    frame.render_widget(header, chunks[0]);

    // Main content - list of port forwards
    let items: Vec<ListItem> = app
        .forwards
        .iter()
        .map(|fwd| {
            ListItem::new(format!("  localhost:{}  ->  localhost:{}", fwd.local_port, fwd.local_port))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Port Forwards ({})", app.forwards.len())),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    // Create list state for selection
    let mut list_state = ListState::default();
    if !app.forwards.is_empty() {
        list_state.select(Some(app.selected));
    }

    frame.render_stateful_widget(list, chunks[1], &mut list_state);

    // Footer with help
    let help = Paragraph::new(" j/k or ↑/↓: Navigate | q: Quit")
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(help, chunks[2]);
}
