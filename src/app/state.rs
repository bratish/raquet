use std::collections::HashMap;
use ratatui::style::Color;
use crossterm::event::KeyEvent;
use log::debug;
use crate::data::{AppConfig, History, CollectionManager};
use crate::models::{ResponseMetadata, CollectionItem};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Editing(Field),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Field {
    Url,            // 1. URL box (default)
    SendButton,     // 2. [Go] button
    SaveButton,     // 3. [+] button
    Headers,        // 4. Headers box
    RequestBody,    // 5. Request body box
    Method,         // 6. Method box
    NavPanel,       // 7. Left nav
    Collections,
    History,
}

impl Field {
    pub fn next(&self) -> Self {
        match self {
            Field::Url => Field::SendButton,
            Field::SendButton => Field::SaveButton,
            Field::SaveButton => Field::Headers,
            Field::Headers => Field::RequestBody,
            Field::RequestBody => Field::Method,
            Field::Method => Field::NavPanel,
            Field::NavPanel => Field::Url,
            Field::Collections => Field::Collections,
            Field::History => Field::History,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Field::Url => Field::NavPanel,
            Field::SendButton => Field::Url,
            Field::SaveButton => Field::SendButton,
            Field::Headers => Field::SaveButton,
            Field::RequestBody => Field::Headers,
            Field::Method => Field::RequestBody,
            Field::NavPanel => Field::Method,
            Field::Collections => Field::Collections,
            Field::History => Field::History,
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
    Selecting,
    EditingKey,
    EditingValue,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollectionView {
    List,
    Requests,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollectionsFocus {
    List,
    NewButton,
}

#[derive(Clone)]
pub struct App {
    // Core state
    pub url: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub input_mode: InputMode,
    pub active_field: Field,
    pub cursor_position: usize,

    // Response state
    pub response: Option<String>,
    pub response_metadata: Option<ResponseMetadata>,
    pub response_scroll: u16,

    // UI state
    pub show_history: bool,
    pub history_selected_index: usize,
    pub show_method_selector: bool,
    pub selector_method_index: usize,
    pub nav_selected: NavItem,
    pub header_edit_state: HeaderEditState,
    pub selected_header_index: usize,
    pub header_edit_key: String,
    pub header_edit_value: String,
    pub header_enabled: HashMap<String, bool>,
    pub headers_scroll: u16,

    // Data managers
    pub config: AppConfig,
    pub history: History,
    pub collection_manager: CollectionManager,

    // Collection state
    pub show_collections: bool,
    pub collections_focus: CollectionsFocus,
    pub show_collection_selector: bool,
    pub selector_collection_index: usize,
    pub collection_view: CollectionView,
    pub selected_collection: Option<String>,
    pub selected_request: Option<String>,
    pub collection_selected_index: usize,
    pub request_selected_index: usize,
    pub header_key_cursor: usize,
    pub header_value_cursor: usize,

    pub save_dialog_visible: bool,
    pub save_dialog_selected_index: usize,

    pub selection_start: Option<usize>,
}

impl App {
    pub fn new() -> Self {
        let config = AppConfig::load().unwrap_or_default();
        debug!("Loaded config with headers: {:?}", config.app.default_headers);
        
        let mut app = Self {
            config,
            url: String::new(),
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: String::new(),
            input_mode: InputMode::Normal,
            active_field: Field::Url,
            cursor_position: 0,
            response: None,
            response_metadata: None,
            response_scroll: 0,
            show_history: false,
            history_selected_index: 0,
            show_method_selector: false,
            selector_method_index: 0,
            nav_selected: NavItem::Collections,
            header_edit_state: HeaderEditState::Selecting,
            selected_header_index: 0,
            header_edit_key: String::new(),
            header_edit_value: String::new(),
            header_enabled: HashMap::new(),
            headers_scroll: 0,
            history: History::new(100).unwrap(),
            collection_manager: CollectionManager::new(),
            show_collections: false,
            collections_focus: CollectionsFocus::List,
            show_collection_selector: false,
            selector_collection_index: 0,
            collection_view: CollectionView::List,
            selected_collection: None,
            selected_request: None,
            collection_selected_index: 0,
            request_selected_index: 0,
            header_key_cursor: 0,
            header_value_cursor: 0,
            save_dialog_visible: false,
            save_dialog_selected_index: 0,
            selection_start: None,
        };
        
        // Initialize headers from config
        debug!("Initializing headers from config...");
        for (key, value) in &app.config.app.default_headers {
            debug!("Adding header: {} = {}", key, value);
            app.headers.insert(key.clone(), value.clone());
            app.header_enabled.insert(key.clone(), true);
        }
        debug!("Final headers state: {:?}", app.headers);
        
        app
    }

    pub fn get_ordered_headers(&self) -> Vec<(String, String)> {
        let mut headers: Vec<_> = self.headers.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        headers.sort_by(|a, b| a.0.cmp(&b.0));
        headers
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        crate::app::input::InputHandler::handle_key(self, key).await
    }

    pub async fn send_request(&mut self) {
        crate::app::actions::RequestHandler::send_request(self).await
    }

    pub fn is_request_in_collection(&self) -> bool {
        if let Some(collection_name) = &self.selected_collection {
            if let Some(collection) = self.collection_manager.get_collection(collection_name) {
                for item in &collection.requests {
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

    pub fn update_request_body(&mut self, new_body: String) {
        self.body = new_body;
        // Update Content-Length when body changes
        self.headers.insert(
            "Content-Length".to_string(),
            self.body.len().to_string()
        );
    }
} 