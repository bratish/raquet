use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
    style::{Color, Style, Modifier},
};
use crate::app::{App, state::HeaderDialogState};

pub fn draw_header_dialog(f: &mut Frame, app: &App, area: Rect) {
    if app.header_dialog == HeaderDialogState::Hidden {
        return;
    }

    // Create a centered box with safe dimensions
    let width = 60.min(area.width.saturating_sub(4));
    let height = 10.min(area.height.saturating_sub(4));
    
    // Calculate center position
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    
    // Create popup area in center of screen
    let popup_area = Rect::new(x, y, width, height);

    // Render background overlay
    let overlay = Block::default()
        .style(Style::default().bg(Color::Black));
    f.render_widget(Clear, area);
    f.render_widget(overlay, area);
    
    // Render the dialog box
    let dialog = Block::default()
        .title(" Add New Header ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(Clear, popup_area);
    f.render_widget(dialog, popup_area);

    // Create layout for input fields
    let inner = popup_area.inner(&Margin::new(1, 1));
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Key
            Constraint::Length(3), // Value
            Constraint::Length(2), // Buttons
        ])
        .spacing(1)
        .split(inner);

    // Key input with better styling
    let key_style = if app.header_dialog == HeaderDialogState::EditingKey {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let mut key_text = app.new_header_key.clone();
    if app.header_dialog == HeaderDialogState::EditingKey {
        key_text.insert(app.new_header_key_cursor, '|');
    }
    let key_input = Paragraph::new(key_text)
        .block(Block::default()
            .title(" Key ")
            .borders(Borders::ALL)
            .border_style(key_style));
    f.render_widget(key_input, chunks[0]);

    // Value input with better styling
    let value_style = if app.header_dialog == HeaderDialogState::EditingValue {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let mut value_text = app.new_header_value.clone();
    if app.header_dialog == HeaderDialogState::EditingValue {
        value_text.insert(app.new_header_value_cursor, '|');
    }
    let value_input = Paragraph::new(value_text)
        .block(Block::default()
            .title(" Value ")
            .borders(Borders::ALL)
            .border_style(value_style));
    f.render_widget(value_input, chunks[1]);

    // Buttons with better styling
    let buttons = Paragraph::new("Press Enter to save, Esc to cancel")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(buttons, chunks[2]);
} 