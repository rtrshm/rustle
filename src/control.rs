use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use crate::model::{ActiveWindow, ModelState};
use ratatui::prelude::*;
use std::io::stdout;
use std::time::Duration;
use tui_textarea::{Input, Key};

use crate::view::view;

pub fn run() -> color_eyre::Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    initialize_panic_handler();
    let mut model = ModelState::default();
    let mut model = model.refresh_menu();

    loop {
        if model.done() {
            break;
        }

        terminal.draw(|frame| view(&mut model, frame))?;

        let mut current_message = handle_event(&mut model)?;

        while current_message.is_some() {
            current_message = update(&mut model, current_message.unwrap())
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

/// leave alternate terminal screen on crash to keep it usable
fn initialize_panic_handler() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));
}

/// catch keypress events
fn handle_event(model: &mut ModelState) -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                return Ok(handle_key(model, key));
            }
        }
    }
    Ok(None)
}

/// interpret action
fn handle_key(model: &mut ModelState, input: impl Into<Input>) -> Option<Message> {
    match model.active_window() {
        ActiveWindow::EditBox => match input.into() {
            Input { key: Key::Tab, .. } => Some(Message::SwitchWindows(ActiveWindow::Menu)),
            Input {
                key: Key::Char('s'),
                ctrl: true,
                ..
            } => Some(Message::SaveFile),
            Input {
                key: Key::Char('n'),
                ctrl: true,
                ..
            } => Some(Message::OpenFilenameEditbox),
            input => {
                model.input_editbox(input);
                None
            }
        },

        ActiveWindow::TextPopup => match input.into() {
            Input { key: Key::Esc, .. } => Some(Message::SwitchWindows(ActiveWindow::Menu)),
            Input {
                key: Key::Enter, ..
            } => Some(Message::CreateFile),
            input => {
                model.input_popup(input);
                None
            }
        },

        // on main menu:
        // q: quit
        // c: toggle calendar
        // ^n: create new file
        ActiveWindow::Menu => match input.into() {
            Input {
                key: Key::Char('q'),
                ..
            } => Some(Message::Quit),
            Input {
                key: Key::Char('c'),
                ..
            } => Some(Message::ToggleCalendar),
            Input {
                key: Key::Char('n'),
                ctrl: true,
                ..
            } => Some(Message::OpenFilenameEditbox),
            Input { key: Key::Tab, .. } => Some(Message::SwitchWindows(ActiveWindow::EditBox)),
            Input { key: Key::Up, .. } => Some(Message::Up),
            Input { key: Key::Down, .. } => Some(Message::Down),
            Input { key: Key::Left, .. } => Some(Message::Left),
            Input {
                key: Key::Right, ..
            } => Some(Message::Right),
            Input {
                key: Key::Enter, ..
            } => Some(Message::Enter),
            _ => None,
        },
    }
}

fn update(model: &mut ModelState, msg: Message) -> Option<Message> {
    match msg {
        Message::Quit => model.terminate(),
        Message::SwitchWindows(window) => {
            model.switch_window(window);
        }
        Message::ToggleCalendar => {
            model.toggle_calendar();
        }
        Message::OpenFilenameEditbox => {
            model.switch_window(ActiveWindow::TextPopup);
        }
        Message::CreateFile => {
            model
                .save_new_file()
                .refresh_menu()
                .switch_window(ActiveWindow::EditBox);
        }

        Message::SaveFile => {
            if model.selected_listing().is_none() {
                return Some(Message::OpenFilenameEditbox)
            } else {
                model.save_selected_file();
            }
        }

        Message::Up => {
            if model.calendar_enabled() {
                model.select_prev_week();
            } else if *model.active_window() == ActiveWindow::Menu {
                model.select_prev_listing();
            }
        }

        Message::Down => {
            if model.calendar_enabled() {
                model.select_next_week();
            } else if *model.active_window() == ActiveWindow::Menu {
                model.select_next_listing();
            }
        }

        Message::Left => {
            if model.calendar_enabled() {
                model.select_prev_day();
            }
        }

        Message::Right => {
            if model.calendar_enabled() {
                model.select_next_day();
            }
        }

        Message::Enter => {
            if model.calendar_enabled() {
                model.toggle_calendar();
            }
        }
        _ => (),
    }
    None
}

#[derive(PartialEq)]
pub enum Message {
    SwitchWindows(crate::model::ActiveWindow),
    OpenFilenameEditbox,
    CreateFile,
    SaveFile,
    Left,
    Right,
    Down,
    Up,
    Save,
    Quit,
    ToggleCalendar,
    Enter,
    Other,
}
