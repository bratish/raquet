use ratatui::style::{Color, Style, Modifier};
use crate::app::{App, Field, InputMode};

pub fn style_for_field(field: Field, app: &App) -> Style {
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