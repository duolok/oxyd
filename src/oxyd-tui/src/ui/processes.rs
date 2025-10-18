use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use oxyd_domain::models::{SystemMetrics, ProcessState};
use crate::app::AppState;
use super::widgets::format_bytes;

pub fn render(f: &mut Frame, area: Rect, metrics: &SystemMetrics, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),   // Summary
            Constraint::Min(10),     // Process table
            Constraint::Length(4),   // Help + Status
        ])
        .split(area);

    render_process_summary(f, chunks[0], metrics, app);
    render_help_and_status(f, chunks[2], app);
}

fn render_process_summary(f: &mut Frame, area: Rect, metrics: &SystemMetrics, app: &AppState) {
    let sort_indicator = if app.sort_ascending { "▲" } else { "▼" };

    let mut headers = vec![
        "PID".to_string(),
        "Name".to_string(),
        "State".to_string(),
        "CPU%".to_string(),
        "Memory".to_string(),
        "User".to_string(),
    ];

    if let Some(col) = headers.get_mut(app.sort_column) {
        *col = format!("{} {}", col, sort_indicator);
    }

    let header = Row::new(
        headers.iter().map(|h| {
            Cell::from(h.clone()).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        })
    );

    let table_height = area.height.saturating_sub(3) as usize; 
    let visible_start = app.scroll_offset;
    let visible_end = visible_start + table_height;

    let rows: Vec<Row> = app.process_list
        .iter()
        .skip(visible_start)
        .take(table_height)
        .enumerate().map(|(i, process)| {
            let actual_index = visible_start + i;
            let is_selected = app.selected_process == Some(actual_index);
            
            let state_str = match process.state {
                ProcessState::Running => "R",
                ProcessState::Sleeping => "S",
                ProcessState::Waiting => "D",
                ProcessState::Zombie => "Z",
                ProcessState::Stopped => "T",
                ProcessState::Idle => "I",
                ProcessState::Dead => "X",
                ProcessState::Unknown => "?",
            };

            let state_color = match process.state {
                ProcessState::Running => Color::Green,
                ProcessState::Sleeping => Color::Yellow,
                ProcessState::Zombie => Color::Red,
                ProcessState::Stopped => Color::Magenta,
                _ => Color::White,
            };

            let style = if is_selected {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(format!("{}", process.pid)).style(style),
                Cell::from(process.name.clone()).style(style),
                Cell::from(state_str).style(style.fg(state_color)),
                Cell::from(format!("{:.1}", process.cpu_usage_percent)).style(style),
                Cell::from(format_bytes(process.memory_usage_bytes)).style(style),
                Cell::from(process.user.clone()).style(style),
            ])
        })
        .collect();

    let rows = if rows.is_empty() {
        vec![Row::new(vec![
            Cell::from("No processes loaded. Press 'r' to refresh.").style(Style::default().fg(Color::DarkGray))
        ])]
    } else {
        rows
    };

    let table = Table::new(rows, vec![
        Constraint::Length(8),   // PID
        Constraint::Min(20),     // Name
        Constraint::Length(6),   // State
        Constraint::Length(8),   // CPU
        Constraint::Length(12),  // Memory
        Constraint::Length(12),  // User
    ])
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Process List ({}) ", 
                app.process_list.len(),
            ))
            .style(Style::default().fg(Color::White)),
    )
    .highlight_symbol("▶ ");

    f.render_widget(table, area);
}

fn render_help_and_status(f: &mut Frame, area: Rect, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(area);

    let help = Paragraph::new(" ↑/↓: Navigate | r: Refresh | K: Kill | s: Suspend | c: Continue | t: Terminate | p/n/C/m: Sort ")
        .block(Block::default().borders(Borders::ALL).title(" Controls "))
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[0]);

    if let Some(ref msg) = app.status_message {
        let style = if msg.starts_with("ERROR") {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        
        let status = Paragraph::new(msg.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Status "))
            .style(style);
        f.render_widget(status, chunks[1]);
    }
}

