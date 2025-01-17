use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use crate::app::App;

pub fn draw_response_body(f: &mut Frame, app: &mut App, area: Rect) {
    let body_block = Block::default()
        .title("Body")
        .borders(Borders::ALL);

    let body = Paragraph::new(app.response.as_deref().unwrap_or(""))
        .block(body_block)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .scroll((app.response_scroll, 0));

    f.render_widget(body, area);
} 