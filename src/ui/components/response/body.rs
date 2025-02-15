use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use crate::app::App;
use serde_json::Value;

pub fn draw_response_body(f: &mut Frame, app: &mut App, area: Rect) {
    let body_block = Block::default()
        .title("Body")
        .borders(Borders::ALL);

    let formatted_body = if let Some(metadata) = &app.response_metadata {
        let content_type = metadata.response_headers
            .get("content-type")
            .map(|s| s.as_str())
            .unwrap_or("");
            
        if content_type.contains("application/json") {
            if let Some(body) = &app.response {
                if let Ok(json) = serde_json::from_str::<Value>(body) {
                    serde_json::to_string_pretty(&json)
                        .unwrap_or_else(|_| body.clone())
                } else {
                    body.clone()
                }
            } else {
                String::new()
            }
        } else {
            app.response.clone().unwrap_or_default()
        }
    } else {
        app.response.clone().unwrap_or_default()
    };

    let body = Paragraph::new(formatted_body)
        .block(body_block)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .scroll((app.response_scroll, 0));

    f.render_widget(body, area);
} 