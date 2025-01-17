use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use log::debug;
use super::state::{
    App, Field, InputMode, NavItem, HeaderEditState, 
    HttpMethod
};
use super::{
    CollectionsFocus, CollectionView
};
use chrono::Utc;
use crate::models::collection::{Collection, CollectionInfo};
use arboard::Clipboard;

pub struct InputHandler;

impl InputHandler {
    pub async fn handle_key(app: &mut App, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(' ') => debug!("Space key pressed"),
            KeyCode::Up => debug!("Up key pressed"),
            KeyCode::Down => debug!("Down key pressed"),
            KeyCode::Enter => debug!("Enter key pressed"),
            _ => debug!("Other key pressed: {:?}", key.code),
        }
        
        match app.input_mode {
            InputMode::Normal => Self::handle_normal_mode(app, key).await,
            InputMode::Editing(field) => Self::handle_editing_mode(app, field, key),
        }
    }

    async fn handle_normal_mode(app: &mut App, key: KeyEvent) -> bool {
        // Handle collections first
        if app.show_collections {
            match key.code {
                KeyCode::Char('n') => {
                    // Handle new collection in a different way
                    // (we'll implement this later if needed)
                    return false;
                }
                KeyCode::Char('d') => {
                    if let Some(collection) = app.collection_manager
                        .get_collections()
                        .values()
                        .nth(app.collection_selected_index) 
                    {
                        let name = collection.info.name.clone();
                        app.collection_manager.delete_collection(&name);
                    }
                    return false;
                }
                KeyCode::Esc => {
                    match app.collection_view {
                        CollectionView::List => {
                            app.show_collections = false;
                            app.active_field = Field::NavPanel;
                        }
                        CollectionView::Requests => {
                            app.collection_view = CollectionView::List;
                            app.selected_collection = None;
                        }
                    }
                    return false;
                }
                KeyCode::Enter => {
                    if app.collection_view == CollectionView::List {
                        let collections = app.collection_manager.get_collections();
                        if let Some(collection) = collections.values().nth(app.collection_selected_index) {
                            app.selected_collection = Some(collection.info.name.clone());
                            app.collection_view = CollectionView::Requests;
                            app.request_selected_index = 0;
                        }
                    }
                    return false;
                }
                KeyCode::Up => {
                    match app.collection_view {
                        CollectionView::List => {
                            if app.collection_selected_index > 0 {
                                app.collection_selected_index -= 1;
                            }
                        }
                        CollectionView::Requests => {
                            if let Some(collection_name) = &app.selected_collection {
                                if let Some(collection) = app.collection_manager.get_collection(collection_name) {
                                    if app.request_selected_index > 0 {
                                        app.request_selected_index -= 1;
                                    }
                                }
                            }
                        }
                    }
                    return false;
                }
                KeyCode::Down => {
                    match app.collection_view {
                        CollectionView::List => {
                            let len = app.collection_manager.get_collections().len();
                            if len > 0 {
                                app.collection_selected_index = (app.collection_selected_index + 1) % len;
                            }
                        }
                        CollectionView::Requests => {
                            if let Some(collection_name) = &app.selected_collection {
                                if let Some(collection) = app.collection_manager.get_collection(collection_name) {
                                    let len = collection.requests.len();
                                    if len > 0 {
                                        app.request_selected_index = (app.request_selected_index + 1) % len;
                                    }
                                }
                            }
                        }
                    }
                    return false;
                }
                _ => {}
            }
        }

        // Handle method selector
        if app.show_method_selector {
            match key.code {
                KeyCode::Esc => {
                    app.show_method_selector = false;
                    return false;
                }
                KeyCode::Enter => {
                    let methods = HttpMethod::all();
                    app.method = methods[app.selector_method_index];
                    app.show_method_selector = false;
                    return false;
                }
                KeyCode::Up => {
                    if app.selector_method_index > 0 {
                        app.selector_method_index -= 1;
                    }
                    return false;
                }
                KeyCode::Down => {
                    let methods = HttpMethod::all();
                    app.selector_method_index = (app.selector_method_index + 1) % methods.len();
                    return false;
                }
                _ => return false,
            }
        }

        // Handle save dialog
        if app.save_dialog_visible {
            match key.code {
                KeyCode::Esc => {
                    app.save_dialog_visible = false;
                    return false;
                }
                KeyCode::Enter => {
                    let collection_name = app.collection_manager
                        .get_collections()
                        .values()
                        .nth(app.save_dialog_selected_index)
                        .map(|c| c.info.name.clone());

                    if let Some(name) = collection_name {
                        if let Err(e) = app.save_to_collection(&name) {
                            debug!("Failed to save to collection: {}", e);
                        }
                    }
                    app.save_dialog_visible = false;
                    return false;
                }
                KeyCode::Up => {
                    if app.save_dialog_selected_index > 0 {
                        app.save_dialog_selected_index -= 1;
                    }
                    return false;
                }
                KeyCode::Down => {
                    let len = app.collection_manager.get_collections().len();
                    if len > 0 {
                        app.save_dialog_selected_index = (app.save_dialog_selected_index + 1) % len;
                    }
                    return false;
                }
                _ => return false,
            }
        }

        // Only handle header-specific keys when Headers field is active
        if app.active_field == Field::Headers {
            match key.code {
                KeyCode::Char('n') => {
                    // Only handle 'n' for new header when not editing
                    if app.header_edit_state == HeaderEditState::Selecting {
                        debug!("Adding new header");
                        app.selected_header_index = app.get_ordered_headers().len();
                        app.header_edit_state = HeaderEditState::EditingKey;
                        app.header_edit_key = String::new();
                        app.header_edit_value = String::new();
                        app.header_key_cursor = 0;
                        app.header_value_cursor = 0;
                    } else {
                        // When editing, insert 'n' into the text
                        match app.header_edit_state {
                            HeaderEditState::EditingKey => {
                                app.header_edit_key.insert(app.header_key_cursor, 'n');
                                app.header_key_cursor += 1;
                            }
                            HeaderEditState::EditingValue => {
                                app.header_edit_value.insert(app.header_value_cursor, 'n');
                                app.header_value_cursor += 1;
                            }
                            _ => {}
                        }
                    }
                    return false;
                }
                KeyCode::Char('d') => {
                    // Only handle 'd' for delete when not editing
                    if app.header_edit_state == HeaderEditState::Selecting {
                        if let Some((key, _)) = app.get_ordered_headers().get(app.selected_header_index) {
                            debug!("Deleting header: {}", key);
                            app.headers.remove(key);
                            app.header_enabled.remove(key);
                            if app.selected_header_index > 0 {
                                app.selected_header_index -= 1;
                            }
                        }
                    } else {
                        // When editing, insert 'd' into the text
                        match app.header_edit_state {
                            HeaderEditState::EditingKey => {
                                app.header_edit_key.insert(app.header_key_cursor, 'd');
                                app.header_key_cursor += 1;
                            }
                            HeaderEditState::EditingValue => {
                                app.header_edit_value.insert(app.header_value_cursor, 'd');
                                app.header_value_cursor += 1;
                            }
                            _ => {}
                        }
                    }
                    return false;
                }
                KeyCode::Char(' ') => {
                    if let Some((key, _)) = app.get_ordered_headers().get(app.selected_header_index) {
                        let current_state = app.header_enabled.get(key).copied().unwrap_or(true);
                        app.header_enabled.insert(key.clone(), !current_state);
                        debug!("Toggled header '{}' to {}", key, !current_state);
                    }
                    return false;
                }
                KeyCode::Up => {
                    if app.selected_header_index > 0 {
                        app.selected_header_index -= 1;
                        debug!("Header up: {}", app.selected_header_index);
                    }
                    return false;
                }
                KeyCode::Down => {
                    let total_items = app.get_ordered_headers().len();
                    if app.selected_header_index < total_items - 1 {
                        app.selected_header_index += 1;
                        debug!("Header down: {}/{}", app.selected_header_index, total_items);
                    }
                    return false;
                }
                _ => {}
            }
        }

        // Rest of the normal mode handling...
        match key.code {
            KeyCode::Tab => {
                if app.header_edit_state == HeaderEditState::EditingKey {
                    app.header_edit_state = HeaderEditState::EditingValue;
                    app.header_value_cursor = app.header_edit_value.len();
                } else {
                    app.active_field = app.active_field.next();
                }
                false
            }
            KeyCode::BackTab => {
                if app.header_edit_state == HeaderEditState::EditingValue {
                    app.header_edit_state = HeaderEditState::EditingKey;
                    app.header_key_cursor = app.header_edit_key.len();
                } else {
                    app.active_field = app.active_field.previous();
                }
                false
            }
            KeyCode::Char(c) => {
                match app.header_edit_state {
                    HeaderEditState::EditingKey => {
                        app.header_edit_key.insert(app.header_key_cursor, c);
                        app.header_key_cursor += 1;
                    }
                    HeaderEditState::EditingValue => {
                        app.header_edit_value.insert(app.header_value_cursor, c);
                        app.header_value_cursor += 1;
                    }
                    _ => {}
                }
                false
            }
            KeyCode::Backspace => {
                match app.header_edit_state {
                    HeaderEditState::EditingKey => {
                        if app.header_key_cursor > 0 {
                            app.header_key_cursor -= 1;
                            app.header_edit_key.remove(app.header_key_cursor);
                        }
                    }
                    HeaderEditState::EditingValue => {
                        if app.header_value_cursor > 0 {
                            app.header_value_cursor -= 1;
                            app.header_edit_value.remove(app.header_value_cursor);
                        }
                    }
                    _ => {}
                }
                false
            }
            KeyCode::Esc => {
                if app.header_edit_state != HeaderEditState::Selecting {
                    app.header_edit_state = HeaderEditState::Selecting;
                    app.header_key_cursor = 0;
                    app.header_value_cursor = 0;
                }
                false
            }
            KeyCode::Enter => {
                if app.input_mode == InputMode::Normal && app.active_field == Field::SendButton {
                    if !app.url.is_empty() {
                        debug!("Sending request to: {}", app.url);
                        app.send_request().await;
                    } else {
                        debug!("Cannot send request: URL is empty");
                    }
                }
                match app.active_field {
                    Field::Url | Field::RequestBody => {
                        app.input_mode = InputMode::Editing(app.active_field);
                        app.cursor_position = match app.active_field {
                            Field::Url => app.url.len(),
                            Field::RequestBody => app.body.len(),
                            _ => 0
                        };
                    }
                    Field::Headers => {
                        match app.header_edit_state {
                            HeaderEditState::Selecting => {
                                if let Some((key, value)) = app.get_ordered_headers().get(app.selected_header_index) {
                                    app.header_edit_state = HeaderEditState::EditingKey;
                                    app.header_edit_key = key.clone();
                                    app.header_edit_value = value.clone();
                                    app.header_key_cursor = key.len();
                                    app.header_value_cursor = value.len();
                                }
                            }
                            HeaderEditState::EditingKey => {
                                // Remove the temporary key before moving to value
                                if let Some((key, _)) = app.get_ordered_headers().get(app.selected_header_index) {
                                    if key.starts_with("zzz_new_header_") {
                                        app.headers.remove(key);
                                    }
                                }
                                app.header_edit_state = HeaderEditState::EditingValue;
                            }
                            HeaderEditState::EditingValue => {
                                if !app.header_edit_key.is_empty() {
                                    debug!("Saving header: {} = {}", app.header_edit_key, app.header_edit_value);
                                    app.headers.insert(app.header_edit_key.clone(), app.header_edit_value.clone());
                                    app.header_enabled.insert(app.header_edit_key.clone(), true);
                                    // Keep selection on the newly added header
                                    app.selected_header_index = app.get_ordered_headers()
                                        .iter()
                                        .position(|(k, _)| k == &app.header_edit_key)
                                        .unwrap_or(0);
                                }
                                app.header_edit_state = HeaderEditState::Selecting;
                            }
                        }
                    }
                    Field::NavPanel => {
                        match app.nav_selected {
                            NavItem::Collections => {
                                app.show_collections = true;
                                app.show_history = false;
                                app.collections_focus = CollectionsFocus::List;
                                app.collection_selected_index = 0;
                                app.input_mode = InputMode::Normal;
                                app.active_field = Field::Collections;
                            }
                            NavItem::History => {
                                app.show_history = true;
                                app.show_collections = false;
                                app.history_selected_index = 0;
                                app.input_mode = InputMode::Normal;
                                app.active_field = Field::History;
                            }
                            NavItem::Quit => return true,
                            _ => {}
                        }
                    }
                    Field::History => {
                        if let Some(entry) = app.history.get_entries().get(app.history_selected_index) {
                            app.url = entry.request.url.clone();
                            app.method = HttpMethod::from_str(&entry.request.method).unwrap_or(HttpMethod::GET);
                            app.headers = entry.request.headers.clone();
                            app.body = entry.request.body.clone().unwrap_or_default();
                            app.show_history = false;
                            app.active_field = Field::Url;
                        }
                    }
                    Field::Collections => {
                        let collections = app.collection_manager.get_collections();
                        if let Some(collection) = collections.values().nth(app.collection_selected_index) {
                            app.selected_collection = Some(collection.info.name.clone());
                            app.collection_view = CollectionView::Requests;
                            app.request_selected_index = 0;
                        }
                    }
                    Field::Method => {
                        if app.show_method_selector {
                            if let Some(method) = HttpMethod::all().get(app.selector_method_index) {
                                app.method = *method;
                                app.show_method_selector = false;
                            }
                        } else {
                            app.show_method_selector = true;
                            app.selector_method_index = HttpMethod::all()
                                .iter()
                                .position(|&m| m == app.method)
                                .unwrap_or(0);
                        }
                    }
                    Field::SaveButton => {
                        app.save_dialog_visible = true;
                        app.save_dialog_selected_index = 0;
                        //////
                        app.show_collections = false;
                        //////
                    }
                    _ => {}
                }
                false
            }
            KeyCode::Up => {
                match app.active_field {
                    Field::Headers => {
                        if app.selected_header_index > 0 {
                            app.selected_header_index -= 1;
                            debug!("Header up: {}", app.selected_header_index);
                        }
                    }
                    Field::Method => {
                        if app.show_method_selector {
                            let methods = HttpMethod::all();
                            if app.selector_method_index > 0 {
                                app.selector_method_index -= 1;
                            } else {
                                app.selector_method_index = methods.len() - 1;
                            }
                        }
                    }
                    Field::NavPanel => {
                        let items = NavItem::all();
                        let current_idx = items.iter().position(|&item| item == app.nav_selected).unwrap_or(0);
                        app.nav_selected = items[if current_idx > 0 { current_idx - 1 } else { items.len() - 1 }];
                    }
                    Field::History => {
                        if app.history_selected_index > 0 {
                            app.history_selected_index -= 1;
                        }
                    }
                    Field::Collections => {
                        if app.collection_selected_index > 0 {
                            app.collection_selected_index -= 1;
                        }
                    }
                    _ => {}
                }
                false
            }
            KeyCode::Down => {
                match app.active_field {
                    Field::Headers => {
                        // Allow going one past the current headers for the placeholder
                        let total_items = app.get_ordered_headers().len() + 1;
                        if app.selected_header_index < total_items - 1 {
                            app.selected_header_index += 1;
                            debug!("Header down: {}/{}", app.selected_header_index, total_items);
                        }
                    }
                    Field::Method => {
                        if app.show_method_selector {
                            let methods = HttpMethod::all();
                            app.selector_method_index = (app.selector_method_index + 1) % methods.len();
                        }
                    }
                    Field::NavPanel => {
                        let items = NavItem::all();
                        let current_idx = items.iter().position(|&item| item == app.nav_selected).unwrap_or(0);
                        app.nav_selected = items[(current_idx + 1) % items.len()];
                    }
                    Field::History => {
                        let len = app.history.get_entries().len();
                        if len > 0 {
                            app.history_selected_index = (app.history_selected_index + 1) % len;
                        }
                    }
                    Field::Collections => {
                        let len = app.collection_manager.get_collections().len();
                        if len > 0 {
                            app.collection_selected_index = (app.collection_selected_index + 1) % len;
                        }
                    }
                    _ => {}
                }
                false
            }
            _ => false,
        }
    }

    fn handle_editing_mode(app: &mut App, field: Field, key: KeyEvent) -> bool {
        // Handle selection with Shift + Arrow keys
        if field == Field::Url && key.modifiers.contains(KeyModifiers::SHIFT) {
            match key.code {
                KeyCode::Left => {
                    if app.selection_start.is_none() {
                        app.selection_start = Some(app.cursor_position);
                    }
                    if app.cursor_position > 0 {
                        app.cursor_position -= 1;
                    }
                    return false;
                }
                KeyCode::Right => {
                    if app.selection_start.is_none() {
                        app.selection_start = Some(app.cursor_position);
                    }
                    if app.cursor_position < app.url.len() {
                        app.cursor_position += 1;
                    }
                    return false;
                }
                _ => {}
            }
        }

        // Handle copy/paste
        if field == Field::Url {
            let is_copy_paste = key.modifiers == KeyModifiers::CONTROL || 
                               key.modifiers == KeyModifiers::SUPER;
            
            if is_copy_paste {
                match key.code {
                    KeyCode::Char('c') => {
                        if let Some(start) = app.selection_start {
                            let (start, end) = if start <= app.cursor_position {
                                (start.min(app.url.len()), app.cursor_position.min(app.url.len()))
                            } else {
                                (app.cursor_position.min(app.url.len()), start.min(app.url.len()))
                            };
                            if let Ok(mut clipboard) = Clipboard::new() {
                                clipboard.set_text(&app.url[start..end]).ok();
                            }
                        }
                        return false;
                    }
                    KeyCode::Char('v') => {
                        if let Ok(mut clipboard) = Clipboard::new() {
                            if let Ok(text) = clipboard.get_text() {
                                // First, sanitize the clipboard text by replacing problematic sequences
                                let safe_text = text
                                    .replace("\nq", "q")  // Replace newline+q specifically
                                    .replace('\n', "")     // Remove other newlines
                                    .replace('\r', "")     // Remove carriage returns
                                    .replace('\t', "");    // Remove tabs
                                
                                let cursor_pos = app.cursor_position.min(app.url.len());
                                if let Some(start) = app.selection_start {
                                    let (start, end) = if start <= cursor_pos {
                                        (start.min(app.url.len()), cursor_pos)
                                    } else {
                                        (cursor_pos, start.min(app.url.len()))
                                    };
                                    app.url.replace_range(start..end, &safe_text);
                                    app.cursor_position = start + safe_text.len();
                                } else {
                                    app.url.insert_str(cursor_pos, &safe_text);
                                    app.cursor_position = cursor_pos + safe_text.len();
                                }
                                app.selection_start = None;
                            }
                        }
                        return false;
                    }
                    _ => {}
                }
            }
        }

        // Clear selection on other keys
        if key.modifiers.is_empty() || 
           key.modifiers == KeyModifiers::CONTROL || 
           key.modifiers == KeyModifiers::SUPER 
        {
            app.selection_start = None;
        }

        // Rest of the key handling...
        match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                app.cursor_position = 0;
                false
            }
            KeyCode::Enter => {
                match field {
                    Field::RequestBody => {
                        app.body.insert(app.cursor_position, '\n');
                        app.cursor_position += 1;
                        app.headers.insert(
                            "Content-Length".to_string(),
                            app.body.len().to_string()
                        );
                        false
                    }
                    _ => {
                        app.input_mode = InputMode::Normal;
                        app.cursor_position = 0;
                        false
                    }
                }
            }
            KeyCode::Tab => {
                if field == Field::RequestBody {
                    app.body.insert_str(app.cursor_position, "    ");
                    app.cursor_position += 4;
                    false
                } else {
                    app.input_mode = InputMode::Normal;
                    false
                }
            }
            KeyCode::Char(c) => {
                match field {
                    Field::Url => {
                        app.url.insert(app.cursor_position, c);
                        app.cursor_position += 1;
                    }
                    Field::RequestBody => {
                        app.body.insert(app.cursor_position, c);
                        app.cursor_position += 1;
                    }
                    _ => {}
                }
                false
            }
            KeyCode::Backspace => {
                match field {
                    Field::Url => {
                        if app.cursor_position > 0 {
                            app.cursor_position -= 1;
                            app.url.remove(app.cursor_position);
                        }
                    }
                    Field::RequestBody => {
                        if app.cursor_position > 0 {
                            app.cursor_position -= 1;
                            app.body.remove(app.cursor_position);
                        }
                    }
                    _ => {}
                }
                false
            }
            KeyCode::Right => {
                let max_pos = match field {
                    Field::Url => app.url.len(),
                    Field::RequestBody => app.body.len(),
                    _ => app.cursor_position,
                };
                app.cursor_position = (app.cursor_position + 1).min(max_pos);
                false
            }
            _ => false,
        }
    }
} 