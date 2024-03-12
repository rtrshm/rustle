use ratatui::{
    style::{
        Style, Color
    },
    layout::*,
    prelude::*,
    widgets::*,
    widgets::calendar::{
        CalendarEventStore,
        Monthly,
        self
    }
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind}
};
use tui_textarea::TextArea;

use std::time::Duration;
use time::OffsetDateTime;

#[derive(Debug, Default)]
pub struct Model<'a> {
    pub running_state: RunningState,
    textarea: TextArea<'a>,
    active_window: ActiveWindow,
    calendar_out: bool
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum RunningState {
    #[default]
    Running,
    Done
}

#[derive(Debug, Default, PartialEq)]
pub enum ActiveWindow { 
    #[default]
    Menu,
    EditBox
}

#[derive(PartialEq)]
pub enum Message {
    Left,
    Right,
    Down,
    Up,
    Save,
    Quit,
    SwitchWindows,
    Other
}


pub fn handle_event(_m: &mut Model) -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250)) ? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match _m.active_window {
                    ActiveWindow::Menu => return Ok(handle_key(key)),
                    ActiveWindow::EditBox => {
                        match key.code {
                            KeyCode::Tab => return Ok(handle_key(key)),
                            _ => {
                                _m.textarea.input(key);
                                return Ok(None)
                            }
                        }   
                    }
                }
            }
        }
    }    
    Ok(None)    
}

fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::Quit),
        KeyCode::Tab => Some(Message::SwitchWindows),
        _ => None
    }
}

pub fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::Quit => {
            model.running_state = RunningState::Done;
        },
        Message::SwitchWindows => {
            match model.active_window {
                ActiveWindow::EditBox => {
                    model.active_window = ActiveWindow::Menu
                },
                ActiveWindow::Menu => {
                    model.active_window = ActiveWindow::EditBox
                }
            }
        }
        _ => return None
    }
    None
}

pub fn view(model: &mut Model, f: &mut Frame) {

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Ratio(1, 3),
            Constraint::Ratio(2, 3)
        ])
        .split(f.size());

    let mut textarea_block = Block::default()
            .borders(Borders::ALL)
            .title("textArea");

    let mut menu_block = Block::default()
            .borders(Borders::ALL)
            .title("menu");

    match model.active_window {
        ActiveWindow::Menu => {
            menu_block = menu_block.border_style(Style::default().fg(Color::LightBlue));
        },
        ActiveWindow::EditBox => {
            textarea_block = textarea_block.border_style(Style::default().fg(Color::LightBlue));
        }
    };

    model.textarea.set_block(
        textarea_block
    );

    f.render_widget(Paragraph::new("left")
        .block(menu_block),
    layout[0]);

    f.render_widget(model.textarea.widget(),
        layout[1]);

    let today = OffsetDateTime::now_utc().date();
    let mut cal = Monthly::new(today,
        CalendarEventStore::today(Style::new().red().bold())
    );

    let area = Rect {
        width: 40,
        height: 20,
        x: 5, 
        y: 5
    };

    f.render_widget(cal, area);

}



