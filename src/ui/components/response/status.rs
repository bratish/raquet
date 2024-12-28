use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style},
    text::{Line, Span},
};
use crate::app::App;

pub fn draw_response_status(f: &mut Frame, app: &mut App, area: Rect) {
    let status_block = Block::default()
        .title("Status")
        .borders(Borders::ALL);

    let lines = match &app.response_metadata {
        Some(metadata) => {
            let status = metadata.status.to_string();
            let status_text = metadata.status_text
                .replace(&status, "")
                .trim()
                .to_string();
            let status_color = if status.starts_with('2') {
                Color::Green
            } else if status.starts_with('4') || status.starts_with('5') {
                Color::Red
            } else {
                Color::Yellow
            };

            vec![
                Line::from(vec![
                    Span::raw("Status: "),
                    Span::styled(
                        format!("{} {}", status, status_text),
                        Style::default().fg(status_color)
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Time: "),
                    Span::raw(format!("{}ms", metadata.time_ms)),
                ]),
                Line::from(vec![
                    Span::raw("Size: "),
                    Span::raw(format_size(metadata.size_bytes)),
                ]),
            ]
        }
        None => vec![
            Line::from("Status: No response"),
            Line::from("Time: -"),
            Line::from("Size: -"),
        ],
    };

    let status = Paragraph::new(lines)
        .block(status_block)
        .alignment(Alignment::Left);

    f.render_widget(status, area);
}

fn format_size(size: usize) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
} 