use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, Wrap};
use tui::Frame;
use tui_logger::TuiLoggerWidget;

use super::actions::Actions;
use super::db::Ingredient;
use super::state::{AppState, PopUp};
use crate::app::App;

pub fn draw<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let size = rect.size();
    check_size(&size);

    // Vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(12),
            ]
            .as_ref(),
        )
        .split(size);

    // Title
    let title = draw_title();
    rect.render_widget(title, chunks[0]);

    // Body & Help
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(32)].as_ref())
        .split(chunks[1]);

    if let AppState::IngredientView {
        ref mut selection,
        ingredients,
        ..
    } = &mut app.state
    {
        let ingredients = draw_ingredients(ingredients.as_slice());
        rect.render_stateful_widget(ingredients, body_chunks[0], selection);
    }

    let help = draw_help(app.actions());
    rect.render_widget(help, body_chunks[1]);

    // Logs
    let logs = draw_logs();
    rect.render_widget(logs, chunks[2]);

    if let Some(popup) = app.state.popup() {
        draw_popups(popup, rect)
    }
}

fn draw_title<'a>() -> Paragraph<'a> {
    Paragraph::new("Plop with TUI")
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        )
}

fn check_size(rect: &Rect) {
    if rect.width < 52 {
        panic!("Require width >= 52, (got {})", rect.width);
    }
    if rect.height < 28 {
        panic!("Require height >= 28, (got {})", rect.height);
    }
}

fn draw_ingredients(ingredients: &[Ingredient]) -> Table {
    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);

    let mut rows = vec![Row::new(vec![
        Cell::from(Span::styled("id", key_style)),
        Cell::from(Span::styled("name", key_style)),
    ])];
    for ingredient in ingredients {
        let row = Row::new(vec![
            Cell::from(Span::styled(
                ingredient.ingredient_id.to_string(),
                help_style,
            )),
            Cell::from(Span::styled(ingredient.name.clone(), help_style)),
        ]);
        rows.push(row);
    }

    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title("Ingredients"),
        )
        .highlight_style(selected_style)
        .highlight_symbol(">>")
        .widths(&[Constraint::Length(11), Constraint::Min(20)])
        .column_spacing(1)
}

fn draw_help(actions: &Actions) -> Table {
    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);

    let mut rows = vec![];
    for action in actions.actions().iter() {
        let mut first = true;
        for key in action.keys() {
            let help = if first {
                first = false;
                action.to_string()
            } else {
                String::from("")
            };
            let row = Row::new(vec![
                Cell::from(Span::styled(key.to_string(), key_style)),
                Cell::from(Span::styled(help, help_style)),
            ]);
            rows.push(row);
        }
    }

    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title("Help"),
        )
        .widths(&[Constraint::Length(11), Constraint::Min(20)])
        .column_spacing(1)
}

fn draw_logs<'a>() -> TuiLoggerWidget<'a> {
    TuiLoggerWidget::default()
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Gray))
        .style_info(Style::default().fg(Color::Blue))
        .block(
            Block::default()
                .title("Logs")
                .border_style(Style::default().fg(Color::White).bg(Color::Black))
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White).bg(Color::Black))
}

fn draw_popups<B: Backend>(popup: &PopUp, frame: &mut Frame<B>) {
    let (title, text) = match popup {
        PopUp::Delete { id } => (
            "Delete".to_string(),
            format!("Do you really want to delete {id}?"),
        ),
        PopUp::AddSourceUrl { ingredient, url } => {
            (format!("Url for {ingredient}:"), url.to_owned())
        }
        PopUp::AddSourceWeight {
            ingredient, weight, ..
        } => (format!("Weight for {ingredient}:"), weight.to_owned()),
    };
    let paragraph = Paragraph::new(Span::styled(
        text,
        Style::default().add_modifier(Modifier::SLOW_BLINK),
    ))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });

    let block = Block::default()
        .style(Style::default().bg(Color::Black))
        .title(title)
        .borders(Borders::ALL);
    //let block = Block::default().title(text.clone()).borders(Borders::ALL);
    let area = centered_rect(60, 20, frame.size());
    frame.render_widget(Clear, area); //this clears out the background
    let block_rect = block.inner(area);
    frame.render_widget(paragraph, block_rect);
    frame.render_widget(block, area);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
