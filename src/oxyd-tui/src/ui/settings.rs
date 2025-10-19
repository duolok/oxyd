use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{AppState, InputMode};

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(15),   // Settings list
            Constraint::Length(5), // Input box
            Constraint::Length(3), // Instructions
        ])
        .split(area);

    render_title(f, chunks[0]);
    render_settings_list(f, chunks[1], app);
    render_input_box(f, chunks[2], app);
    render_instructions(f, chunks[3], app);
}

fn render_title(f: &mut Frame, area: Rect) {
    let title = Paragraph::new(vec![Line::from(vec![Span::styled(
        "Alert Configuration",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    )
    .alignment(Alignment::Center);

    f.render_widget(title, area);
}

fn render_settings_list(f: &mut Frame, area: Rect, app: &AppState) {
    let items = vec![
        ListItem::new(vec![Line::from(vec![
            Span::styled(
                "CPU Alert Threshold:      ",
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!("{:.0}%", app.cpu_alert_threshold),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  [Press 'c' to edit]",
                Style::default().fg(Color::DarkGray),
            ),
        ])]),
        ListItem::new(Line::from("")),
        ListItem::new(vec![Line::from(vec![
            Span::styled(
                "Memory Alert Threshold:   ",
                Style::default().fg(Color::Blue),
            ),
            Span::styled(
                format!("{:.0}%", app.memory_alert_threshold),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  [Press 'm' to edit]",
                Style::default().fg(Color::DarkGray),
            ),
        ])]),
        ListItem::new(Line::from("")),
        ListItem::new(vec![Line::from(vec![
            Span::styled(
                "Disk Alert Threshold:     ",
                Style::default().fg(Color::Magenta),
            ),
            Span::styled(
                format!("{:.0}%", app.disk_alert_threshold),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  [Press 'd' to edit]",
                Style::default().fg(Color::DarkGray),
            ),
        ])]),
        ListItem::new(Line::from("")),
        ListItem::new(Line::from("")),
        ListItem::new(vec![Line::from(vec![Span::styled(
            "Alert Status:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )])]),
        ListItem::new(vec![Line::from(vec![
            Span::raw("  • CPU:    "),
            if app.last_cpu_alert.is_some() {
                Span::styled(
                    "⚠ TRIGGERED",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled("✓ OK", Style::default().fg(Color::Green))
            },
        ])]),
        ListItem::new(vec![Line::from(vec![
            Span::raw("  • Memory: "),
            if app.last_memory_alert.is_some() {
                Span::styled(
                    "⚠ TRIGGERED",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled("✓ OK", Style::default().fg(Color::Green))
            },
        ])]),
        ListItem::new(vec![Line::from(vec![
            Span::raw("  • Disk:   "),
            if app.last_disk_alert.is_some() {
                Span::styled(
                    "⚠ TRIGGERED",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled("✓ OK", Style::default().fg(Color::Green))
            },
        ])]),
    ];

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Alert Thresholds ")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(Style::default().fg(Color::White)),
    );

    f.render_widget(list, area);
}

fn render_input_box(f: &mut Frame, area: Rect, app: &AppState) {
    let (title, input_text, color) = match app.input_mode {
        InputMode::EditCpuThreshold => (
            " Edit CPU Threshold (0-100) ",
            app.input_buffer.as_str(),
            Color::Yellow,
        ),
        InputMode::EditMemoryThreshold => (
            " Edit Memory Threshold (0-100) ",
            app.input_buffer.as_str(),
            Color::Blue,
        ),
        InputMode::EditDiskThreshold => (
            " Edit Disk Threshold (0-100) ",
            app.input_buffer.as_str(),
            Color::Magenta,
        ),
        _ => (" Input ", "", Color::Gray),
    };

    let is_editing =
        app.input_mode != InputMode::Normal && app.input_mode != InputMode::SearchProcess;

    let border_style = if is_editing {
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let display_text = if is_editing {
        format!("{}_", input_text)
    } else {
        "Press 'c', 'm', or 'd' to edit thresholds".to_string()
    };

    let input = Paragraph::new(display_text)
        .style(Style::default().fg(if is_editing {
            Color::White
        } else {
            Color::DarkGray
        }))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
                .border_style(border_style),
        );

    f.render_widget(input, area);
}

fn render_instructions(f: &mut Frame, area: Rect, app: &AppState) {
    let text = if app.input_mode != InputMode::Normal && app.input_mode != InputMode::SearchProcess
    {
        "Press Enter to save | Esc to cancel"
    } else {
        "c: Edit CPU | m: Edit Memory | d: Edit Disk | Tab: Switch tabs | ?: Help | q: Quit"
    };

    let instructions = Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    f.render_widget(instructions, area);
}
