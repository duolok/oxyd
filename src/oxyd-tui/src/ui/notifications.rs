use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::AppState;
use crate::notifications::{NotificationLevel};

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),   
            Constraint::Min(10),     
            Constraint::Length(3),   
        ])
        .split(area);

    render_header(f, chunks[0], app);
    render_notifications_table(f, chunks[1], app);
    render_footer(f, chunks[2]);
}

fn render_header(f: &mut Frame, area: Rect, app: &AppState) {
    let notification_mgr = &app.notification_manager;
    
    let lines = vec![
        Line::from(vec![
            Span::styled("Total Notifications: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}", notification_mgr.total_count()), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Unread: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}", notification_mgr.unread_count()), Style::default().fg(Color::Yellow)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Notification Center ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .style(Style::default().fg(Color::White));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn render_notifications_table(f: &mut Frame, area: Rect, app: &AppState) {
    let notification_mgr = &app.notification_manager;
    let notifications = notification_mgr.get_all();

    let header = Row::new(vec![
        Cell::from("Time").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Level").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Title").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Message").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ])
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = notifications
        .iter()
        .rev() // Most recent first
        .map(|notif| {
            let time_str = notif.timestamp.format("%H:%M:%S").to_string();
            
            let (level_str, level_color) = match notif.level {
                NotificationLevel::Info => ("INFO", Color::Blue),
                NotificationLevel::Warning => ("WARN", Color::Yellow),
                NotificationLevel::Critical => ("CRIT", Color::Red),
                NotificationLevel::Success => ("OK", Color::Green),
            };

            let status_str = if notif.read { "Read" } else { "New" };
            let status_color = if notif.read { Color::DarkGray } else { Color::Yellow };

            let base_style = if !notif.read {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            Row::new(vec![
                Cell::from(time_str).style(base_style),
                Cell::from(level_str).style(base_style.fg(level_color)),
                Cell::from(truncate_string(&notif.title, 25)).style(base_style),
                Cell::from(truncate_string(&notif.message, 60)).style(base_style),
                Cell::from(status_str).style(base_style.fg(status_color)),
            ])
            .height(1)
        })
        .collect();

    let rows = if rows.is_empty() {
        vec![Row::new(vec![
            Cell::from("No notifications yet")
                .style(Style::default().fg(Color::DarkGray))
        ])]
    } else {
        rows
    };

    let table = Table::new(
        rows,
        vec![
            Constraint::Length(10),  // Time
            Constraint::Length(6),   // Level
            Constraint::Length(25),  // Title
            Constraint::Min(60),     // Message
            Constraint::Length(8),   // Status
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Recent Notifications ")
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .style(Style::default().fg(Color::White)),
    );

    f.render_widget(table, area);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Press ", Style::default().fg(Color::DarkGray)),
        Span::styled("m", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled(" to mark all as read | ", Style::default().fg(Color::DarkGray)),
        Span::styled("x", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled(" to clear all", Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
    );

    f.render_widget(footer, area);
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
