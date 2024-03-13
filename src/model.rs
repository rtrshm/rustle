use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::*,
    prelude::*,
    style::{Color, Style},
    widgets::calendar::{CalendarEventStore, Monthly},
    widgets::*,
};

use std::{
    fs::{read_dir, ReadDir},
    path::PathBuf,
    time::Duration,
};
use time::{format_description, Date, OffsetDateTime};
use tui_textarea::TextArea;

#[derive(Debug, Default)]
pub struct Model<'a> {
    pub running_state: RunningState,
    textarea: TextArea<'a>,
    active_window: ActiveWindow,
    calendar_state: CalendarState,
    selected_listing: usize,
    num_listings: usize,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Debug, Default, PartialEq)]
pub enum ActiveWindow {
    #[default]
    Menu,
    EditBox,
}

#[derive(Debug)]
struct CalendarState {
    enabled: bool,
    selected_date: Date,
}

impl Default for CalendarState {
    fn default() -> CalendarState {
        CalendarState {
            enabled: false,
            selected_date: OffsetDateTime::now_utc().date(),
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
    Other,
}

pub fn handle_event(model: &mut Model) -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match model.active_window {
                    ActiveWindow::Menu => return Ok(handle_key(key)),
                    ActiveWindow::EditBox => match key.code {
                        KeyCode::Tab => return Ok(handle_key(key)),
                        _ => {
                            model.textarea.input(key);
                            return Ok(None);
                        }
                    },
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
        _ => None,
    }
}

fn update_date(model: &mut Model, offset_in_days: i64) {
    let result = model
        .calendar_state
        .selected_date
        .checked_add(time::Duration::days(offset_in_days));
    if let Some(result_date) = result {
        if result_date.month() == model.calendar_state.selected_date.month() {
            model.calendar_state.selected_date = result_date;
            model.selected_listing = 0;
            model.num_listings = 0;
        }
    }
}

pub fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::Quit => {
            model.running_state = RunningState::Done;
        }
        Message::SwitchWindows => match model.active_window {
            ActiveWindow::EditBox => model.active_window = ActiveWindow::Menu,
            ActiveWindow::Menu => model.active_window = ActiveWindow::EditBox,
        },
        Message::ToggleCalendar => {
            model.calendar_state.enabled = !model.calendar_state.enabled;
        }

        Message::Up => {
            if model.calendar_state.enabled {
                update_date(model, -7);
            }

            if model.active_window == ActiveWindow::Menu && model.selected_listing > 0 {
                model.selected_listing -= 1;
            }
        }

        Message::Down => {
            if model.calendar_state.enabled {
                update_date(model, 7);
            }

            if model.active_window == ActiveWindow::Menu
                && model.num_listings > 0
                && model.selected_listing < model.num_listings - 1
            {
                model.selected_listing += 1
            }
        }

        Message::Left => {
            if model.calendar_state.enabled {
                update_date(model, -1);
            }
        }

        Message::Right => {
            if model.calendar_state.enabled {
                update_date(model, 1)
            }
        }

        Message::Enter => {
            if model.calendar_state.enabled {
                model.calendar_state.enabled = !model.calendar_state.enabled;
            }
        }
        _ => return None,
    }
    None
}

fn format_date(date: Date) -> String {
    let format = format_description::parse("[year]-[month]-[day]").unwrap();
    date.format(&format).unwrap()
}

fn list_entries(selected_date: Date) -> Option<ReadDir> {
    let dirname = format_date(selected_date);
    let mut path = PathBuf::new();
    path.push("entries");
    path.push(dirname);

    match read_dir(path) {
        Ok(dir_entries) => Some(dir_entries),
        _ => None,
    }
}

pub fn view(model: &mut Model, f: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
        .split(f.size());

    let mut textarea_block = Block::default().borders(Borders::ALL).title("textArea");

    let mut menu_block = Block::default().borders(Borders::ALL);

    match model.active_window {
        ActiveWindow::Menu => {
            menu_block = menu_block.border_style(Style::default().fg(Color::LightBlue));
        }
        ActiveWindow::EditBox => {
            textarea_block = textarea_block.border_style(Style::default().fg(Color::LightBlue));
        }
    };

    model.textarea.set_block(textarea_block);

    const MENU_LISTING_HEIGHT: u16 = 2;

    let menu_listings: usize = (layout[0].as_size().height / MENU_LISTING_HEIGHT - 1).into();

    let menu_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(MENU_LISTING_HEIGHT); menu_listings])
        .split(menu_block.inner(layout[0]));

    if let Some(selected_entries) = list_entries(model.calendar_state.selected_date) {
        for entry in Iterator::enumerate(selected_entries) {
            if let (idx, Ok(file_data)) = entry {
                if let Some(file_name) = file_data.file_name().to_str() {
                    let mut entry_block = Block::new();
                    if model.selected_listing == idx {
                        entry_block = entry_block.style(Style::new().bg(Color::LightBlue));
                    }
                    f.render_widget(
                        Paragraph::new(file_name).block(entry_block).centered(),
                        menu_layout[idx],
                    );
                }
                model.num_listings = idx + 1;
            }
        }
    }

    menu_block = menu_block.title(format!(
        "{date} - {listings} entries",
        date = format_date(model.calendar_state.selected_date),
        listings = model.num_listings
    ));
    f.render_widget(menu_block, layout[0]);
    f.render_widget(model.textarea.widget(), layout[1]);

    if model.calendar_state.enabled {
        let mut cal_event_store =
            CalendarEventStore::today(Style::new().light_blue().add_modifier(Modifier::UNDERLINED));
        cal_event_store.add(
            model.calendar_state.selected_date,
            Style::new().yellow().bold(),
        );

        let area = Rect {
            width: 30,
            height: 12,
            x: 5,
            y: f.size().height - 12,
        };

        let cal_block = Block::bordered().border_type(BorderType::Rounded);

        let cal = Monthly::new(model.calendar_state.selected_date, cal_event_store)
            .block(cal_block.clone())
            .show_month_header(Style::new().bold());

        let inner_area = cal_block.inner(area);

        f.render_widget(cal, inner_area);
    }
}
