use ratatui::{
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Sparkline},
};

pub fn create_gauge_bar(percentage: f32, width: usize) -> Vec<Span<'static>> {
    let filled = ((percentage / 100.0) * width as f32) as usize;
    let empty = width.saturating_sub(filled);

    let (color, symbol) = match percentage {
        p if p < 50.0 => (Color::Green, "█"),
        p if p < 80.0 => (Color::Yellow, "█"),
        _ => (Color::Red, "█"),
    };

    vec![
        Span::styled(
            symbol.repeat(filled),
            Style::default().fg(color),
        ),
        Span::styled(
            "░".repeat(empty),
            Style::default().fg(Color::DarkGray),
        ),
    ]
}

pub fn create_sparkline<'a>(
    data: &'a [u64],
    title: &'a str,
    color: Color,
) -> Sparkline<'a> {
    Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default().fg(color)),
        )
        .data(data)
        .style(Style::default().fg(color))
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

pub fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
