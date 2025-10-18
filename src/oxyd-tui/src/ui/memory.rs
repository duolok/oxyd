use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use oxyd_domain::models::SystemMetrics;
use crate::app::AppState;
use super::widgets::{create_gauge_bar, format_bytes};

pub fn render(f: &mut Frame, area: Rect, metrics: &SystemMetrics, _app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12),  
            Constraint::Min(6),     
        ])
        .split(area);

    render_ram(f, chunks[0], metrics);
    render_swap(f, chunks[1], metrics);
}

fn render_ram(f: &mut Frame, area: Rect, metrics: &SystemMetrics) {
    let mem = &metrics.memory;
    let bar = create_gauge_bar(mem.usage_percent, 50);
    
    let lines = vec![
        Line::from(vec![
            Span::styled("Total:     ", Style::default().fg(Color::Cyan)),
            Span::raw(format_bytes(mem.total_bytes)),
        ]),
        Line::from(vec![
            Span::styled("Used:      ", Style::default().fg(Color::Cyan)),
            Span::raw(format_bytes(mem.used_bytes)),
            Span::raw(format!(" ({:.1}%)", mem.usage_percent)),
        ]),
        Line::from(vec![
            Span::styled("Available: ", Style::default().fg(Color::Cyan)),
            Span::raw(format_bytes(mem.available_bytes)),
        ]),
        Line::from(vec![
            Span::styled("Free:      ", Style::default().fg(Color::Cyan)),
            Span::raw(format_bytes(mem.free_bytes)),
        ]),
        Line::from(vec![
            Span::styled("Cached:    ", Style::default().fg(Color::Cyan)),
            Span::raw(format_bytes(mem.cached_bytes)),
        ]),
        Line::from(vec![
            Span::styled("Buffers:   ", Style::default().fg(Color::Cyan)),
            Span::raw(format_bytes(mem.buffers_bytes)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Usage: ", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(bar),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" RAM Usage ")
        .style(Style::default().fg(Color::Blue));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn render_swap(f: &mut Frame, area: Rect, metrics: &SystemMetrics) {
    let mem = &metrics.memory;
    let bar = create_gauge_bar(mem.swap_usage_percent, 50);
    
    let lines = vec![
        Line::from(vec![
            Span::styled("Total: ", Style::default().fg(Color::Cyan)),
            Span::raw(format_bytes(mem.swap_total_bytes)),
        ]),
        Line::from(vec![
            Span::styled("Used:  ", Style::default().fg(Color::Cyan)),
            Span::raw(format_bytes(mem.swap_used_bytes)),
            Span::raw(format!(" ({:.1}%)", mem.swap_usage_percent)),
        ]),
        Line::from(vec![
            Span::styled("Free:  ", Style::default().fg(Color::Cyan)),
            Span::raw(format_bytes(mem.swap_free_bytes)),
        ]),
        Line::from(""),
        Line::from(bar),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Swap Usage ")
        .style(Style::default().fg(Color::Magenta));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
