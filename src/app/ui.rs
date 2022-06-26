use sqlx::postgres::types::PgMoney;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, TableState, Wrap};
use tui::Frame;
use tui_logger::TuiLoggerWidget;

use super::actions::Actions;
use super::db::{Ingredient, Meal, RecipeIngredient};
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
        .constraints([Constraint::Length(3), Constraint::Min(10), Constraint::Length(12)].as_ref())
        .split(size);

    // Title
    let title = draw_title();
    rect.render_widget(title, chunks[0]);

    // Body & Help
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(32)].as_ref())
        .split(chunks[1]);

    match &mut app.state {
        AppState::IngredientView {
            ref mut selection,
            ingredients,
            ..
        } => {
            let ingredients = draw_ingredients(ingredients.as_slice());
            rect.render_stateful_widget(ingredients, body_chunks[0], selection);
        },
        AppState::MealView {
            ref mut selection,
            meals,
            ..
        } => {
            let ingredients = draw_meal_list(meals.as_slice());
            rect.render_stateful_widget(ingredients, body_chunks[0], selection);
        },
        _ => (),
    }

    let help = draw_help(app.actions());
    rect.render_widget(help, body_chunks[1]);

    // Logs
    let logs = draw_logs();
    rect.render_widget(logs, chunks[2]);

    if let Some(mut popup) = app.state.popup().cloned() {
        draw_popups(&mut popup, rect)
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
    if rect.height < 12 {
        panic!("Require height >= 12, (got {})", rect.height);
    }
}

fn draw_table(title: &str, header: Vec<String>, content: Vec<Vec<String>>) -> Table {
    let key_style = Style::default().fg(Color::LightCyan);
    let item_style = Style::default().fg(Color::Gray);
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);

    let mut rows = vec![Row::new(
        header.into_iter().map(|name| Cell::from(Span::styled(name, key_style))),
    )];
    rows.extend(content.into_iter().map(|content_row| {
        let cells = content_row
            .into_iter()
            .map(|name| Cell::from(Span::styled(name, item_style)));
        Row::new(cells)
    }));

    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title(title),
        )
        .highlight_style(selected_style)
        .widths(&[
            Constraint::Length(11),
            Constraint::Min(20),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ])
        .column_spacing(1)
}

fn draw_ingredients(ingredients: &[Ingredient]) -> Table {
    draw_table(
        "Ingredients",
        vec!["id".to_owned(), "name".to_owned()],
        ingredients
            .iter()
            .map(|ingredient| vec![ingredient.ingredient_id.to_string(), ingredient.name.to_string()])
            .collect(),
    )
}

fn draw_meal_list(meals: &[Meal]) -> Table {
    draw_table(
        "Meals",
        vec![
            "recipe_id".to_owned(),
            "name".to_owned(),
            "weight".to_owned(),
            "energy".to_owned(),
            "price".to_owned(),
            "servings".to_owned(),
            "start_time".to_owned(),
        ],
        meals
            .iter()
            .map(|meal| {
                vec![
                    meal.recipe_id.to_string(),
                    meal.name.to_string(),
                    format!("{}\tkg", meal.weight),
                    format!("{}\tkj", meal.energy),
                    format_price(&meal.price),
                    meal.servings.to_string(),
                    meal.start_time.format("%b %d %H:%M"),
                ]
            })
            .collect(),
    )
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

fn draw_popups<B: Backend>(popup: &mut PopUp, frame: &mut Frame<B>) {
    fn render_paragraph<B: Backend>(title: String, text: String, frame: &mut Frame<B>, area: Rect) {
        let paragraph = Paragraph::new(Span::styled(text, Style::default().add_modifier(Modifier::SLOW_BLINK)))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        let block = Block::default()
            .style(Style::default().bg(Color::Black))
            .title(title)
            .borders(Borders::ALL);
        let block_rect = block.inner(area);
        frame.render_widget(paragraph, block_rect);
        frame.render_widget(block, area);
    }

    fn render_table<B: Backend>(
        title: &str,
        header: Vec<String>,
        data: Vec<Vec<String>>,
        state: &mut TableState,
        frame: &mut Frame<B>,
        block_rect: Rect,
    ) {
        let table = draw_table(title, header, data);
        frame.render_stateful_widget(table, block_rect, state)
    }

    let title = match popup {
        PopUp::Delete { .. } => "Delete".to_string(),
        PopUp::AddSourceUrl { ingredient, .. } => format!("Url for {ingredient}:"),
        PopUp::AddSourceWeight { ingredient, .. } => format!("Weight for {ingredient}:"),
        PopUp::ViewMealIngredients { meal, .. } => format!("Ingredients for {}:", meal.name),
    };

    let area = centered_rect(60, 20, frame.size());
    frame.render_widget(Clear, area); //this clears out the background

    match popup {
        PopUp::Delete { id } => {
            let text = format!("Do you really want to delete {id}?");
            render_paragraph(title, text, frame, area);
        },
        PopUp::AddSourceUrl { url, .. } => {
            render_paragraph(title, url.to_owned(), frame, area);
        },

        PopUp::AddSourceWeight { weight, .. } => {
            render_paragraph(title, weight.to_owned(), frame, area);
        },
        PopUp::ViewMealIngredients {
            meal,
            ingredients,
            selection,
        } => {
            let (headers, ingredients) = recipe_ingredients_table(ingredients);
            let title = format!("Ingredients for {}:", meal.name);
            render_table(title.as_str(), headers, ingredients, selection, frame, area);
        },
    };
}

fn recipe_ingredients_table(ingredients: &[RecipeIngredient]) -> (Vec<String>, Vec<Vec<String>>) {
    let headers = ["id", "name", "weight", "energy", "price"];
    let headers = headers.iter().map(|name| name.to_string()).collect();
    let format_ingredient = |ingredient: &RecipeIngredient| {
        vec![
            ingredient.ingredient_id.to_string(),
            ingredient.name.to_string(),
            ingredient.weight.to_string(),
            ingredient.energy.to_string(),
            format_price(&ingredient.price),
        ]
    };
    let ingredients = ingredients.iter().map(format_ingredient).collect();
    (headers, ingredients)
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

fn format_price(price: &PgMoney) -> String {
    format!("{}€", price.0 as f32 / 100.)
}
