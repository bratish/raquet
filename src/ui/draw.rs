use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
    style::{Color, Style, Modifier},
};
use crate::app::{App, Field, NavItem};
use super::components::{
    self, draw_collections, draw_history, draw_headers, 
    draw_request, draw_request_body, draw_response_headers, 
    draw_response_body, draw_save_dialog,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    // Create main layout with left nav and main content
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // Left navigation panel
            Constraint::Min(0),     // Main content
        ])
        .split(f.size());

    // Draw navigation panel
    draw_nav_panel(f, app, main_layout[0]);

    // If collections or history is being shown, draw it over main content
    if app.show_collections && !app.save_dialog_visible {
        draw_collections(f, app, main_layout[1]);
        return;
    }
    if app.show_history {
        draw_history(f, app, main_layout[1]);
        return;
    }

    // Split main content into request and response sections
    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // Request section (40%)
            Constraint::Percentage(60), // Response section (60%)
        ])
        .split(main_layout[1]);

    // Draw request section
    let request_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // URL bar
            Constraint::Min(0),     // Headers and body
        ])
        .margin(1)
        .split(content_layout[0]);

    // Draw URL bar
    draw_request(f, app, request_layout[0]);

    // Split headers and body horizontally
    let request_sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(request_layout[1]);

    draw_headers(f, app, request_sections[0]);
    draw_request_body(f, app, request_sections[1]);

    // Draw response section with divider
    let response_area = {
        let response_block = Block::default()
            .title("Response")
            .title_alignment(Alignment::Left)
            .borders(Borders::TOP)  // Only show top border as divider
            .border_style(Style::default())
            .style(Style::default());
        
        f.render_widget(response_block.clone(), content_layout[1]);
        response_block.inner(content_layout[1])
    };

    // Split response area into left column and body
    let response_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Left column (status + headers)
            Constraint::Percentage(70), // Response body
        ])
        .split(response_area);

    draw_response_headers(f, app, response_layout[0]);
    draw_response_body(f, app, response_layout[1]);

    // Draw save dialog on top if visible
    if app.save_dialog_visible {
        draw_save_dialog(f, app, f.size());
    }
}

fn draw_nav_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let nav_items: Vec<ListItem> = NavItem::all()
        .iter()
        .map(|item| {
            let style = if *item == app.nav_selected {
                if app.active_field == Field::NavPanel {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
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

    f.render_widget(nav_list, area);
}