use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    style::{Color, Style, Modifier},
};
use crate::app::{App, Field, HeaderEditState};

pub fn draw_headers(f: &mut Frame, app: &mut App, area: Rect) {
    let header_block = Block::default()
        .title("Headers")
        .borders(Borders::ALL)
        .border_style(if app.active_field == Field::Headers {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    // Create inner layout with help text and list
    let inner_area = header_block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Help text
            Constraint::Min(0),     // Headers list
        ])
        .split(inner_area);

    // Draw the block first
    f.render_widget(header_block, area);

    // Draw help text inside the block
    let help_text = Paragraph::new("Press 'n' to add new header, 'd' to delete")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help_text, chunks[0]);

    let ordered_headers = app.get_ordered_headers();
    let mut items: Vec<ListItem> = ordered_headers
        .iter()
        .enumerate()
        .map(|(i, (key, value))| {
            let checkbox = if *app.header_enabled.get(key).unwrap_or(&true) {
                "[✓]"
            } else {
                "[ ]"
            };
            
            let (key_style, value_style) = match (app.header_edit_state, app.selected_header_index == i) {
                (HeaderEditState::EditingKey, true) => (
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED),
                    Style::default()
                ),
                (HeaderEditState::EditingValue, true) => (
                    Style::default(),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)
                ),
                (_, true) => (
                    Style::default().fg(Color::Yellow),
                    Style::default().fg(Color::Yellow)
                ),
                _ => (Style::default(), Style::default())
            };

            ListItem::new(Line::from(vec![
                Span::styled(checkbox, Style::default()),
                Span::raw(" "),
                Span::styled(key, key_style),
                Span::raw(": "),
                Span::styled(value, value_style),
            ]))
        })
        .collect();

    // Add new header being edited if we're in edit mode and selected_index is at the end
    if app.header_edit_state != HeaderEditState::Selecting && 
       app.selected_header_index >= ordered_headers.len() {
        let key_text = if app.header_edit_state == HeaderEditState::EditingKey {
            let mut k = app.header_edit_key.clone();
            k.insert(app.header_key_cursor, '|');
            k
        } else {
            app.header_edit_key.clone()
        };

        let value_text = if app.header_edit_state == HeaderEditState::EditingValue {
            let mut v = app.header_edit_value.clone();
            v.insert(app.header_value_cursor, '|');
            v
        } else {
            app.header_edit_value.clone()
        };

        items.push(ListItem::new(Line::from(vec![
            Span::raw("[✓] "),
            Span::styled(key_text, Style::default().fg(Color::Yellow)),
            Span::raw(": "),
            Span::styled(value_text, Style::default().fg(Color::Yellow)),
        ])));
    }

    let headers_list = List::new(items)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(headers_list, chunks[1]);
}
