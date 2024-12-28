use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    style::{Color, Style},
};
use crate::app::{App, HttpMethod};

pub fn draw_method_selector(f: &mut Frame, app: &App, area: Rect) {
    let methods = HttpMethod::all();
    let width = 15;
    let height = (methods.len() + 2) as u16;
    
    // Account for main layout and request section
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20),  // Nav panel
            Constraint::Min(0),      // Main content
        ])
        .split(area);

    let content_area = main_layout[1];
    let request_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // URL bar area
            Constraint::Min(0),
        ])
        .margin(1)
        .split(content_area)[0];

    // Get method box position (first 10% of URL bar)
    let method_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),  // Method width
            Constraint::Min(0),
        ])
        .split(request_area)[0];

    // Create popup area aligned with method box
    let popup_area = Rect {
        x: method_area.x,
        y: method_area.y,
        width: width.max(method_area.width),
        height,
    };

    // Clear the background and draw the list
    f.render_widget(Clear, popup_area);

    let items: Vec<ListItem> = methods
        .iter()
        .map(|method| {
            ListItem::new(method.as_str())
                .style(Style::default().fg(method.color()))
        })
        .collect();

    let methods_list = List::new(items)
        .block(Block::default()
            .title("Method")
            .borders(Borders::ALL))
        .highlight_style(Style::default()
            .fg(Color::Black)
            .bg(Color::White));

    f.render_stateful_widget(
        methods_list,
        popup_area,
        &mut ListState::default().with_selected(Some(app.selector_method_index)),
    );
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
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
            Constraint::Length((r.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
} 