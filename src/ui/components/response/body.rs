use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use crate::app::App;
use serde_json::Value;
use ratatui::text::{Line as TLine, Span as TSpan};

pub fn draw_response_body(f: &mut Frame, app: &mut App, area: Rect) {
    let body_block = Block::default()
        .title("Body")
        .borders(Borders::ALL);

    let formatted_body = if let Some(metadata) = &app.response_metadata {
        let content_type = metadata
            .response_headers
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

    let paragraph = if let Some(metadata) = &app.response_metadata {
        let content_type = metadata
            .response_headers
            .get("content-type")
            .map(|s| s.as_str())
            .unwrap_or("");
        if content_type.contains("application/json") {
            Paragraph::new(colorize_json(&formatted_body))
                .block(body_block)
                .wrap(ratatui::widgets::Wrap { trim: false })
                .scroll((app.response_scroll, 0))
        } else {
            Paragraph::new(formatted_body)
                .block(body_block)
                .wrap(ratatui::widgets::Wrap { trim: false })
                .scroll((app.response_scroll, 0))
        }
    } else {
        Paragraph::new(formatted_body)
            .block(body_block)
            .wrap(ratatui::widgets::Wrap { trim: false })
            .scroll((app.response_scroll, 0))
    };

    f.render_widget(paragraph, area);
}

// Helper function to colorize each line of a JSON string.
// Keys are colored yellow, and string values are colored green.
fn colorize_json(json_str: &str) -> Vec<TLine> {
    let mut lines_vec = Vec::new();
    for line in json_str.lines() {
        let mut spans = Vec::new();
        let parts: Vec<&str> = line.split('"').collect();
        for (i, part) in parts.iter().enumerate() {
            if i % 2 == 0 {
                spans.push(TSpan::raw(*part));
            } else {
                // If the part is immediately followed by a colon in the next segment, treat it as a key.
                let style = if i + 1 < parts.len() && parts[i + 1].trim_start().starts_with(':') {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Green)
                };
                spans.push(TSpan::styled(*part, style));
            }
            if i < parts.len() - 1 {
                spans.push(TSpan::raw("\""));
            }
        }
        lines_vec.push(TLine::from(spans));
    }
    lines_vec
} 