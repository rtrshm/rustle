use crossterm::{
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen
    },
    ExecutableCommand
};
use ratatui::prelude::*;
use std::io::stdout;

pub mod model;

use crate::model::{
    Model, RunningState, view, handle_event, update
};

fn main() -> color_eyre::Result<()> {

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut model = Model::default();


    while model.running_state != RunningState::Done {

        terminal.draw(|frame| view(&mut model, frame))?;

        let mut current_message = handle_event(&mut model)?;

        while current_message.is_some() {
            current_message = update(&mut model, current_message.unwrap());
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

