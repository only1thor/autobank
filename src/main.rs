use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::io;

use ratatui::{Terminal, backend::CrosstermBackend, widgets::ListState};

use crate::{fileio::read_access_token_file, models::Account};

mod api;
mod auth;
mod fileio;
mod models;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = fileio::get_config_file();

    auth::auth(config.client_id, config.client_secret);

    // Setup terminal
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    // Our list of names
    //let names: Vec<&str> = vec!["Norma", "Bob", "Charlie", "Diana", "Eve", "Frank"];

    let accounts = get_accounts();

    // Track selected item
    let mut state = ListState::default();
    state.select(Some(0));

    loop {
        ui::draw(&mut terminal, &mut state, &accounts);

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => {
                        let i = state.selected().map_or(0, |i| (i + 1) % accounts.len());
                        state.select(Some(i));
                    }
                    KeyCode::Up => {
                        let i = state
                            .selected()
                            .map_or(0, |i| (i + accounts.len() - 1) % accounts.len());
                        state.select(Some(i));
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn get_accounts() -> Vec<Account> {
    let access_token = read_access_token_file().access_token;
    let data = api::get_accounts(access_token);
    data.accounts
}
