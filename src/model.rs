use ratatui::{
    style::{
        Style, Color
    },
    layout::*,
    prelude::*,
    widgets::*,
    widgets::calendar::{
        CalendarEventStore,
        Monthly
    }
};
use crossterm::event::{self,
        Event, 
        KeyCode, 
        KeyEventKind};

use tui_textarea::TextArea;
use std::time::Duration;
use time::{
    Date,
    OffsetDateTime};

#[derive(Debug, Default)]
pub struct Model<'a> {
    pub running_state: RunningState,
    textarea: TextArea<'a>,
    active_window: ActiveWindow,
    calendar_state: CalendarState
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

#[derive(Debug)]
struct CalendarState {
    enabled: bool,
    selected_date: Date
}

impl Default for CalendarState {
    fn default() -> CalendarState {
        CalendarState {
            enabled: false,
            selected_date: OffsetDateTime::now_utc().date()
        }
    }
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
    ToggleCalendar,
    Enter,
    Other
}


pub fn handle_event(model: &mut Model) -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250)) ? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match model.active_window {
                    ActiveWindow::Menu => return Ok(handle_key(key)),
                    ActiveWindow::EditBox => {
                        match key.code {
                            KeyCode::Tab => return Ok(handle_key(key)),
                            _ => {
                                model.textarea.input(key);
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
        KeyCode::Char('c') => Some(Message::ToggleCalendar),
        KeyCode::Up => Some(Message::Up),
        KeyCode::Down => Some(Message::Down),
        KeyCode::Left => Some(Message::Left),
        KeyCode::Right => Some(Message::Right),
        KeyCode::Enter => Some(Message::Enter),
        _ => None
    }
}

fn update_date(model: &mut Model, offset_in_days: i64) {
    let result = model.calendar_state.selected_date.checked_add(time::Duration::days(offset_in_days));
    match result {
        Some(result_date) =>
            if result_date.month() == model.calendar_state.selected_date.month() {
                model.calendar_state.selected_date = result_date
            } 
        _ => ()
    };
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
        },
        Message::ToggleCalendar => {
            model.calendar_state.enabled = !model.calendar_state.enabled;
        },

        Message::Up => {
            if model.calendar_state.enabled {
                update_date(model, -7);
            }
        },

        Message::Down => {
            if model.calendar_state.enabled {
                update_date(model, 7);
            }
        },

        Message::Left => {
            if model.calendar_state.enabled{
                update_date(model, -1);
            }
        },

        Message::Right => {
            if model.calendar_state.enabled {
                update_date(model, 1)
            }
        },

        Message::Enter => {
            if model.calendar_state.enabled {
                model.calendar_state.enabled = !model.calendar_state.enabled;
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

    if model.calendar_state.enabled {

        let mut cal_event_store = CalendarEventStore::today(Style::new().light_blue().add_modifier(Modifier::UNDERLINED));
        cal_event_store.add(model.calendar_state.selected_date, Style::new().red().bold());



        let area = Rect {
            width: 30,
            height: 12,
            x: 5, 
            y: f.size().height - 12
        };

        let cal_block = Block::bordered()
            .border_type(BorderType::Rounded);

        let cal = 
            Monthly::new(model.calendar_state.selected_date, cal_event_store)
                .block(cal_block.clone())
                .show_month_header(Style::new().bold());
        
        let inner_area = cal_block.inner(area);

        f.render_widget(cal, inner_area);
    
    }
}



