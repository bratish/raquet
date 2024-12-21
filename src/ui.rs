use ratatui::{
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, List, ListItem, Clear, ListState},
    text::Line,
};
use itertools::Itertools;

use crate::app::{
    App, Field, InputMode, NavItem, HttpMethod, CollectionView, 
    CollectionDialogState, DialogFocus, CollectionsFocus, DialogButtonFocus,
    HeaderEditState,
};
use crate::collections::CollectionItem;

pub fn draw(f: &mut Frame, app: &mut App) {
    // Create a layout with a left navigation panel
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // Navigation panel
            Constraint::Min(0),     // Main content
        ])
        .split(f.size());

    // Draw navigation panel
    let nav_items: Vec<ListItem> = NavItem::all()
        .iter()
        .map(|item| {
            let style = if *item == app.nav_selected {
                if app.active_field == Field::NavPanel {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::REVERSED)
                } else {
                    Style::default().fg(Color::Yellow)
                }
            } else {
                Style::default()
            };

            let text = if *item == app.nav_selected {
                format!("▶ {}", item.as_str())
            } else {
                format!("  {}", item.as_str())
            };

            ListItem::new(text).style(style)
        })
        .collect();

    let nav_list = List::new(nav_items)
        .block(Block::default()
            .title("Raquet (↑↓)")
            .borders(Borders::ALL)
            .border_style(if app.active_field == Field::NavPanel {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }))
        .highlight_style(Style::default().fg(Color::Yellow));

    f.render_widget(nav_list, main_layout[0]);

    // If collections is being shown, draw it over the main content
    if app.show_collections {
        draw_collections(f, app, main_layout[1]);
        return;
    }

    // If history is being shown, draw it over the main content
    if app.show_history {
        draw_history(f, app, main_layout[1]);
    } else {
        // Draw main content in the right panel
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)  // Remove margin to allow banner to stretch fully
            .constraints([
                Constraint::Length(1),   // Collection banner + Save button (reduced to 1)
                Constraint::Min(0),      // Content area
            ])
            .split(main_layout[1]);

        // Create a horizontal layout for Collection banner and Save button
        let banner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)  // Remove margin
            .constraints([
                Constraint::Min(0),      // Collection name + divider line
                Constraint::Length(20),  // Save button
            ])
            .split(chunks[0]);

        // Draw collection banner with full-width divider
        let collection_banner = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                format!(" {} ", app.selected_collection.as_deref().unwrap_or("No Collection Selected")),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            ))
            .title_alignment(Alignment::Left);
        
        // Draw the banner background first to get the full-width line
        f.render_widget(collection_banner, chunks[0]);  // Render on full width first

        // Draw save button if a collection is selected
        if app.selected_collection.is_some() && !app.is_request_in_collection() {
            let save_button = Paragraph::new("[ Save to Collection ]")
                .alignment(Alignment::Right)
                .style(Style::default().fg(Color::Green));
            f.render_widget(save_button, banner_chunks[1]);
        }

        // Create layout for the main content area
        let content_area = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)  // Add margin for the content
            .constraints([
                Constraint::Length(3),   // Method + URL + Go row
                Constraint::Length(8),   // Headers + Body row
                Constraint::Length(1),   // Separator line
                Constraint::Min(0),      // Response section
            ])
            .split(chunks[1]);

        // Create a horizontal layout for Method, URL, Go and Save buttons
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(10),  // Method
                Constraint::Percentage(78),  // URL (increased)
                Constraint::Percentage(7),   // Go button
                Constraint::Percentage(5),   // Save button (reduced)
            ])
            .split(content_area[0]);

        // Method Selection (in left 10%)
        let method_block = Block::default()
            .title(match app.input_mode {
                InputMode::Editing(Field::Method) => "Method (Use ↑↓ to change)",
                _ => "Method",
            })
            .borders(Borders::ALL)
            .border_style(style_for_field(Field::Method, app));
        let method = Paragraph::new(app.method.as_str())
            .block(method_block)
            .style(style_for_field(Field::Method, app)
                .fg(app.method.color()))
            .alignment(Alignment::Center);
        f.render_widget(method, top_chunks[0]);

        // URL Input (in middle 80%)
        let url_block = Block::default()
            .title("URL")
            .borders(Borders::ALL)
            .border_style(style_for_field(Field::Url, app));
        let url = Paragraph::new(app.url.as_str())
            .block(url_block)
            .style(style_for_field(Field::Url, app));
        f.render_widget(url, top_chunks[1]);

        // Go button
        let button_style = style_for_field(Field::SendButton, app);
        let send_button = Paragraph::new("Go")
            .block(Block::default().borders(Borders::ALL))
            .style(button_style)
            .alignment(Alignment::Center);
        f.render_widget(send_button, top_chunks[2]);

        // Save button
        let save_button_style = style_for_field(Field::SaveButton, app);
        let save_button = Paragraph::new("+")
            .block(Block::default().borders(Borders::ALL))
            .style(save_button_style)
            .alignment(Alignment::Center);
        f.render_widget(save_button, top_chunks[3]);

        // Create a horizontal layout for Headers and Body
        let middle_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),  // Headers
                Constraint::Percentage(70),  // Body
            ])
            .split(content_area[1]);

        // Headers section
        let headers_block = Block::default()
            .title(match app.header_edit_state {
                HeaderEditState::Viewing => "Headers (Enter to edit)",
                HeaderEditState::Selecting => "Headers (↑↓: select, Enter: edit, Space: toggle, Esc: exit)",
                HeaderEditState::EditingKeyInPlace => "Headers (Enter/Tab: next, Esc: cancel)",
                HeaderEditState::EditingValueInPlace => "Headers (Enter: save, Esc: cancel)",
            })
            .borders(Borders::ALL)
            .border_style(style_for_field(Field::Headers, app));

        match app.header_edit_state {
            HeaderEditState::Viewing | HeaderEditState::Selecting => {
                // Create initial header items
                let mut header_items: Vec<ListItem> = app.get_ordered_headers()
                    .iter()
                    .enumerate()
                    .map(|(idx, (key, value))| {
                        let enabled = app.header_enabled.get(key).copied().unwrap_or(true);
                        let checkbox = if enabled { "[✓] " } else { "[ ] " };
                        let line = format!("{}{}: {}", checkbox, key, value);
                        let style = if app.header_edit_state == HeaderEditState::Selecting 
                            && Some(key) == app.headers.keys().nth(app.selected_header_index) {
                            Style::default().fg(Color::Yellow)
                        } else if !enabled {
                            Style::default().fg(Color::DarkGray)
                        } else {
                            Style::default()
                        };
                        ListItem::new(line).style(style)
                    })
                    .collect();

                // Add blank header item at the end with placeholder text
                let blank_line = format!("[ ] <key>: <value>");
                let style = if app.header_edit_state == HeaderEditState::Selecting 
                    && app.selected_header_index >= app.headers.len() {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                header_items.push(ListItem::new(blank_line).style(style));

                let list_height = header_items.len() as u16;
                let headers_list = List::new(header_items)
                    .block(headers_block)
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .highlight_symbol("▶ ");

                // Create a mutable ListState for scrolling
                let mut list_state = ListState::default()
                    .with_selected(Some(app.selected_header_index));

                // Calculate scroll position to keep selected item in view
                let viewport_height = middle_chunks[0].height.saturating_sub(2);
                if app.selected_header_index as u16 >= app.headers_scroll + viewport_height {
                    app.headers_scroll = (app.selected_header_index as u16).saturating_sub(viewport_height - 1);
                }

                f.render_stateful_widget(
                    headers_list,
                    middle_chunks[0],
                    &mut list_state,
                );

                // Add scrollbar if needed
                let viewport_height = middle_chunks[0].height.saturating_sub(2);
                
                if list_height > viewport_height {
                    let scrollbar = Scrollbar::default()
                        .orientation(ScrollbarOrientation::VerticalRight)
                        .begin_symbol(Some("↑"))
                        .end_symbol(Some("↓"));

                    f.render_stateful_widget(
                        scrollbar,
                        middle_chunks[0].inner(&Margin { vertical: 1, horizontal: 0 }),
                        &mut ScrollbarState::new(list_height.into())
                            .position(app.headers_scroll as usize),
                    );
                }
            }
            HeaderEditState::EditingKeyInPlace | HeaderEditState::EditingValueInPlace => {
                let mut header_items: Vec<ListItem> = app.get_ordered_headers()
                    .iter()
                    .enumerate()
                    .map(|(idx, (key, value))| {
                        let enabled = app.header_enabled.get(key.as_str()).copied().unwrap_or(true);
                        let checkbox = if enabled { "[✓] " } else { "[ ] " };
                        
                        if idx == app.selected_header_index && app.selected_header_index < app.headers.len() {
                            // Show editing interface for selected header
                            let (display_key, display_value) = if app.header_edit_state == HeaderEditState::EditingKeyInPlace {
                                (format!("{}█", app.header_edit_key), value.to_string())
                            } else {
                                (key.to_string(), format!("{}█", app.header_edit_value))
                            };
                            
                            let line = format!("{}{}: {}", checkbox, display_key, display_value);
                            ListItem::new(line).style(Style::default().fg(Color::Yellow))
                        } else {
                            // Show normal header for non-selected items
                            let line = format!("{}{}: {}", checkbox, key, value);
                            ListItem::new(line).style(if !enabled {
                                Style::default().fg(Color::DarkGray)
                            } else {
                                Style::default()
                            })
                        }
                    })
                    .collect();

                // Always add new header item at the end when editing
                if app.selected_header_index >= app.headers.len() {
                    let (display_key, display_value) = if app.header_edit_state == HeaderEditState::EditingKeyInPlace {
                        (format!("{}█", app.header_edit_key), String::new())
                    } else {
                        (app.header_edit_key.clone(), format!("{}█", app.header_edit_value))
                    };
                    
                    let line = format!("[ ] {}: {}", display_key, display_value);
                    header_items.push(ListItem::new(line).style(Style::default().fg(Color::Yellow)));
                }

                let headers_list = List::new(header_items)
                    .block(headers_block)
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .highlight_symbol("▶ ");

                // Create a mutable ListState for scrolling
                let mut list_state = ListState::default()
                    .with_selected(Some(app.selected_header_index));

                f.render_stateful_widget(
                    headers_list,
                    middle_chunks[0],
                    &mut list_state,
                );
            }
        }

        // Body (now in right 70%)
        let body_block = Block::default()
            .title("Body")
            .borders(Borders::ALL)
            .border_style(style_for_field(Field::Body, app));
        let body = Paragraph::new(app.body.as_str())
            .block(body_block)
            .style(style_for_field(Field::Body, app));
        f.render_widget(body, middle_chunks[1]);

        // Add a horizontal line with "Response" label (always visible)
        let separator = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                " Response ",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            ))
            .title_alignment(Alignment::Left);
        
        f.render_widget(separator, content_area[2]);  // Render in the separator chunk

        // Response section (always visible)
        let response_area = content_area[3];  // Use the fourth chunk for response area

        // Split response area into metadata+headers and body
        let response_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),  // Left panel: Status, Time, Size, Headers
                Constraint::Percentage(70),  // Right panel: Body
            ])
            .split(response_area);

        // Left panel layout (metadata and headers)
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),  // Info box
                Constraint::Min(0),     // Headers box
            ])
            .split(response_chunks[0]);

        if let Some(_response) = &app.response {
            // Render metadata if response exists
            if let Some(metadata) = &app.response_metadata {
                let status_text = metadata.status_text.as_deref().unwrap_or("Unknown");
                let status_style = match metadata.status {
                    Some(status) if status < 300 => Style::default().fg(Color::Green),
                    Some(status) if status < 400 => Style::default().fg(Color::Yellow),
                    Some(_) => Style::default().fg(Color::Red),
                    None => Style::default(),
                };

                let size_text = format_size(metadata.size_bytes);
                let metadata_text = vec![
                    Line::from(vec![
                        Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(
                            format!("{} {}", metadata.status.unwrap_or(0), status_text),
                            status_style,
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("Time: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(format!("{} ms", metadata.time_ms)),
                    ]),
                    Line::from(vec![
                        Span::styled("Size: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(size_text),
                    ]),
                ];

                let metadata_widget = Paragraph::new(metadata_text)
                    .block(Block::default().borders(Borders::ALL).title("Info"))
                    .alignment(Alignment::Left);  // Ensure left alignment
                f.render_widget(metadata_widget, left_chunks[0]);

                // Extract and render response headers (not request headers)
                let headers_text = metadata.response_headers
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");

                let headers_widget = Paragraph::new(headers_text)
                    .block(Block::default().borders(Borders::ALL).title("Response Headers"))
                    .wrap(ratatui::widgets::Wrap { trim: true });
                f.render_widget(headers_widget, left_chunks[1]);
            }
        } else {
            // Render empty info box
            let empty_info = Paragraph::new("No response yet")
                .block(Block::default().borders(Borders::ALL).title("Info"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(empty_info, left_chunks[0]);

            // Render empty headers box
            let empty_headers = Paragraph::new("No headers")
                .block(Block::default().borders(Borders::ALL).title("Response Headers"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(empty_headers, left_chunks[1]);
        }

        // Right panel (body)
        if let Some(_response) = &app.response {
            // Right panel (body only)
            let body_text = if let Some(body) = _response.split("\n\nBody:\n").nth(1) {
                body
            } else {
                _response.as_str()
            };

            let body_widget = Paragraph::new(body_text)
                .block(Block::default().borders(Borders::ALL).title("Body"))
                .wrap(ratatui::widgets::Wrap { trim: true })
                .scroll((app.response_scroll, 0));
            f.render_widget(body_widget, response_chunks[1]);

            // Add scrollbar to body section
            let response_lines = _response.lines().count() as u16;
            let viewport_height = response_chunks[1].height.saturating_sub(2);
            
            if response_lines > viewport_height {
                let scrollbar = Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓"));

                f.render_stateful_widget(
                    scrollbar,
                    response_chunks[1].inner(&Margin { vertical: 1, horizontal: 0 }),
                    &mut ScrollbarState::new(response_lines.into())
                        .position(app.response_scroll as usize),
                );
            }
        } else {
            // Render empty body box
            let empty_body = Paragraph::new("No response body")
                .block(Block::default().borders(Borders::ALL).title("Body"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(empty_body, response_chunks[1]);
        }

        // Show cursor in active input field
        if let InputMode::Editing(field) = app.input_mode {
            match field {
                Field::Url => {
                    f.set_cursor(
                        top_chunks[1].x + app.cursor_position as u16 + 1,
                        top_chunks[1].y + 1,
                    );
                }
                Field::Body => {
                    f.set_cursor(
                        middle_chunks[1].x + (app.body.len() as u16 % (middle_chunks[1].width - 2)) + 1,
                        middle_chunks[1].y + 1 + (app.body.len() as u16 / (middle_chunks[1].width - 2)),
                    );
                }
                _ => {}
            }
        }
    }

    // Draw collection selector popup if active
    if app.show_collection_selector {
        let collections: Vec<_> = app.collection_manager
            .get_collections()
            .values()
            .collect::<Vec<_>>()
            .into_iter()
            .sorted_by(|a, b| b.info.created_at.cmp(&a.info.created_at))
            .collect();

        let items: Vec<ListItem> = collections
            .iter()
            .map(|collection| {
                ListItem::new(collection.info.name.clone())
            })
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .title("Select Collection")
                .borders(Borders::ALL))
            .highlight_style(Style::default().fg(Color::Yellow))
            .highlight_symbol("▶ ");

        let area = centered_rect(30, 10, f.size());
        f.render_widget(Clear, area); // Clear the background
        f.render_stateful_widget(
            list,
            area,
            &mut ListState::default().with_selected(Some(app.selector_collection_index)),
        );
    }

    // Draw method selector popup if active
    if app.show_method_selector {
        let methods = HttpMethod::all();
        let items: Vec<ListItem> = methods
            .iter()
            .map(|method| {
                ListItem::new(method.as_str())
                    .style(Style::default().fg(method.color()))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .title("Select Method")
                .borders(Borders::ALL))
            .highlight_style(Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ ");

        let area = centered_rect(20, 8, f.size());
        f.render_widget(Clear, area); // Clear the background
        f.render_stateful_widget(
            list,
            area,
            &mut ListState::default().with_selected(Some(app.selector_method_index)),
        );
    }
}

fn draw_history(f: &mut Frame, app: &App, area: Rect) {
    let history_items: Vec<ListItem> = app
        .history
        .get_entries()
        .iter()
        .map(|entry| {
            let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S");
            let method = HttpMethod::from_str(&entry.request.method)
                .unwrap_or(HttpMethod::GET);
            
            let title = Line::from(vec![
                Span::styled(
                    format!("{} ", method.as_str()),
                    Style::default().fg(method.color())
                ),
                Span::raw(format!("{} ({})",
                    entry.request.url,
                    timestamp
                )),
            ]);
            
            let status = entry
                .response
                .as_ref()
                .and_then(|r| r.status)
                .map(|s| s.to_string())
                .unwrap_or_default();

            let style = if app.history.get_entries().get(app.history_selected_index)
                .map(|entry_ref| entry_ref == entry)
                .unwrap_or(false)
            {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            ListItem::new(vec![
                title,
                Line::from(format!("Status: {}", status)),
            ])
            .style(style)
        })
        .collect();

    let history_list = List::new(history_items)
        .block(Block::default().title("History (Esc to close)").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow));

    f.render_widget(Clear, area); // Clear the background
    f.render_widget(history_list, area);
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

fn format_size(size: usize) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn draw_collections(f: &mut Frame, app: &mut App, area: Rect) {
    let collections = app.collection_manager.get_collections();

    match app.collection_view {
        CollectionView::List => {
            // Split area into button and list
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Button height
                    Constraint::Min(0),     // List takes remaining space
                ])
                .split(area);

            // Draw "New Collection" button
            let new_button = Paragraph::new("[ + New Collection ]")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(if app.collections_focus == CollectionsFocus::NewButton {
                    Color::Yellow
                } else {
                    Color::Green
                }));
            f.render_widget(new_button, chunks[0]);

            // Draw collections list
            let items: Vec<ListItem> = collections
                .values()
                .collect::<Vec<_>>()
                .into_iter()
                .sorted_by(|a, b| b.info.created_at.cmp(&a.info.created_at))
                .map(|collection| {
                    let title = format!("{} ({} requests)", 
                        collection.info.name,
                        count_requests(&collection.item)
                    );
                    ListItem::new(title)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default()
                    .title("Collections (Tab: New Collection, Esc: Close)")
                    .borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Yellow))
                .highlight_symbol(if app.collections_focus == CollectionsFocus::List {
                    "▶ "
                } else {
                    "  "
                });

            f.render_stateful_widget(
                list,
                chunks[1],
                &mut ListState::default().with_selected(Some(app.collection_selected_index)),
            );
        }
        CollectionView::Requests => {
            if let Some(collection_name) = &app.selected_collection {
                if let Some(collection) = collections.get(collection_name) {
                    // Draw requests in the selected collection
                    let items: Vec<ListItem> = get_request_items(&collection.item)
                        .map(|(name, method)| {
                            let method_style = HttpMethod::from_str(&method)
                                .map(|m| m.color())
                                .unwrap_or(Color::White);

                            ListItem::new(Line::from(vec![
                                Span::styled(
                                    format!("{} ", method),
                                    Style::default().fg(method_style)
                                ),
                                Span::raw(name),
                            ]))
                        })
                        .collect();

                    let list = List::new(items)
                        .block(Block::default()
                            .title(format!("{} (Esc to go back)", collection_name))
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

    // Draw dialog if active
    match app.collection_dialog {
        CollectionDialogState::NewCollection => {
            let dialog_area = centered_rect(60, 12, area);  // Made taller to accommodate larger description
            let dialog_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Name field
                    Constraint::Length(5),  // Description field (increased height)
                    Constraint::Length(3),  // Buttons
                ])
                .margin(1)
                .split(dialog_area);

            // Draw dialog background
            let dialog_block = Block::default()
                .title("New Collection")
                .borders(Borders::ALL);
            f.render_widget(Clear, dialog_area);
            f.render_widget(dialog_block, dialog_area);

            // Draw name input field
            let name_style = if app.dialog_focus == DialogFocus::Name {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            let name_input = Paragraph::new(app.new_collection_name.as_str())
                .block(Block::default()
                    .title("Name")
                    .borders(Borders::ALL)
                    .border_style(name_style));
            f.render_widget(name_input, dialog_chunks[0]);

            // Draw description input field with scrolling
            let desc_style = if app.dialog_focus == DialogFocus::Description {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let desc_input = Paragraph::new(app.new_collection_description.as_str())
                .block(Block::default()
                    .title("Description")
                    .borders(Borders::ALL)
                    .border_style(desc_style))
                .wrap(ratatui::widgets::Wrap { trim: true })
                .scroll((app.description_scroll, 0));
            f.render_widget(desc_input, dialog_chunks[1]);

            // Draw buttons
            let buttons = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(dialog_chunks[2]);

            let save_button = Paragraph::new("[ Save ]")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(
                    if app.dialog_focus == DialogFocus::Buttons && app.dialog_button_focus == DialogButtonFocus::Save {
                        Color::Yellow
                    } else {
                        Color::Green
                    }
                ));

            let cancel_button = Paragraph::new("[ Cancel ]")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(
                    if app.dialog_focus == DialogFocus::Buttons && app.dialog_button_focus == DialogButtonFocus::Cancel {
                        Color::Yellow
                    } else {
                        Color::White
                    }
                ));

            f.render_widget(save_button, buttons[0]);
            f.render_widget(cancel_button, buttons[1]);

            // Show cursor only in text input fields
            match app.dialog_focus {
                DialogFocus::Name => {
                    f.set_cursor(
                        dialog_chunks[0].x + app.new_collection_name.len() as u16 + 1,
                        dialog_chunks[0].y + 1,
                    );
                }
                DialogFocus::Description => {
                    // Calculate cursor position based on current line
                    let current_line = app.new_collection_description.matches('\n').count() as u16;
                    let last_line = app.new_collection_description
                        .split('\n')
                        .last()
                        .map(|line| line.len())
                        .unwrap_or(0);

                    // Adjust scroll position if cursor would be outside visible area
                    let visible_height = dialog_chunks[1].height.saturating_sub(2);
                    if current_line >= visible_height + app.description_scroll {
                        app.description_scroll = current_line.saturating_sub(visible_height) + 1;
                    }

                    f.set_cursor(
                        dialog_chunks[1].x + last_line as u16 + 1,
                        dialog_chunks[1].y + 1 + current_line.saturating_sub(app.description_scroll),
                    );
                }
                DialogFocus::Buttons => {
                    // No cursor when focused on buttons
                }
            }
        }
        _ => {}
    }
}

// Helper function to count requests in a collection
fn count_requests(items: &[CollectionItem]) -> usize {
    items.iter().fold(0, |count, item| {
        match item {
            CollectionItem::Request(_) => count + 1,
            CollectionItem::Folder(folder) => count + count_requests(&folder.item),
        }
    })
}

// Helper function to get request items with their methods
fn get_request_items(items: &[CollectionItem]) -> impl Iterator<Item = (String, String)> + '_ {
    items.iter().filter_map(|item| match item {
        CollectionItem::Request(req) => Some((
            req.name.clone(),
            req.request.method.clone(),
        )),
        CollectionItem::Folder(_) => None, // Skip folders for now
    })
}

// Helper function to create a centered rect
fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
} 