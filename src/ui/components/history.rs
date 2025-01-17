use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
    style::{Color, Style},
    text::Line,
};
use crate::app::App;

pub fn draw_history(f: &mut Frame, app: &mut App, area: Rect) {
    let history_block = Block::default()
        .title("History (↑↓ to navigate, Enter to select, Esc to close)")
        .borders(Borders::ALL);

    let items: Vec<ListItem> = app.history.get_entries()
        .iter()
        .map(|entry| {
            let style = if app.history.get_entries().get(app.history_selected_index)
                .map(|entry_ref| entry_ref == entry)
                .unwrap_or(false)
            {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let status = entry.response.as_ref()
                .and_then(|r| r.status)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "---".to_string());

            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", entry.request.method),
                    Style::default().fg(Color::Cyan)
                ),
                Span::raw(format!("{} ", entry.request.url)),
                Span::styled(
                    status.clone(),
                    Style::default().fg(if status.starts_with('2') {
                        Color::Green
                    } else if status.starts_with('4') || status.starts_with('5') {
                        Color::Red
                    } else {
                        Color::DarkGray
                    })
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let history_list = List::new(items)
        .block(history_block)
        .highlight_style(Style::default().fg(Color::Yellow));

    let mut list_state = ListState::default()
        .with_selected(Some(app.history_selected_index));

    f.render_stateful_widget(history_list, area, &mut list_state);
}
