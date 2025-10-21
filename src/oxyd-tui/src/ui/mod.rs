pub mod cpu;
pub mod disk;
pub mod help;
pub mod memory;
pub mod network;
pub mod notifications;
pub mod overview;
pub mod processes;
pub mod settings;
pub mod widgets;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
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

    if app.show_help {
        help::render_help(f);
    }
}

fn render_header(f: &mut Frame, area: Rect, app: &AppState) {
    let titles: Vec<Line> = Tab::all()
        .iter()
        .map(|tab| {
            let mut title = format!(" {} ", tab.title());

            if matches!(tab, Tab::Notifications) {
                let unread = app.notification_manager.unread_count();
                if unread > 0 {
                    title = format!(" {} ({}) ", tab.title(), unread);
                }
            }

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
                .title(" OXYD ")
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
            Tab::Notifications => notifications::render(f, area, app),
            Tab::Settings => settings::render(f, area, app), // NOVO
        }
    } else {
        render_loading(f, area);
    }
}

fn render_footer(f: &mut Frame, area: Rect, app: &AppState) {
    let unread_notifs = app.notification_manager.unread_count();
    let notif_indicator = if unread_notifs > 0 {
        format!(" Notifications {} ", unread_notifs)
    } else {
        String::new()
    };

    let help_text = match app.current_tab {
        Tab::Processes => {
            if !app.process_filter.is_empty() {
                format!(
                    " Filter: '{}' | /: Search | x: Clear | ↑/↓: Scroll | Tab: Next | 1-8: Switch Tab | ?: Help | q: Quit{}",
                    app.process_filter, notif_indicator
                )
            } else {
                format!(
                    " /: Search | ↑/↓: Scroll | PgUp/PgDn: Page | Tab: Next | 1-8: Switch Tab | ?: Help | q: Quit{}",
                    notif_indicator
                )
            }
        }
        Tab::Settings => format!(
            " c/m/d: Edit thresholds | Tab: Next | 1-8: Switch Tab | ?: Help | q: Quit{}",
            notif_indicator
        ),
        Tab::Notifications => format!(
            " m: Mark all read | x: Clear all | Tab: Next | 1-8: Switch Tab | ?: Help | q: Quit{}",
            notif_indicator
        ),
        _ => format!(
            " Tab: Next Tab | 1-8: Switch Tab | ?: Help | q: Quit{}",
            notif_indicator
        ),
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
