use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style},
};
use crate::app::{App, Field, InputMode};

pub fn draw_request_body(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("Request Body")
        .borders(Borders::ALL)
        .border_style(if app.active_field == Field::RequestBody {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let inner = block.inner(area);
    
    // Insert cursor into text if we're editing
    let mut content = app.body.clone();
    if app.input_mode == InputMode::Editing(Field::RequestBody) {
        content.insert(app.cursor_position, '|');
    }

    let paragraph = Paragraph::new(content)
        .style(Style::default())
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: false });  // Enable text wrapping

    f.render_widget(paragraph, area);
} 