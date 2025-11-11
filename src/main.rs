use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use log::debug;
use std::{
    io,
    time::{Duration, Instant},
};

use ratatui::{Terminal, backend::CrosstermBackend, widgets::{ListState, TableState}};

use crate::{fileio::read_access_token_file, models::Account};

mod api;
mod auth;
mod fileio;
mod models;
mod ui;

use tachyonfx::{
    EffectManager, Interpolation,
    fx::{self},
};

pub struct AppState {
    pub account_state: TableState,
    pub menu_state: ListState,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger (off by default, enable with RUST_LOG=debug)
    env_logger::init();

    let config = fileio::get_config_file();

    auth::auth(config.client_id, config.client_secret);

    // Setup terminal
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    let accounts = get_accounts();

    // Track selected item
    // let mut account_state = TableState::default();
    // account_state.select(Some(0));

    // let mut menu_state = ListState::default();
    // menu_state.select(Some(0));


    let mut show_balance = false;

    let mut effects: EffectManager<()> = EffectManager::default();

    // Add a simple fade-in effect
    let coalesce_in = fx::coalesce((500, Interpolation::QuintIn));
    effects.add_effect(coalesce_in);

    let mut last_frame = Instant::now();
    let mut exiting = false;
    let mut exit_start_time: Option<Instant> = None;
    let exit_duration = Duration::from_millis(500);
    let mut menu_open = false;

    let mut app_state = AppState {
        account_state:  TableState::new().with_selected(0),
        menu_state:  ListState::default().with_selected(Some(0))
    };

    let menu_length = 2;

    loop {
        let elapsed = last_frame.elapsed();
        last_frame = Instant::now();

        ui::draw(
            &mut app_state,
            &mut terminal,
            &accounts,
            &show_balance,
            &menu_open,
            &mut effects,
            elapsed,
        );

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        if !exiting {
                            effects.add_effect(fx::dissolve((500, Interpolation::QuintIn)));
                            exiting = true;
                            exit_start_time = Some(Instant::now());
                        }
                    }
                    KeyCode::Down => {

                        if !menu_open {
                            let i = app_state.account_state.selected().map_or(0, |i| (i + 1) % accounts.len());
                            app_state.account_state.select(Some(i));
                        }
                        else {
                            let i = app_state.menu_state.selected().map_or(0, |i| (i + 1) % menu_length);
                            app_state.menu_state.select(Some(i));
                        }
                        
                    }
                    KeyCode::Up => {
                        if !menu_open {
                            let i = app_state.account_state.selected().map_or(0, |i| (i + accounts.len() - 1) % accounts.len());
                            app_state.account_state.select(Some(i));
                        }
                        else {
                            let i = app_state.menu_state.selected().map_or(0, |i| (i + menu_length - 1) % menu_length);
                            app_state.menu_state.select(Some(i));
                        }
                    }
                    KeyCode::Enter => {menu_open = true},
                    KeyCode::Esc => {menu_open = false},
                    KeyCode::Char('b') => show_balance = !show_balance,
                    _ => {}
                }
            }
        }

        // If exiting and dissolve effect is done, break the loop
        if exiting {
            if let Some(start_time) = exit_start_time {
                if start_time.elapsed() >= exit_duration {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn get_accounts() -> Vec<Account> {
    debug!("Fetching accounts");
    let access_token = read_access_token_file().unwrap().access_token;
    let data = api::get_accounts(access_token);
    data.accounts
}
