use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use oxyd_domain::models::SystemMetrics;
use crate::app::AppState;
use super::widgets::{create_gauge_bar, create_sparkline, format_bytes, format_duration};

pub fn render(f: &mut Frame, area: Rect, metrics: &SystemMetrics, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),   
            Constraint::Length(10),  
            Constraint::Length(10),  
            Constraint::Min(6),      
        ])
        .split(area);

    render_system_info(f, chunks[0], metrics);
    render_cpu_with_graph(f, chunks[1], metrics, app);
    render_memory_with_graph(f, chunks[2], metrics, app);
    render_process_summary(f, chunks[3], metrics);
}

fn render_system_info(f: &mut Frame, area: Rect, metrics: &SystemMetrics) {
    let info = &metrics.system_info;
    
    let lines = vec![
        Line::from(vec![
            Span::styled("Hostname: ", Style::default().fg(Color::Cyan)),
            Span::raw(&info.hostname),
        ]),
        Line::from(vec![
            Span::styled("Architecture: ", Style::default().fg(Color::Cyan)),
            Span::raw(&info.architecture),
        ]),
        Line::from(vec![
            Span::styled("OS: ", Style::default().fg(Color::Cyan)),
            Span::raw(&info.os_version),
        ]),
        Line::from(vec![
            Span::styled("Uptime: ", Style::default().fg(Color::Cyan)),
            Span::raw(format_duration(info.uptime_seconds)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" System Information ")
        .style(Style::default().fg(Color::White));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn render_cpu_with_graph(f: &mut Frame, area: Rect, metrics: &SystemMetrics, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),  // Info
            Constraint::Percentage(50),  // Graph
        ])
        .split(area);

    // CPU Info
    let cpu = &metrics.cpu;
    let bar = create_gauge_bar(cpu.overall_usage_percent, 30);
    
    let lines = vec![
        Line::from(vec![
            Span::styled("Overall: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{:.1}%", cpu.overall_usage_percent)),
        ]),
        Line::from(bar),
        Line::from(""),
        Line::from(vec![
            Span::styled("Load: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{:.2} / {:.2} / {:.2}",
                cpu.load_average.one_minute,
                cpu.load_average.five_minutes,
                cpu.load_average.fifteen_minutes
            )),
        ]),
        Line::from(vec![
            Span::styled("Cores: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", cpu.cores.len())),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" CPU Summary ")
        .style(Style::default().fg(Color::Green));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, chunks[0]);

    // CPU Graph
    if let Some(history) = &app.metrics_history {
        let cpu_data = history.cpu_data();
        let sparkline = create_sparkline(&cpu_data, " CPU History ", Color::Green);
        f.render_widget(sparkline, chunks[1]);
    }
}

fn render_memory_with_graph(f: &mut Frame, area: Rect, metrics: &SystemMetrics, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),  // Info
            Constraint::Percentage(50),  // Graph
        ])
        .split(area);

    // Memory Info
    let mem = &metrics.memory;
    let bar = create_gauge_bar(mem.usage_percent, 30);
    
    let lines = vec![
        Line::from(vec![
            Span::styled("Total: ", Style::default().fg(Color::Cyan)),
            Span::raw(format_bytes(mem.total_bytes)),
        ]),
        Line::from(vec![
            Span::styled("Used: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} ({:.1}%)", 
                format_bytes(mem.used_bytes),
                mem.usage_percent
            )),
        ]),
        Line::from(bar),
        Line::from(""),
        Line::from(vec![
            Span::styled("Available: ", Style::default().fg(Color::Cyan)),
            Span::raw(format_bytes(mem.available_bytes)),
        ]),
        Line::from(vec![
            Span::styled("Swap: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} ({:.1}%)",
                format_bytes(mem.swap_used_bytes),
                mem.swap_usage_percent
            )),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Memory Summary ")
        .style(Style::default().fg(Color::Blue));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, chunks[0]);

    // Memory Graph
    if let Some(history) = &app.metrics_history {
        let mem_data = history.memory_data();
        let sparkline = create_sparkline(&mem_data, " Memory History ", Color::Blue);
        f.render_widget(sparkline, chunks[1]);
    }
}

fn render_process_summary(f: &mut Frame, area: Rect, metrics: &SystemMetrics) {
    let procs = &metrics.processes;
    
    let lines = vec![
        Line::from(vec![
            Span::styled("Total: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}", procs.total_count)),
        ]),
        Line::from(vec![
            Span::styled("Running: ", Style::default().fg(Color::Green)),
            Span::raw(format!("{}", procs.running_count)),
        ]),
        Line::from(vec![
            Span::styled("Sleeping: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", procs.sleeping_count)),
        ]),
        Line::from(vec![
            Span::styled("Stopped: ", Style::default().fg(Color::Magenta)),
            Span::raw(format!("{}", procs.stopped_count)),
        ]),
        Line::from(vec![
            Span::styled("Zombie: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{}", procs.zombie_count)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Process Summary ")
        .style(Style::default().fg(Color::Magenta));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

