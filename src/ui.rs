use std::io::Stdout;

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::models::Account;

pub fn draw(
    terminal: &mut Terminal<CrosstermBackend<&mut Stdout>>,
    state: &mut ListState,
    accounts: &Vec<Account>,
) {
    let _ = terminal.draw(|f| {
        // Layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(f.area());

        // Convert names to ListItems
        let items: Vec<ListItem> = accounts
            .iter()
            .map(|acc| ListItem::new(acc.name.clone()))
            .collect();

        // Create the List widget
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Accounts"))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(list, chunks[0], &mut state.clone());
    });
}
