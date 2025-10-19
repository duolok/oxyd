use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

const OXYD_ART: &str = r#"
   ██████╗ ██╗  ██╗██╗   ██╗██████╗ 
  ██╔═══██╗╚██╗██╔╝╚██╗ ██╔╝██╔══██╗
  ██║   ██║ ╚███╔╝  ╚████╔╝ ██║  ██║
  ██║   ██║ ██╔██╗   ╚██╔╝  ██║  ██║
  ╚██████╔╝██╔╝ ██╗   ██║   ██████╔╝
   ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ╚═════╝ 
"#;

pub fn render_help(f: &mut Frame) {
    let area = centered_rect(80, 85, f.area());

    // Clear the background
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // ASCII art
            Constraint::Min(10),   // Controls
            Constraint::Length(3), // Footer
        ])
        .split(area);

    render_ascii_art(f, chunks[0]);
    render_controls(f, chunks[1]);
    render_help_footer(f, chunks[2]);
}

fn render_ascii_art(f: &mut Frame, area: Rect) {
    let art_lines: Vec<Line> = OXYD_ART
        .lines()
        .map(|line| {
            Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let paragraph = Paragraph::new(art_lines)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_controls(f: &mut Frame, area: Rect) {
    let controls = vec![
        (
            "NAVIGATION",
            vec![
                ("Tab / Shift+Tab", "Switch between tabs"),
                ("1-7", "Jump to specific tab"),
                ("↑ / k", "Move up"),
                ("↓ / j", "Move down"),
                ("PgUp / PgDn", "Page up / down"),
                ("Home / End", "Jump to first / last"),
            ],
        ),
        (
            "PROCESS MANAGEMENT",
            vec![
                ("K (Shift+k)", "Kill selected process (SIGKILL)"),
                ("t", "Terminate selected process (SIGTERM)"),
                ("s", "Suspend selected process (SIGSTOP)"),
                ("c", "Continue selected process (SIGCONT)"),
                ("r", "Refresh process list"),
            ],
        ),
        (
            "SORTING",
            vec![
                ("p", "Sort by PID"),
                ("n", "Sort by Name"),
                ("C (Shift+c)", "Sort by CPU usage"),
                ("m", "Sort by Memory usage"),
            ],
        ),
        (
            "OTHER",
            vec![
                ("?", "Toggle this help screen"),
                ("f", "Open filter/search"),
                ("z", "Open alerts configuration"),
                ("n", "Open notifications"),
                ("q / Esc", "Quit application"),
            ],
        ),
    ];

    let mut lines = vec![Line::from("")];

    for (section, items) in controls {
        lines.push(Line::from(vec![Span::styled(
            format!("  {}  ", section),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]));
        lines.push(Line::from(""));

        for (key, description) in items {
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(
                    format!("{:20}", key),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(description, Style::default().fg(Color::White)),
            ]));
        }
        lines.push(Line::from(""));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Keyboard Shortcuts ")
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(Color::Black));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn render_help_footer(f: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Linux System Monitor", Style::default().fg(Color::Cyan)),
        Span::raw(" | "),
        Span::styled("Press ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "?",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" or ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to close", Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black)),
    );

    f.render_widget(footer, area);
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
