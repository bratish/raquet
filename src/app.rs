use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use reqwest;
use std::time::{Instant, Duration};
use crate::config::AppConfig;
use crate::history::{History, ResponseData};
use ratatui::style::Color;
use crate::collections::{CollectionManager, Collection, CollectionItem, Request, CollectionInfo};
use chrono::Utc;
use itertools::Itertools;
use reqwest::Url;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Editing(Field),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Field {
    Url,
    Method,
    Headers,
    Body,
    SendButton,
    SaveButton,
    Response,
    NavPanel,
}

impl Field {
    fn next(&self) -> Self {
        match self {
            Field::NavPanel => Field::Url,
            Field::Url => Field::Method,
            Field::Method => Field::Headers,
            Field::Headers => Field::Body,
            Field::Body => Field::SendButton,
            Field::SendButton => Field::SaveButton,
            Field::SaveButton => Field::Response,
            Field::Response => Field::NavPanel,
        }
    }

    fn previous(&self) -> Self {
        match self {
            Field::NavPanel => Field::Response,
            Field::Url => Field::NavPanel,
            Field::Method => Field::Url,
            Field::Headers => Field::Method,
            Field::Body => Field::Headers,
            Field::SendButton => Field::Body,
            Field::SaveButton => Field::SendButton,
            Field::Response => Field::SaveButton,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
        }
    }

    pub fn all() -> Vec<HttpMethod> {
        vec![
            HttpMethod::GET,
            HttpMethod::POST,
            HttpMethod::PUT,
            HttpMethod::DELETE,
            HttpMethod::PATCH,
        ]
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "GET" => Some(HttpMethod::GET),
            "POST" => Some(HttpMethod::POST),
            "PUT" => Some(HttpMethod::PUT),
            "DELETE" => Some(HttpMethod::DELETE),
            "PATCH" => Some(HttpMethod::PATCH),
            _ => None,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            HttpMethod::GET => Color::Green,
            HttpMethod::POST => Color::Cyan,
            HttpMethod::PUT => Color::Magenta,
            HttpMethod::PATCH => Color::Magenta,
            HttpMethod::DELETE => Color::Yellow,
        }
    }
}

