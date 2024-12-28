use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style},
};
use crate::app::{App, Field, InputMode};
use super::{
    headers::draw_headers,
    save_dialog::draw_save_dialog,
};

pub fn draw_request(f: &mut Frame, app: &mut App, area: Rect) {
    // Top area for URL and buttons
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),  // Method
            Constraint::Percentage(78),  // URL
            Constraint::Percentage(7),   // Go button
            Constraint::Percentage(5),   // Save button
        ])
        .split(area);

    // Create vertical layout for the whole area
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // URL row
            Constraint::Percentage(40),  // Headers
            Constraint::Percentage(60),  // Request body
        ])
        .split(area);

    // Draw top row components
    draw_method(f, app, top_chunks[0]);
    draw_url(f, app, top_chunks[1]);
    draw_send_button(f, app, top_chunks[2]);

    // Save button
    let save_button_style = if app.selected_collection.is_some() && !app.is_request_in_collection() {
        style_for_field(Field::SaveButton, app).fg(Color::Green)
    } else {
        style_for_field(Field::SaveButton, app)
    };

    let save_button = Paragraph::new("+")
        .block(Block::default().borders(Borders::ALL))
        .style(save_button_style)
        .alignment(Alignment::Center);
    f.render_widget(save_button, top_chunks[3]);

    // Draw headers and body
    draw_headers(f, app, main_chunks[1]);
    draw_request_body(f, app, main_chunks[2]);
}

fn style_for_field(field: Field, app: &App) -> Style {
    let is_active = app.active_field == field;
    let is_editing = matches!(app.input_mode, InputMode::Editing(f) if f == field);

    let mut style = Style::default();
    if is_active {
        style = style.fg(Color::Yellow);
    }
    if is_editing {
        style = style.add_modifier(Modifier::BOLD);
    }
    style
}

fn draw_method(f: &mut Frame, app: &App, area: Rect) {
    let method_block = Block::default()
        .title(if app.show_method_selector { "Method (↑↓)" } else { "Method" })
        .borders(Borders::ALL)
        .border_style(style_for_field(Field::Method, app));

    let method = Paragraph::new(app.method.as_str())
        .block(method_block)
        .style(Style::default().fg(app.method.color()))
        .alignment(Alignment::Center);
    f.render_widget(method, area);
}

fn draw_url(f: &mut Frame, app: &App, area: Rect) {
    let url_block = Block::default()
        .title("URL")
        .borders(Borders::ALL)
        .border_style(style_for_field(Field::Url, app));

    let spans = if app.input_mode == InputMode::Editing(Field::Url) {
        let mut styled_spans = Vec::new();

        if let Some(selection_start) = app.selection_start {
            let (start, end) = if selection_start <= app.cursor_position {
                (selection_start.min(app.url.len()), app.cursor_position.min(app.url.len()))
            } else {
                (app.cursor_position.min(app.url.len()), selection_start.min(app.url.len()))
            };

            // Before selection
            if start > 0 {
                styled_spans.push(Span::raw(&app.url[..start]));
            }

            // Selected text
            styled_spans.push(
                Span::styled(
                    &app.url[start..end],
                    Style::default().bg(Color::Gray).fg(Color::Black)
                )
            );

            // After selection
            if end < app.url.len() {
                styled_spans.push(Span::raw(&app.url[end..]));
            }
        } else {
            // No selection, just cursor
            let url_chars: Vec<char> = app.url.chars().collect();
            let pos = app.cursor_position.min(url_chars.len());

            // Text before cursor
            if pos > 0 {
                styled_spans.push(Span::raw(url_chars[..pos].iter().collect::<String>()));
            }

            // Cursor
            if pos < url_chars.len() {
                styled_spans.push(Span::styled(
                    url_chars[pos].to_string(),
                    Style::default().bg(Color::Yellow)
                ));
                
                // Text after cursor
                if pos + 1 < url_chars.len() {
                    styled_spans.push(Span::raw(url_chars[pos + 1..].iter().collect::<String>()));
                }
            } else {
                styled_spans.push(Span::styled(" ", Style::default().bg(Color::Yellow)));
            }
        }

        Line::from(styled_spans)
    } else {
        Line::from(app.url.as_str())
    };

    let url = Paragraph::new(spans)
        .block(url_block)
        .style(style_for_field(Field::Url, app));

    f.render_widget(url, area);
}

fn draw_send_button(f: &mut Frame, app: &App, area: Rect) {
    let button_style = style_for_field(Field::SendButton, app);
    let send_button = Paragraph::new("Go")
        .block(Block::default().borders(Borders::ALL))
        .style(button_style)
        .alignment(Alignment::Center);
    f.render_widget(send_button, area);
}

pub fn draw_request_body(f: &mut Frame, app: &mut App, area: Rect) {
    // ... existing code ...
}
