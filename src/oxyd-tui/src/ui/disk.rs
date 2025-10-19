use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Cell, Row, Table},
};

use super::widgets::{create_gauge_bar, format_bytes};
use crate::app::AppState;
use oxyd_domain::models::SystemMetrics;

pub fn render(f: &mut Frame, area: Rect, metrics: &SystemMetrics, _app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10)])
        .split(area);

    render_disk_table(f, chunks[0], metrics);
}

fn render_disk_table(f: &mut Frame, area: Rect, metrics: &SystemMetrics) {
    let header = Row::new(vec![
        Cell::from("Device").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Mount").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Type").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Total").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Used").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Free").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Usage").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let rows: Vec<Row> = metrics
        .disks
        .iter()
        .map(|disk| {
            let usage_bar = create_gauge_bar(disk.info.usage_percent, 20);

            Row::new(vec![
                Cell::from(disk.info.device.clone()),
                Cell::from(disk.info.mount_point.clone()),
                Cell::from(disk.info.filesystem.clone()),
                Cell::from(format_bytes(disk.info.total_bytes)),
                Cell::from(format_bytes(disk.info.used_bytes)),
                Cell::from(format_bytes(disk.info.free_bytes)),
                Cell::from(Line::from(usage_bar)),
            ])
        })
        .collect();

    let rows = if rows.is_empty() {
        vec![Row::new(vec![
            Cell::from("No disk information available").style(Style::default().fg(Color::DarkGray)),
        ])]
    } else {
        rows
    };

    let table = Table::new(
        rows,
        vec![
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Min(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Disk Usage ")
            .style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(table, area);
}
