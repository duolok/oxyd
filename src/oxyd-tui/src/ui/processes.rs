use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use super::widgets::format_bytes;
use crate::app::AppState;
use oxyd_domain::models::{ProcessState, SystemMetrics};

pub fn render(f: &mut Frame, area: Rect, metrics: &SystemMetrics, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    render_process_stats(f, chunks[0], metrics, app);
    render_process_table(f, chunks[1], app);
    render_status_bar(f, chunks[2], app);
}

fn render_process_stats(f: &mut Frame, area: Rect, metrics: &SystemMetrics, app: &AppState) {
    let procs = &metrics.processes;

    let sort_info = match app.sort_column {
        0 => "PID",
        1 => "Name",
        2 => "CPU%",
        3 => "Memory",
        4 => "State",
        5 => "User",
        _ => "Unknown",
    };

    let sort_direction = if app.sort_ascending { "↑" } else { "↓" };

    let filter_line = if !app.process_filter.is_empty() {
        Line::from(vec![
            Span::styled(
                "Filter: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("'{}' ", app.process_filter),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("(Press 'x' to clear)", Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "/",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " to search/filter processes",
                Style::default().fg(Color::DarkGray),
            ),
        ])
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(
                "Total: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}", procs.total_count),
                Style::default().fg(Color::White),
            ),
            Span::raw("  "),
            Span::styled(
                "Running: ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}", procs.running_count),
                Style::default().fg(Color::Green),
            ),
            Span::raw("  "),
            Span::styled(
                "Sleeping: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}", procs.sleeping_count),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw("  "),
            Span::styled(
                "Stopped: ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}", procs.stopped_count),
                Style::default().fg(Color::Magenta),
            ),
            Span::raw("  "),
            Span::styled(
                "Zombie: ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}", procs.zombie_count),
                Style::default().fg(Color::Red),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Showing: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}", app.filtered_process_list.len()),
                Style::default().fg(Color::White),
            ),
            Span::raw(" processes"),
            Span::raw("    "),
            Span::styled(
                "Sort: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{} {}", sort_info, sort_direction),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(""),
        filter_line,
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Process Summary ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().fg(Color::White));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn render_process_table(f: &mut Frame, area: Rect, app: &AppState) {
    let header_cells = vec![
        "PID", "Name", "State", "CPU%", "Memory", "Mem%", "User", "Threads", "Priority",
    ];

    let sort_indicator = if app.sort_ascending { " ▲" } else { " ▼" };

    let header = Row::new(header_cells.iter().enumerate().map(|(i, h)| {
        let text = if i == app.sort_column {
            format!("{}{}", h, sort_indicator)
        } else {
            h.to_string()
        };
        Cell::from(text).style(
            Style::default()
                .fg(if i == app.sort_column {
                    Color::Yellow
                } else {
                    Color::Cyan
                })
                .add_modifier(Modifier::BOLD),
        )
    }))
    .height(1)
    .bottom_margin(1);

    let table_height = area.height.saturating_sub(4) as usize;
    let visible_start = app.scroll_offset;

    let rows: Vec<Row> = app
        .filtered_process_list
        .iter()
        .enumerate()
        .skip(visible_start)
        .take(table_height)
        .map(|(i, process)| {
            let actual_index = visible_start + i;
            let is_selected = app.selected_process == Some(actual_index);

            let state_str = match process.state {
                ProcessState::Running => "RUN",
                ProcessState::Sleeping => "SLP",
                ProcessState::Waiting => "WAIT",
                ProcessState::Zombie => "ZMB",
                ProcessState::Stopped => "STP",
                ProcessState::Idle => "IDL",
                ProcessState::Dead => "DED",
                ProcessState::Unknown => "???",
            };

            let state_color = match process.state {
                ProcessState::Running => Color::Green,
                ProcessState::Sleeping => Color::Yellow,
                ProcessState::Zombie => Color::Red,
                ProcessState::Stopped => Color::Magenta,
                ProcessState::Dead => Color::DarkGray,
                _ => Color::White,
            };

            let cpu_color = if process.cpu_usage_percent > 80.0 {
                Color::Red
            } else if process.cpu_usage_percent > 50.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            let mem_color = if process.memory_usage_percent > 10.0 {
                Color::Red
            } else if process.memory_usage_percent > 5.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            let base_style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(format!("{}", process.pid)).style(base_style),
                Cell::from(truncate_string(&process.name, 20)).style(base_style),
                Cell::from(state_str).style(base_style.fg(state_color)),
                Cell::from(format!("{:.1}", process.cpu_usage_percent))
                    .style(base_style.fg(cpu_color)),
                Cell::from(format_bytes(process.memory_usage_bytes)).style(base_style),
                Cell::from(format!("{:.1}%", process.memory_usage_percent))
                    .style(base_style.fg(mem_color)),
                Cell::from(truncate_string(&process.user, 12)).style(base_style),
                Cell::from(format!("{}", process.threads)).style(base_style),
                Cell::from(format!("{}", process.priority)).style(base_style),
            ])
            .height(1)
        })
        .collect();

    let rows = if rows.is_empty() {
        vec![Row::new(vec![
            Cell::from("No processes loaded. Press 'r' to refresh.")
                .style(Style::default().fg(Color::DarkGray)),
        ])]
    } else {
        rows
    };

    let selected_style = Style::default()
        .bg(Color::DarkGray)
        .add_modifier(Modifier::BOLD);

    let table = Table::new(
        rows,
        vec![
            Constraint::Length(8),  // pID
            Constraint::Min(20),    // name
            Constraint::Length(6),  // state
            Constraint::Length(8),  // cPU
            Constraint::Length(10), // memory bytes
            Constraint::Length(8),  // memory %
            Constraint::Length(12), // user
            Constraint::Length(8),  // threads
            Constraint::Length(9),  // priority
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Process List ({} total) ", app.process_list.len()))
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().fg(Color::White)),
    )
    .row_highlight_style(selected_style)
    .highlight_symbol("▶ ");

    f.render_widget(table, area);
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &AppState) {
    let status_text = if let Some(ref msg) = app.status_message {
        msg.clone()
    } else if let Some(selected_idx) = app.selected_process {
        if let Some(process) = app.process_list.get(selected_idx) {
            format!(
                "Selected: {} (PID: {}) | CPU: {:.1}% | MEM: {} ({:.1}%) | Threads: {}",
                process.name,
                process.pid,
                process.cpu_usage_percent,
                format_bytes(process.memory_usage_bytes),
                process.memory_usage_percent,
                process.threads
            )
        } else {
            "No process selected".to_string()
        }
    } else {
        "No process selected".to_string()
    };

    let style = if let Some(ref msg) = app.status_message {
        if msg.starts_with("ERROR") {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        }
    } else {
        Style::default().fg(Color::Cyan)
    };

    let status = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Status ")
                .title_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .style(style);

    f.render_widget(status, area);
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
