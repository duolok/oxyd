pub mod overview;
pub mod cpu;
pub mod memory;
pub mod processes;
pub mod widgets;
pub mod network;
pub mod disk;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::{app::AppState, tabs::Tab};

pub fn render(f: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  
            Constraint::Min(0),     
            Constraint::Length(3), 
        ])
        .split(f.area());

    render_header(f, chunks[0], app);
    render_content(f, chunks[1], app);
    render_footer(f, chunks[2], app);
}

fn render_header(f: &mut Frame, area: Rect, app: &AppState) {
    let titles: Vec<Line> = Tab::all()
        .iter()
        .map(|tab| {
            let title = format!(" {} ", tab.title());
            Line::from(Span::styled(
                title,
                if *tab == app.current_tab {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                },
            ))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" OXYD System Monitor ")
                .style(Style::default().fg(Color::Cyan)),
        )
        .select(app.current_tab as usize)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}

fn render_content(f: &mut Frame, area: Rect, app: &AppState) {
    if let Some(ref metrics) = app.metrics {
        match app.current_tab {
            Tab::Overview => overview::render(f, area, metrics, app),
            Tab::Cpu => cpu::render(f, area, metrics, app),
            Tab::Memory => memory::render(f, area, metrics, app),
            Tab::Processes => processes::render(f, area, metrics, app),
            Tab::Network => network::render(f, area, metrics, app),  
            Tab::Disk => disk::render(f, area, metrics, app),       
        }
    } else {
        render_loading(f, area);
    }
}

fn render_footer(f: &mut Frame, area: Rect, app: &AppState) {
    let help_text = match app.current_tab {
        Tab::Processes => " ↑/↓: Scroll | PgUp/PgDn: Page | Tab: Next | 1-6: Switch Tab | q: Quit ",
        _ => " Tab: Next Tab | 1-6: Switch Tab | q: Quit ",
    };

    let footer = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Updates: {} ", app.update_count))
        .style(Style::default().fg(Color::Gray));

    let help = ratatui::widgets::Paragraph::new(help_text)
        .block(footer)
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(help, area);
}

fn render_loading(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Loading... ")
        .style(Style::default().fg(Color::Yellow));

    let text = ratatui::widgets::Paragraph::new("Waiting for system metrics...")
        .block(block)
        .style(Style::default().fg(Color::White));

    f.render_widget(text, area);
}

fn render_placeholder(f: &mut Frame, area: Rect, title: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} (Coming Soon) ", title))
        .style(Style::default().fg(Color::Gray));

    let text = ratatui::widgets::Paragraph::new(format!("{} metrics will be displayed here.", title))
        .block(block)
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(text, area);
}