#[derive(Clone)]
pub struct ResponseMetadata {
    pub status: Option<u16>,
    pub status_text: Option<String>,
    pub time_ms: u128,
    pub size_bytes: usize,
    pub response_headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NavItem {
    Collections,
    Environments,
    History,
    Quit,
}

impl NavItem {
    pub fn all() -> Vec<NavItem> {
        vec![
            NavItem::Collections,
            NavItem::Environments,
            NavItem::History,
            NavItem::Quit,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            NavItem::Collections => "Collections",
            NavItem::Environments => "Environments",
            NavItem::History => "History",
            NavItem::Quit => "Quit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeaderEditState {
    Viewing,           // Normal state, just viewing headers
    Selecting,         // Moving up/down through headers
    EditingKeyInPlace, // Editing key part of the selected header
    EditingValueInPlace, // Editing value part of the selected header
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollectionView {
    List,     // Showing list of collections
    Requests, // Showing requests in a collection
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollectionDialogState {
    Hidden,
    NewCollection,
    SaveRequest,
    SelectCollection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DialogFocus {
    Name,
    Description,
    Buttons,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollectionsFocus {
    List,
    NewButton,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DialogButtonFocus {
    Save,
    Cancel,
}

#[derive(Clone)]
pub struct App {
    pub url: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub input_mode: InputMode,
    pub cursor_position: usize,
    pub response: Option<String>,
    pub response_metadata: Option<ResponseMetadata>,
    pub active_field: Field,
    pub response_scroll: u16,
    pub config: AppConfig,
    pub history: History,
    pub nav_selected: NavItem,
    pub show_history: bool,
    pub history_selected_index: usize,
    pub header_edit_state: HeaderEditState,
    pub selected_header_index: usize,
    pub header_edit_key: String,
    pub header_edit_value: String,
    pub header_enabled: HashMap<String, bool>,  // Track which headers are enabled
    pub collection_manager: CollectionManager,
    pub selected_collection: Option<String>,
    pub selected_request: Option<String>,
    pub show_collections: bool,
    pub collection_view: CollectionView,
    pub collection_selected_index: usize,
    pub request_selected_index: usize,
    pub save_request_name: String,
    pub is_saving_request: bool,
    pub collection_dialog: CollectionDialogState,
    pub new_collection_name: String,
    pub new_collection_description: String,
    pub dialog_focus: DialogFocus,
    pub description_scroll: u16,
    pub collections_focus: CollectionsFocus,
    pub dialog_button_focus: DialogButtonFocus,
    pub show_collection_selector: bool,
    pub selector_collection_index: usize,
    pub show_method_selector: bool,
    pub selector_method_index: usize,
    pub headers_scroll: u16,
}

impl App {
    pub fn new() -> Self {
        let config = AppConfig::load().unwrap_or_else(|_| AppConfig::default());
        let history = History::new(config.history_size).unwrap_or_else(|_| {
            History::new(100).expect("Failed to create history with default size")
        });
        
        let collection_manager = CollectionManager::new()
            .expect("Failed to initialize collection manager");
        
        let mut app = Self {
            url: String::new(),
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: String::new(),
            input_mode: InputMode::Normal,
            cursor_position: 0,
            response: None,
            response_metadata: None,
            active_field: Field::NavPanel,
            response_scroll: 0,
            config,
            history,
            nav_selected: NavItem::Collections,
            show_history: false,
            history_selected_index: 0,
            header_edit_state: HeaderEditState::Viewing,
            selected_header_index: 0,
            header_edit_key: String::new(),
            header_edit_value: String::new(),
            header_enabled: HashMap::new(),
            collection_manager,
            selected_collection: None,
            selected_request: None,
            show_collections: false,
            collection_view: CollectionView::List,
            collection_selected_index: 0,
            request_selected_index: 0,
            save_request_name: String::new(),
            is_saving_request: false,
            collection_dialog: CollectionDialogState::Hidden,
            new_collection_name: String::new(),
            new_collection_description: String::new(),
            dialog_focus: DialogFocus::Name,
            description_scroll: 0,
            collections_focus: CollectionsFocus::List,
            dialog_button_focus: DialogButtonFocus::Save,
            show_collection_selector: false,
            selector_collection_index: 0,
            show_method_selector: false,
            selector_method_index: 0,
            headers_scroll: 0,
        };

        // Apply default headers from config and mark them as enabled
        app.headers = app.config.default_headers.clone();
        for key in app.headers.keys() {
            app.header_enabled.insert(key.clone(), true);
        }

        // Apply default URL if set
        if !app.config.default_url.is_empty() {
            app.url = app.config.default_url.clone();
        }

        app
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.show_history {
            match key.code {
                KeyCode::Esc => {
                    self.show_history = false;
                    self.active_field = Field::NavPanel;
                }
                KeyCode::Enter => self.select_history_item(),
                KeyCode::Up => {
                    if self.history_selected_index > 0 {
                        self.history_selected_index -= 1;
                    }
                }
                KeyCode::Down => {
                    if self.history_selected_index < self.history.get_entries().len().saturating_sub(1) {
                        self.history_selected_index += 1;
                    }
                }
                _ => {} // Handle all other keys
            }
            return false;
        }

        if self.show_collection_selector {
            match key.code {
                KeyCode::Esc => {
                    self.show_collection_selector = false;
                }
                KeyCode::Enter => {
                    // Collect collections and get the name first
                    let collection_name = self.collection_manager
                        .get_collections()
                        .values()
                        .collect::<Vec<_>>()
                        .into_iter()
                        .sorted_by(|a, b| b.info.created_at.cmp(&a.info.created_at))
                        .nth(self.selector_collection_index)
                        .map(|collection| collection.info.name.clone());

                    // Then use the name to save
                    if let Some(name) = collection_name {
                        if let Err(e) = self.save_to_collection(&name) {
                            eprintln!("Error saving request: {}", e);
                        }
                    }
                    self.show_collection_selector = false;
                }
                KeyCode::Up => {
                    if self.selector_collection_index > 0 {
                        self.selector_collection_index -= 1;
                    }
                }
                KeyCode::Down => {
                    let max_index = self.collection_manager.get_collections().len().saturating_sub(1);
                    if self.selector_collection_index < max_index {
                        self.selector_collection_index += 1;
                    }
                }
                _ => {}
            }
            return false;
        }

        if self.show_method_selector {
            match key.code {
                KeyCode::Esc => {
                    self.show_method_selector = false;
                }
                KeyCode::Enter => {
                    let methods = HttpMethod::all();
                    if let Some(method) = methods.get(self.selector_method_index) {
                        self.method = *method;
                    }
                    self.show_method_selector = false;
                }
                KeyCode::Up => {
                    if self.selector_method_index > 0 {
                        self.selector_method_index -= 1;
                    }
                }
                KeyCode::Down => {
                    let max_index = HttpMethod::all().len() - 1;
                    if self.selector_method_index < max_index {
                        self.selector_method_index += 1;
                    }
                }
                _ => {}
            }
            return false;
        }

        if self.show_collections {
            match self.collection_dialog {
                CollectionDialogState::Hidden => {
                    match key.code {
                        KeyCode::Tab => {
                            // Toggle between list and button
                            self.collections_focus = match self.collections_focus {
                                CollectionsFocus::List => CollectionsFocus::NewButton,
                                CollectionsFocus::NewButton => CollectionsFocus::List,
                            };
                        }
                        KeyCode::Enter => {
                            match self.collection_view {
                                CollectionView::List => {
                                    match self.collections_focus {
                                        CollectionsFocus::NewButton => {
                                            self.collection_dialog = CollectionDialogState::NewCollection;
                                        }
                                        CollectionsFocus::List => {
                                            let sorted_collections: Vec<_> = self.collection_manager
                                                .get_collections()
                                                .values()
                                                .collect::<Vec<_>>()
                                                .into_iter()
                                                .sorted_by(|a, b| b.info.created_at.cmp(&a.info.created_at))
                                                .collect();

                                            if let Some(collection) = sorted_collections.get(self.collection_selected_index) {
                                                self.select_collection(collection.info.name.clone());
                                                self.collection_view = CollectionView::Requests;
                                            }
                                        }
                                    }
                                }
                                CollectionView::Requests => {
                                    if let Some(name) = self.get_selected_request_name() {
                                        self.selected_request = Some(name);
                                        if self.load_selected_request() {
                                            self.show_collections = false;
                                            self.reset_collections_state();
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char('n') => {
                            self.collection_dialog = CollectionDialogState::NewCollection;
                        }
                        KeyCode::Down => {
                            match self.collection_view {
                                CollectionView::List => {
                                    let max = self.collection_manager.get_collections().len();
                                    if self.collection_selected_index < max - 1 {
                                        self.collection_selected_index += 1;
                                    }
                                }
                                CollectionView::Requests => {
                                    if self.request_selected_index < self.get_selected_collection().unwrap().item.len().saturating_sub(1) {
                                        self.request_selected_index += 1;
                                    }
                                }
                            }
                        }
                        KeyCode::Up => {
                            match self.collection_view {
                                CollectionView::List => {
                                    if self.collection_selected_index > 0 {
                                        self.collection_selected_index -= 1;
                                    }
                                }
                                CollectionView::Requests => {
                                    if self.request_selected_index > 0 {
                                        self.request_selected_index -= 1;
                                    }
                                }
                            }
                        }
                        KeyCode::Esc => {
                            match self.collection_view {
                                CollectionView::List => {
                                    self.show_collections = false;
                                    self.reset_collections_state();
                                }
                                CollectionView::Requests => {
                                    self.collection_view = CollectionView::List;
                                    self.selected_collection = None;
                                    self.request_selected_index = 0;
                                }
                            }
                        }
                        _ => {} // Handle all other keys
                    }
                }
                CollectionDialogState::NewCollection => {
                    match key.code {
                        KeyCode::Tab => {
                            // Cycle through: Name -> Description -> Save -> Cancel -> Name
                            match self.dialog_focus {
                                DialogFocus::Name => self.dialog_focus = DialogFocus::Description,
                                DialogFocus::Description => {
                                    self.dialog_focus = DialogFocus::Buttons;
                                    self.dialog_button_focus = DialogButtonFocus::Save;
                                }
                                DialogFocus::Buttons => {
                                    match self.dialog_button_focus {
                                        DialogButtonFocus::Save => {
                                            self.dialog_button_focus = DialogButtonFocus::Cancel;
                                        }
                                        DialogButtonFocus::Cancel => {
                                            self.dialog_focus = DialogFocus::Name;
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Enter => {
                            match self.dialog_focus {
                                DialogFocus::Name => {
                                    if !self.new_collection_name.is_empty() {
                                        self.dialog_focus = DialogFocus::Description;
                                    }
                                }
                                DialogFocus::Description => {
                                    self.new_collection_description.push('\n');
                                }
                                DialogFocus::Buttons => {
                                    match self.dialog_button_focus {
                                        DialogButtonFocus::Save => {
                                            if !self.new_collection_name.is_empty() {
                                                let collection = Collection {
                                                    info: CollectionInfo {
                                                        name: self.new_collection_name.clone(),
                                                        description: self.new_collection_description.clone(),
                                                        created_at: Utc::now(),
                                                    },
                                                    item: Vec::new(),
                                                };
                                                if let Err(e) = self.collection_manager.save_collection(&collection) {
                                                    eprintln!("Error saving collection: {}", e);
                                                } else {
                                                    if let Err(e) = self.collection_manager.reload_collections() {
                                                        eprintln!("Error reloading collections: {}", e);
                                                    }
                                                    self.collection_selected_index = 1;
                                                }
                                                self.collection_dialog = CollectionDialogState::Hidden;
                                                self.new_collection_name.clear();
                                                self.new_collection_description.clear();
                                            }
                                        }
                                        DialogButtonFocus::Cancel => {
                                            self.collection_dialog = CollectionDialogState::Hidden;
                                            self.new_collection_name.clear();
                                            self.new_collection_description.clear();
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if !self.new_collection_name.is_empty() {
                                let collection = Collection {
                                    info: CollectionInfo {
                                        name: self.new_collection_name.clone(),
                                        description: self.new_collection_description.clone(),
                                        created_at: Utc::now(),
                                    },
                                    item: Vec::new(),
                                };
                                if let Err(e) = self.collection_manager.save_collection(&collection) {
                                    eprintln!("Error saving collection: {}", e);
                                } else {
                                    if let Err(e) = self.collection_manager.reload_collections() {
                                        eprintln!("Error reloading collections: {}", e);
                                    }
                                    self.collection_selected_index = 1;
                                }
                                self.collection_dialog = CollectionDialogState::Hidden;
                                self.new_collection_name.clear();
                                self.new_collection_description.clear();
                            }
                        }
                        KeyCode::Char(c) => self.handle_char_input(c),
                        KeyCode::Backspace => self.handle_backspace(),
                        _ => {} // Handle all other keys
                    }
                }
                CollectionDialogState::SaveRequest => {
                    match key.code {
                        KeyCode::Esc => {
                            self.collection_dialog = CollectionDialogState::Hidden;
                            self.save_request_name.clear();
                        }
                        KeyCode::Enter => {
                            if !self.save_request_name.is_empty() {
                                if let Some(collection_name) = self.selected_collection.clone() {
                                    if let Err(e) = self.save_current_request(&collection_name) {
                                        eprintln!("Error saving request: {}", e);
                                    }
                                }
                                self.collection_dialog = CollectionDialogState::Hidden;
                            }
                        }
                        KeyCode::Char(c) => {
                            self.save_request_name.push(c);
                        }
                        KeyCode::Backspace => {
                            self.save_request_name.pop();
                        }
                        _ => {} // Handle all other keys
                    }
                }
                CollectionDialogState::SelectCollection => {
                    match key.code {
                        KeyCode::Enter => {
                            if let Some(collection_name) = self.selected_collection.clone() {
                                if let Err(e) = self.save_to_collection(&collection_name) {
                                    eprintln!("Error saving request: {}", e);
                                }
                                self.collection_dialog = CollectionDialogState::Hidden;
                            }
                        }
                        KeyCode::Esc => {
                            self.collection_dialog = CollectionDialogState::Hidden;
                        }
                        _ => {} // Handle all other keys
                    }
                }
            }
        } else {
            match self.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Tab => {
                        self.active_field = self.active_field.next();
                        self.response_scroll = 0;
                    }
                    KeyCode::BackTab => {
                        self.active_field = self.active_field.previous();
                        self.response_scroll = 0;
                    }
                    KeyCode::Up => {
                        if self.active_field == Field::NavPanel {
                            self.nav_selected = match self.nav_selected {
                                NavItem::Collections => NavItem::Quit,
                                NavItem::Environments => NavItem::Collections,
                                NavItem::History => NavItem::Environments,
                                NavItem::Quit => NavItem::History,
                            };
                        } else if self.active_field == Field::Response && self.response_scroll > 0 {
                            self.response_scroll -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.active_field == Field::NavPanel {
                            self.nav_selected = match self.nav_selected {
                                NavItem::Collections => NavItem::Environments,
                                NavItem::Environments => NavItem::History,
                                NavItem::History => NavItem::Quit,
                                NavItem::Quit => NavItem::Collections,
                            };
                        } else if self.active_field == Field::Response {
                            self.response_scroll += 1;
                        }
                    }
                    KeyCode::PageUp => {
                        if self.active_field == Field::Response && self.response_scroll > 0 {
                            self.response_scroll = self.response_scroll.saturating_sub(10);
                        }
                    }
                    KeyCode::PageDown => {
                        if self.active_field == Field::Response {
                            self.response_scroll += 10;
                        }
                    }
                    KeyCode::Enter => {
                        if self.active_field == Field::NavPanel {
                            return self.handle_nav_selection();
                        } else if self.active_field == Field::SendButton {
                            self.input_mode = InputMode::Normal;
                            self.send_request();
                        } else if self.active_field == Field::SaveButton {
                            self.show_collection_selector = true;
                            self.selector_collection_index = 0;
                        } else if self.active_field == Field::Method {
                            self.show_method_selector = true;
                            self.selector_method_index = HttpMethod::all()
                                .iter()
                                .position(|&m| m == self.method)
                                .unwrap_or(0);
                        } else if self.active_field == Field::Headers {
                            // Go directly to selecting mode when entering headers
                            self.input_mode = InputMode::Editing(Field::Headers);
                            self.header_edit_state = HeaderEditState::Selecting;
                            self.selected_header_index = 0;
                        } else {
                            if self.active_field != Field::SendButton {
                                self.input_mode = InputMode::Editing(self.active_field);
                            }
                        }
                    }
                    _ => {} // Handle all other keys
                },
                InputMode::Editing(field) => {
                    match field {
                        Field::Url => self.handle_url_input(key),
                        Field::Method => self.handle_method_input(key),
                        Field::Headers => self.handle_headers_input(key),
                        Field::Body => self.handle_body_input(key),
                        Field::SendButton | Field::Response | Field::NavPanel | Field::SaveButton => {}
                    }
                }
            }
        }
        false
    }

    fn handle_url_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                // Exit URL editing mode and return to normal mode
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                self.url.insert(self.cursor_position, c);
                self.cursor_position += 1;
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.url.remove(self.cursor_position);
                }
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_position < self.url.len() {
                    self.cursor_position += 1;
                }
            }
            _ => {} // Catch all other keys
        }
    }

    fn handle_method_input(&mut self, key: KeyEvent) {
        let methods = HttpMethod::all();
        let current_idx = methods.iter().position(|&m| m == self.method).unwrap_or(0);
        
        match key.code {
            KeyCode::Up | KeyCode::Left => {
                let new_idx = if current_idx > 0 {
                    current_idx - 1
                } else {
                    methods.len() - 1
                };
                self.method = methods[new_idx];
            }
            KeyCode::Down | KeyCode::Right => {
                let new_idx = (current_idx + 1) % methods.len();
                self.method = methods[new_idx];
            }
            KeyCode::Char(c) => {
                let c = c.to_ascii_uppercase();
                if let Some(method) = methods.iter().find(|m| m.as_str().starts_with(c)) {
                    self.method = *method;
                }
            }
            _ => {} // Catch all other keys
        }
    }

    fn handle_headers_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                match self.header_edit_state {
                    HeaderEditState::EditingKeyInPlace | HeaderEditState::EditingValueInPlace => {
                        self.header_edit_state = HeaderEditState::Selecting;
                    }
                    HeaderEditState::Selecting => {
                        self.header_edit_state = HeaderEditState::Viewing;
                        self.input_mode = InputMode::Normal;
                    }
                    _ => {
                        self.header_edit_state = HeaderEditState::Viewing;
                        self.input_mode = InputMode::Normal;
                    }
                }
            }
            KeyCode::Enter => {
                match self.header_edit_state {
                    HeaderEditState::Viewing => {
                        self.header_edit_state = HeaderEditState::Selecting;
                    }
                    HeaderEditState::Selecting => {
                        // Start editing the selected header
                        if self.selected_header_index < self.headers.len() {
                            // Editing existing header
                            if let Some((key, value)) = self.headers.iter().nth(self.selected_header_index) {
                                self.header_edit_key = key.clone();
                                self.header_edit_value = value.clone();
                            }
                        } else {
                            // New header
                            self.header_edit_key = String::new();
                            self.header_edit_value = String::new();
                        }
                        self.header_edit_state = HeaderEditState::EditingKeyInPlace;
                    }
                    HeaderEditState::EditingKeyInPlace => {
                        // Save key and move to value editing
                        self.header_edit_state = HeaderEditState::EditingValueInPlace;
                    }
                    HeaderEditState::EditingValueInPlace => {
                        // Save both key and value
                        if !self.header_edit_key.is_empty() {
                            // Get and clone the old key first
                            let old_key = self.headers
                                .iter()
                                .nth(self.selected_header_index)
                                .map(|(k, _)| k.clone());

                            // Then use the cloned key for removal
                            if let Some(key_to_remove) = old_key {
                                self.headers.remove(&key_to_remove);
                                self.header_enabled.remove(&key_to_remove);
                            }

                            // Insert new key-value pair
                            self.headers.insert(
                                self.header_edit_key.clone(),
                                self.header_edit_value.clone()
                            );
                            // Only enable the header if it's not the empty placeholder
                            if self.header_edit_key != "<key>" {
                                self.header_enabled.insert(self.header_edit_key.clone(), true);
                            }
                        }
                        self.header_edit_state = HeaderEditState::Selecting;
                    }
                }
            }
            KeyCode::Up => {
                if self.header_edit_state == HeaderEditState::Selecting {
                    if self.selected_header_index > 0 {
                        self.selected_header_index -= 1;
                        // Adjust scroll if needed
                        if self.headers_scroll > self.selected_header_index as u16 {
                            self.headers_scroll = self.selected_header_index as u16;
                        }
                    }
                }
            }
            KeyCode::Down => {
                if self.header_edit_state == HeaderEditState::Selecting {
                    let max = self.headers.len();
                    if self.selected_header_index <= max {
                        self.selected_header_index += 1;
                        // Adjust scroll to keep selected item in view
                        let viewport_height = 6; // Approximate height of headers section
                        if (self.selected_header_index as u16) >= self.headers_scroll + viewport_height {
                            self.headers_scroll = (self.selected_header_index as u16).saturating_sub(viewport_height - 1);
                        }
                    }
                }
            }
            KeyCode::Tab => {
                if self.header_edit_state == HeaderEditState::EditingKeyInPlace {
                    self.header_edit_state = HeaderEditState::EditingValueInPlace;
                }
            }
            KeyCode::Char(c) => {
                match self.header_edit_state {
                    HeaderEditState::EditingKeyInPlace => {
                        self.header_edit_key.push(c);
                    }
                    HeaderEditState::EditingValueInPlace => {
                        self.header_edit_value.push(c);
                    }
                    _ => {}
                }
            }
            KeyCode::Backspace => {
                match self.header_edit_state {
                    HeaderEditState::EditingKeyInPlace => {
                        self.header_edit_key.pop();
                    }
                    HeaderEditState::EditingValueInPlace => {
                        self.header_edit_value.pop();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn handle_body_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                // Exit body editing mode and return to normal mode
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => {
                // Add newline character to the body
                self.body.push('\n');
            }
            KeyCode::Char(c) => {
                self.body.push(c);
            }
            KeyCode::Backspace => {
                self.body.pop();
            }
            _ => {} // Catch all other keys
        }
    }

    pub async fn send_request(&mut self) {
        // Before building the request, update dynamic headers
        if let Some(enabled) = self.header_enabled.get("Content-Length") {
            if *enabled {
                self.headers.insert(
                    "Content-Length".to_string(), 
                    self.body.len().to_string()
                );
            }
        }

        if let Some(enabled) = self.header_enabled.get("Host") {
            if *enabled {
                if let Ok(url) = Url::parse(&self.url) {
                    if let Some(host) = url.host_str() {
                        self.headers.insert(
                            "Host".to_string(),
                            host.to_string()
                        );
                    }
                }
            }
        }

        // Generate a new Random-Token for each request
        if let Some(enabled) = self.header_enabled.get("Random-Token") {
            if *enabled {
                self.headers.insert(
                    "Random-Token".to_string(),
                    Uuid::new_v4().to_string()
                );
            }
        }

        let start_time = Instant::now();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(self.config.timeout_seconds))
            .build()
            .unwrap_or_default();
        
        let request = match self.method {
            HttpMethod::GET => client.get(&self.url),
            HttpMethod::POST => client.post(&self.url),
            HttpMethod::PUT => client.put(&self.url),
            HttpMethod::DELETE => client.delete(&self.url),
            HttpMethod::PATCH => client.patch(&self.url),
        };

        let mut request = request;
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        if self.method != HttpMethod::GET && !self.body.is_empty() {
            request = request.body(self.body.clone());
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status();
                let status_code = status.as_u16();
                let status_text = status.canonical_reason().map(String::from);
                
                let headers = response
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect::<HashMap<_, _>>();
                
                match response.text().await {
                    Ok(body) => {
                        let elapsed = start_time.elapsed();
                        let response_data = ResponseData {
                            status: Some(status_code),
                            status_text: status_text.clone(),
                            headers: headers.clone(),
                            body: body.clone(),
                            time_ms: elapsed.as_millis(),
                            size_bytes: body.len(),
                        };

                        let _ = self.history.add_entry(
                            self.url.clone(),
                            self.method,
                            self.headers.clone(),
                            if self.body.is_empty() { None } else { Some(self.body.clone()) },
                            Some(response_data),
                        );

                        self.response_metadata = Some(ResponseMetadata {
                            status: Some(status_code),
                            status_text,
                            time_ms: elapsed.as_millis(),
                            size_bytes: body.len(),
                            response_headers: headers.clone(),
                        });

                        self.response = Some(body);
                    }
                    Err(e) => {
                        self.response = Some(format!("Error reading response body: {}", e));
                        self.response_metadata = None;
                    }
                }
            }
            Err(e) => {
                self.response = Some(format!("Error sending request: {}", e));
                self.response_metadata = None;
            }
        }
    }

    pub fn select_history_item(&mut self) {
        if let Some(entry) = self.history.get_entries().get(self.history_selected_index) {
            self.url = entry.request.url.clone();
            self.method = HttpMethod::from_str(&entry.request.method).unwrap_or(HttpMethod::GET);
            self.headers = entry.request.headers.clone();
            self.body = entry.request.body.clone().unwrap_or_default();
            self.show_history = false;
        }
    }

    pub fn select_collection(&mut self, name: String) {
        self.selected_collection = Some(name);
        self.selected_request = None;
    }

    pub fn load_selected_request(&mut self) -> bool {
        if let (Some(coll_name), Some(req_name)) = (&self.selected_collection, &self.selected_request) {
            if let Some(collection) = self.collection_manager.get_collection(coll_name) {
                if let Some(request) = find_request_by_name(collection, req_name) {
                    self.method = HttpMethod::from_str(&request.method).unwrap_or(HttpMethod::GET);
                    self.url = request.url.clone();
                    self.headers = request.header.iter()
                        .map(|h| (h.key.clone(), h.value.clone()))
                        .collect();
                    if !request.body.raw.is_empty() {
                        self.body = request.body.raw.clone();
                    }
                    return true;
                }
            }
        }
        false
    }

    pub fn save_current_request(&mut self, collection_name: &str) -> anyhow::Result<()> {
        let headers: Vec<(String, String)> = self.headers.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        self.collection_manager.save_request(
            collection_name,
            &self.save_request_name,
            self.method.as_str(),
            &self.url,
            &headers,
            &self.body,
        )?;
        
        self.is_saving_request = false;
        self.save_request_name.clear();
        Ok(())
    }

    #[allow(dead_code)]
    fn get_selected_collection_name(&self) -> Option<String> {
        if self.collection_selected_index == 0 {
            None
        } else {
            self.collection_manager
                .get_collections()
                .keys()
                .nth(self.collection_selected_index - 1)
                .cloned()
        }
    }

    fn get_selected_request_name(&self) -> Option<String> {
        self.selected_collection.as_ref().and_then(|coll_name| {
            self.collection_manager
                .get_collection(coll_name)
                .and_then(|collection| {
                    get_request_names(&collection.item)
                        .nth(self.request_selected_index)
                })
        })
    }

    fn get_selected_collection(&self) -> Option<&Collection> {
        self.selected_collection
            .as_ref()
            .and_then(|name| self.collection_manager.get_collection(name))
    }

    fn reset_collections_state(&mut self) {
        self.collection_view = CollectionView::List;
        self.collection_selected_index = 0;
        self.request_selected_index = 0;
        self.selected_collection = None;
        self.selected_request = None;
    }

    fn handle_char_input(&mut self, c: char) {
        match self.dialog_focus {
            DialogFocus::Name => self.new_collection_name.push(c),
            DialogFocus::Description => self.new_collection_description.push(c),
            DialogFocus::Buttons => {} // No character input when buttons are focused
        }
    }

    fn handle_backspace(&mut self) {
        match self.dialog_focus {
            DialogFocus::Name => { self.new_collection_name.pop(); }
            DialogFocus::Description => { self.new_collection_description.pop(); }
            DialogFocus::Buttons => {} // No backspace when buttons are focused
        }
    }

    fn handle_nav_selection(&mut self) -> bool {
        match self.nav_selected {
            NavItem::Collections => {
                self.show_collections = true;
                false
            }
            NavItem::Environments => false, // TODO: Implement environments handling
            NavItem::History => {
                self.show_history = true;
                self.history_selected_index = 0;  // Reset selection to top
                // Stay in Normal mode but focus on history
                self.active_field = Field::Response;
                false
            }
            NavItem::Quit => true
        }
    }

    pub fn is_request_in_collection(&self) -> bool {
        if let Some(collection_name) = &self.selected_collection {
            if let Some(collection) = self.collection_manager.get_collection(collection_name) {
                for item in &collection.item {
                    if let CollectionItem::Request(req) = item {
                        if req.request.method == self.method.as_str() &&
                           req.request.url == self.url {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn save_to_collection(&mut self, collection_name: &str) -> anyhow::Result<()> {
        let request_name = format!("{} {}", self.method.as_str(), self.url);
        let headers: Vec<(String, String)> = self.headers.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        self.collection_manager.save_request(
            collection_name,
            &request_name,
            self.method.as_str(),
            &self.url,
            &headers,
            &self.body,
        )
    }

    pub fn get_ordered_headers(&self) -> Vec<(String, String)> {
        let mut headers: Vec<_> = self.headers.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        headers.sort_by(|a, b| a.0.cmp(&b.0));  // Sort by key
        headers
    }
}

fn find_request_by_name<'a>(collection: &'a Collection, name: &str) -> Option<&'a Request> {
    fn search_items<'a>(items: &'a [CollectionItem], name: &str) -> Option<&'a Request> {
        for item in items {
            match item {
                CollectionItem::Request(req) if req.name == name => {
                    return Some(&req.request)
                }
                CollectionItem::Folder(folder) => {
                    if let Some(req) = search_items(&folder.item, name) {
                        return Some(req);
                    }
                }
                _ => {}
            }
        }
        None
    }
    search_items(&collection.item, name)
}

fn get_request_names(items: &[CollectionItem]) -> impl Iterator<Item = String> + '_ {
    items.iter().filter_map(|item| match item {
        CollectionItem::Request(req) => Some(req.name.clone()),
        CollectionItem::Folder(_folder) => None,
    })
}