use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
    style::{Color, Style},
};
use crate::app::{
    App, CollectionView,
};
use crate::models::CollectionItem;

pub fn draw_collections(f: &mut Frame, app: &mut App, area: Rect) {
    match app.collection_view {
        CollectionView::List => {
            // Collections list with help text inside
            let collections = app.collection_manager.get_collections();
            let mut items = vec![
                ListItem::new("Press 'n' to add new collection, 'd' to delete")
                    .style(Style::default().fg(Color::DarkGray))
            ];

            // Add collection items
            items.extend(collections
                .values()
                .enumerate()
                .map(|(index, collection)| {
                    ListItem::new(format!("{}. {}", index + 1, collection.info.name))
                }));

            let list = List::new(items)
                .block(Block::default()
                    .title("Collections")
                    .borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Yellow))
                .highlight_symbol("▶ ");

            f.render_stateful_widget(
                list,
                area,
                &mut ListState::default().with_selected(Some(app.collection_selected_index + 1)), // +1 to account for help text
            );
        }
        CollectionView::Requests => {
            if let Some(collection_name) = &app.selected_collection {
                if let Some(collection) = app.collection_manager.get_collection(collection_name) {
                    let items: Vec<ListItem> = collection.requests
                        .iter()
                        .enumerate()
                        .map(|(index, item)| {
                            match item {
                                CollectionItem::Request(req) => {
                                    ListItem::new(format!("{}. {} {}", 
                                        index + 1,
                                        req.request.method,
                                        req.name
                                    ))
                                }
                                CollectionItem::Folder(_) => ListItem::new("📁 Folder"),
                            }
                        })
                        .collect();

                    let list = List::new(items)
                        .block(Block::default()
                            .title(format!("Requests in {}", collection_name))
                            .borders(Borders::ALL))
                        .highlight_style(Style::default().fg(Color::Yellow))
                        .highlight_symbol("▶ ");

                    f.render_stateful_widget(
                        list,
                        area,
                        &mut ListState::default().with_selected(Some(app.request_selected_index)),
                    );
                }
            }
        }
    }
}
