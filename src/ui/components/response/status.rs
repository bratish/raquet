use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style},
};
use crate::app::App;

pub fn draw_response_status(f: &mut Frame, app: &mut App, area: Rect) {
    let status_block = Block::default()
        .title("Status")
        .borders(Borders::ALL);

    let status_style = if let Some(metadata) = &app.response_metadata {
        match metadata.status {
            s if s >= 200 && s < 300 => Style::default().fg(Color::Green),
            s if s >= 300 && s < 400 => Style::default().fg(Color::Blue),
            s if s >= 400 && s < 500 => Style::default().fg(Color::Yellow),
            s if s >= 500 => Style::default().fg(Color::Red),
            _ => Style::default(),
        }
    } else {
        Style::default()
    };

    let status_text = if let Some(metadata) = &app.response_metadata {
        format!("{} {}", metadata.status, metadata.status_text)
    } else {
        "No response".to_string()
    };

    let status = Paragraph::new(status_text)
        .block(status_block)
        .style(status_style)
        .alignment(Alignment::Center);

    f.render_widget(status, area);
} 