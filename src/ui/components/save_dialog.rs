use itertools::Itertools;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    style::{Color, Style},
};
use crate::app::App;

pub fn draw_save_dialog(f: &mut Frame, app: &App, area: Rect) {
    if !app.save_dialog_visible {
        return;
    }

    let collections: Vec<_> = app.collection_manager
        .get_collections()
        .values()
        .collect::<Vec<_>>()
        .into_iter()
        .sorted_by(|a, b| b.info.created_at.cmp(&a.info.created_at))
        .collect();

    // Create list items
    let items: Vec<ListItem> = if collections.is_empty() {
        vec![ListItem::new("No collections available. Create one first.")]
    } else {
        collections.iter()
            .map(|collection| ListItem::new(collection.info.name.clone()))
            .collect()
    };

    // Create a simple floating window in the center
    let width = 40;
    let height = 12;
    let x = area.width.saturating_sub(width) / 2;
    let y = area.height.saturating_sub(height) / 2;

    let dialog_area = Rect::new(x, y, width, height);

    let list = List::new(items)
        .block(Block::default()
            .title("Select Collection")
            .borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow))
        .highlight_symbol("▶ ");

    // Clear background and render dialog
    f.render_widget(Clear, dialog_area);
    f.render_stateful_widget(
        list,
        dialog_area,
        &mut ListState::default().with_selected(Some(app.save_dialog_selected_index)),
    );
} 