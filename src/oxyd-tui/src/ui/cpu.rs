use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use oxyd_domain::models::SystemMetrics;
use crate::app::AppState;
use super::widgets::create_gauge_bar;

pub fn render(f: &mut Frame, area: Rect, metrics: &SystemMetrics, _app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  
            Constraint::Min(0),    
        ])
        .split(area);

    render_overall_cpu(f, chunks[0], metrics);
    render_per_core_cpu(f, chunks[1], metrics);
}

fn render_overall_cpu(f: &mut Frame, area: Rect, metrics: &SystemMetrics) {
    let cpu = &metrics.cpu;
    let bar = create_gauge_bar(cpu.overall_usage_percent, 50);
    
    let lines = vec![
        Line::from(vec![
            Span::styled("Overall CPU Usage: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{:.1}%", cpu.overall_usage_percent)),
        ]),
        Line::from(bar),
        Line::from(""),
        Line::from(vec![
            Span::styled("Load Average: ", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw(format!("  1 min:  {:.2}", cpu.load_average.one_minute)),
        ]),
        Line::from(vec![
            Span::raw(format!("  5 min:  {:.2}", cpu.load_average.five_minutes)),
        ]),
        Line::from(vec![
            Span::raw(format!("  15 min: {:.2}", cpu.load_average.fifteen_minutes)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" CPU Metrics ")
        .style(Style::default().fg(Color::Green));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn render_per_core_cpu(f: &mut Frame, area: Rect, metrics: &SystemMetrics) {
    let cpu = &metrics.cpu;

    let title = format!(" CPU Cores ({}) ", cpu.cores.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Green));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let cores_per_row = 4;
    let rows = (cpu.cores.len() as f32 / cores_per_row as f32).ceil() as usize;

    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3); rows])
        .split(inner_area);

    for (i, row_area) in row_chunks.iter().enumerate() {
        let cores_in_row = cpu
            .cores
            .iter()
            .skip(i * cores_per_row)
            .take(cores_per_row)
            .collect::<Vec<_>>();

        let col_constraints = vec![Constraint::Percentage(100 / cores_in_row.len() as u16); cores_in_row.len()];
        let col_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .horizontal_margin(1)
            .split(*row_area);

        for (j, core) in cores_in_row.iter().enumerate() {
            let bar = create_gauge_bar(core.usage_percent, 15);
            let lines = vec![
                Line::from(Span::styled(
                    format!("CPU {:2}: {:.1}%", core.id, core.usage_percent),
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(bar),
            ];
            let paragraph = Paragraph::new(lines);
            f.render_widget(paragraph, col_chunks[j]);
        }
    }
}

