use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use oxyd_domain::models::SystemMetrics;
use crate::app::AppState;
use super::widgets::{create_gauge_bar, format_bytes};

pub fn render(f: &mut Frame, area: Rect, metrics: &SystemMetrics, _app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),  // Disk table
        ])
        .split(area);

    render_disk_table(f, chunks[0], metrics);
}

fn render_disk_table(f: &mut Frame, area: Rect, metrics: &SystemMetrics) {
    let header = Row::new(vec![
        Cell::from("Device").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Mount").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Type").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Total").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Used").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Free").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Usage").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = metrics.disks.iter().map(|disk| {
        let usage_bar = create_gauge_bar(disk.info.usage_percent, 20);
        let usage_text = format!("{:.1}%", disk.info.usage_percent);
        
        Row::new(vec![
            Cell::from(disk.info.device.clone()),
            Cell::from(disk.info.mount_point.clone()),
            Cell::from(disk.info.filesystem.clone()),
            Cell::from(format_bytes(disk.info.total_bytes)),
            Cell::from(format_bytes(disk.info.used_bytes)),
            Cell::from(format_bytes(disk.info.free_bytes)),
            Cell::from(Line::from(usage_bar)),
        ])
    }).collect();

    // If no disks, show placeholder
    let rows = if rows.is_empty() {
        vec![Row::new(vec![
            Cell::from("No disk information available").style(Style::default().fg(Color::DarkGray))
        ])]
    } else {
        rows
    };

    let table = Table::new(rows, vec![
        Constraint::Length(15),  // Device
        Constraint::Length(15),  // Mount
        Constraint::Length(10),  // Type
        Constraint::Length(12),  // Total
        Constraint::Length(12),  // Used
        Constraint::Length(12),  // Free
        Constraint::Min(20),     // Usage bar
    ])
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Disk Usage ")
            .style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(table, area);
}

