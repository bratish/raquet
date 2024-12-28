use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
};
use crate::app::App;

pub fn draw_response_headers(f: &mut Frame, app: &mut App, area: Rect) {
    let headers_block = Block::default()
        .title("Headers")
        .borders(Borders::ALL);

    let headers: Vec<ListItem> = if let Some(metadata) = &app.response_metadata {
        metadata.response_headers
            .iter()
            .map(|(k, v)| {
                ListItem::new(format!("{}: {}", k, v))
            })
            .collect()
    } else {
        vec![]
    };

    let headers_list = List::new(headers)
        .block(headers_block);

    f.render_widget(headers_list, area);
} 