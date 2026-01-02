use crate::app::{App, InputMode};
use crate::ssh::ControlMasterStatus;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
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
                    match app.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char(c) => app.on_key(c),
                            KeyCode::Esc => app.should_quit = true,
                            KeyCode::Up => app.select_prev(),
                            KeyCode::Down => app.select_next(),
                            _ => {}
                        },
                        InputMode::AddingForward => match key.code {
                            KeyCode::Char(c) => app.on_input_key(c),
                            KeyCode::Backspace => app.on_input_backspace(),
                            KeyCode::Enter => app.submit_input(),
                            KeyCode::Esc => app.cancel_input(),
                            _ => {}
                        },
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

    // Footer with help or status message
    let footer_content = if let Some(ref msg) = app.status_message {
        Line::from(vec![
            Span::raw(" "),
            Span::styled(msg, Style::default().fg(Color::Yellow)),
        ])
    } else {
        Line::from(" j/k: Navigate | a: Add | d: Delete | q: Quit")
    };

    let help = Paragraph::new(footer_content).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(help, chunks[2]);

    // Draw input popup if in input mode
    if app.input_mode == InputMode::AddingForward {
        draw_input_popup(frame, app);
    }
}

fn draw_input_popup(frame: &mut ratatui::Frame, app: &App) {
    let area = frame.area();

    // Center a popup
    let popup_area = centered_rect(40, 5, area);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    // Draw the input box
    let input = Paragraph::new(Line::from(vec![
        Span::raw(&app.input_buffer),
        Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Add Port Forward")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(input, popup_area);
}

/// Create a centered rectangle with given width and height
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let horizontal = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .split(area);

    Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .split(horizontal[0])[0]
}
