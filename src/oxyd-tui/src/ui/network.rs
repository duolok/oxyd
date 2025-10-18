use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use oxyd_domain::models::SystemMetrics;
use crate::app::AppState;
use super::widgets::format_bytes;

pub fn render(f: &mut Frame, area: Rect, metrics: &SystemMetrics, _app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Summary
            Constraint::Min(10),    // Interface details
        ])
        .split(area);

    render_network_summary(f, chunks[0], metrics);
    render_interface_table(f, chunks[1], metrics);
}

fn render_network_summary(f: &mut Frame, area: Rect, metrics: &SystemMetrics) {
    let net = &metrics.network;
    
    let lines = vec![
        Line::from(vec![
            Span::styled("Total Bytes Sent:     ", Style::default().fg(Color::Cyan)),
            Span::raw(format_bytes(net.total_bytes_sent)),
        ]),
        Line::from(vec![
            Span::styled("Total Bytes Received: ", Style::default().fg(Color::Cyan)),
            Span::raw(format_bytes(net.total_bytes_received)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Active Interfaces: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", net.interfaces.len())),
        ]),
        Line::from(vec![
            Span::styled("Active Connections: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", net.active_connections.len())),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Network Summary ")
        .style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn render_interface_table(f: &mut Frame, area: Rect, metrics: &SystemMetrics) {
    let net = &metrics.network;
    
    let header = Row::new(vec![
        Cell::from("Interface").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("TX Bytes").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("RX Bytes").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("TX Packets").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("RX Packets").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = net.stats.iter().map(|stat| {
        let status = if net.interfaces.iter().any(|i| i.name == stat.interface && i.is_up) {
            Span::styled("UP", Style::default().fg(Color::Green))
        } else {
            Span::styled("DOWN", Style::default().fg(Color::Red))
        };

        Row::new(vec![
            Cell::from(stat.interface.clone()),
            Cell::from(status),
            Cell::from(format_bytes(stat.bytes_sent)),
            Cell::from(format_bytes(stat.bytes_received)),
            Cell::from(format!("{}", stat.packets_sent)),
            Cell::from(format!("{}", stat.packets_received)),
        ])
    }).collect();

    let table = Table::new(rows, vec![
        Constraint::Length(12),  // Interface
        Constraint::Length(8),   // Status
        Constraint::Length(12),  // TX Bytes
        Constraint::Length(12),  // RX Bytes
        Constraint::Length(12),  // TX Packets
        Constraint::Length(12),  // RX Packets
    ])
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Network Interfaces ")
            .style(Style::default().fg(Color::White)),
    );

    f.render_widget(table, area);
}
