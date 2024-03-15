use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{
    layout::*,
    prelude::*,
    style::{Color, Style},
    widgets::calendar::{CalendarEventStore, Monthly},
    widgets::*,
};

use std::{
    fs::{read_dir, read_to_string, DirEntry, File},
    io::prelude::*,
    path::PathBuf,
    time::Duration,
};
use time::{format_description, Date, OffsetDateTime};
use tui_textarea::{Input, Key, TextArea};

const MENU_LISTING_HEIGHT: u16 = 2;

#[derive(Debug, Default)]
pub struct Model<'a> {
    pub running_state: RunningState,
    active_window: ActiveWindow,
    calendar_state: CalendarState,
    menu_state: MenuState,
    editbox_textarea: TextArea<'a>,
    popup_textarea: TextArea<'a>
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
    TextPopup,
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

#[derive(Debug, Default)]
pub struct MenuState {
    selected_file_idx: i8,
    listings: Vec<MenuListing>,
}

#[derive(Debug, Default)]
pub struct MenuListing {
    path: PathBuf,
    filename: String,
}

#[derive(PartialEq)]
pub enum Message {
    SwitchWindows(ActiveWindow),
    CreateNewFile,
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

pub fn handle_event(model: &mut Model) -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                return Ok(handle_key(model, key));
            }
        }
    }
    Ok(None)
}

fn handle_key(model: &mut Model, input: impl Into<Input>) -> Option<Message> {
    match model.active_window {
        ActiveWindow::EditBox => match input.into() {
            Input {
                key: Key::Tab,
                ctrl: false,
                alt: false,
                shift: false,
            } => Some(Message::SwitchWindows(ActiveWindow::Menu)),
            Input {
                key: Key::Char('s'),
                ctrl: true,
                alt: false,
                shift: false,
            } => Some(Message::SaveFile),
            Input {
                key: Key::Char('n'),
                ctrl: true,
                alt: false,
                shift: false,
            } => Some(Message::CreateNewFile),
            input => {
                model.editbox_textarea.input(input);
                None
            }
        },

        ActiveWindow::TextPopup => match input.into() {
            Input { key: Key::Esc, .. } => Some(Message::SwitchWindows(ActiveWindow::Menu)),
            Input {
                key: Key::Enter, ..
            } => Some(Message::SaveFile),
            input => {
                model.popup_textarea.input(input);
                None
            }
        },

        ActiveWindow::Menu => match input.into().key {
            Key::Char('q') => Some(Message::Quit),
            Key::Char('c') => Some(Message::ToggleCalendar),
            Key::Tab => Some(Message::SwitchWindows(ActiveWindow::EditBox)),
            Key::Up => Some(Message::Up),
            Key::Down => Some(Message::Down),
            Key::Left => Some(Message::Left),
            Key::Right => Some(Message::Right),
            Key::Enter => Some(Message::Enter),
            _ => None,
        },
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
            update_listings(model, result_date, None);
        }
    }
}

fn update_listings(model: &mut Model, new_date: Date, set_selected: Option<&str>) {
    let mut to_select: i8 = 0;
    let mut menu_entries: Vec<MenuListing> = Vec::new();
        
    if let Some(entries) = list_entries(new_date) {
        for (idx, entry) in entries.iter().enumerate() {
            if let Some(filename) = entry.file_name().to_str() {
                if let Some(to_be_selected) = set_selected {
                    if filename == to_be_selected {
                        to_select = idx as i8;
                    }
                }
                menu_entries.push(MenuListing {
                    path: entry.path(),
                    filename: String::from(filename),
                });
            }
        }
    } else {
        model.editbox_textarea = TextArea::default();
    }

    model.menu_state = MenuState {
        listings: menu_entries,
        selected_file_idx: to_select
    };
    update_selected_listing(model, to_select);
}

fn update_selected_listing(model: &mut Model, new_idx: i8) {
    if new_idx >= 0 && new_idx < model.menu_state.listings.len() as i8 {
        if let Some(menu_listing) = model.menu_state.listings.get(new_idx as usize) {
            if let Ok(file) = read_to_string(&menu_listing.path) {
                model.menu_state.selected_file_idx = new_idx;

                let mut textarea_buffer = Vec::new();
                for line in file.lines() {
                    textarea_buffer.push(line);
                }
                model.editbox_textarea = TextArea::from(textarea_buffer);
            }
        }
    }
}

pub fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::Quit => {
            model.running_state = RunningState::Done;
        }
        Message::SwitchWindows(window) => model.active_window = window,

        Message::ToggleCalendar => {
            model.calendar_state.enabled = !model.calendar_state.enabled;
        }

        Message::CreateNewFile => return Some(Message::SwitchWindows(ActiveWindow::TextPopup)),

        Message::CreateFile => {
            let current_name = &model.popup_textarea.lines()[0].replace('\n', ".txt");
            let path: PathBuf = [
                ".",
                "entries",
                &format_date(model.calendar_state.selected_date),
                &current_name,
            ]
            .iter()
            .collect();

            if File::create(path).is_ok() {
                model.editbox_textarea = TextArea::default();
                update_listings(
                    model,
                    model.calendar_state.selected_date,
                    Some(current_name),
                )
            }
        }

        Message::SaveFile => {
            if let Some(listing) = model
                .menu_state
                .listings
                .get(model.menu_state.selected_file_idx as usize)
            {
                return if let Ok(mut file) = File::create(&listing.path) {
                    let bytes: Vec<u8> = model
                        .editbox_textarea
                        .lines()
                        .iter()
                        .flat_map(|s| s.as_bytes().iter().copied())
                        .collect();
                    file.write_all(&bytes)
                        .expect("Failed to write current buffer");
                    file.flush().expect("Failed to flush");
                    Some(Message::SwitchWindows(ActiveWindow::Menu))
                } else {
                    None
                };
            }
        }

        Message::Up => {
            if model.calendar_state.enabled {
                update_date(model, -7);
            }

            if model.active_window == ActiveWindow::Menu {
                update_selected_listing(model, model.menu_state.selected_file_idx - 1);
            }
        }

        Message::Down => {
            if model.calendar_state.enabled {
                update_date(model, 7);
            }

            if model.active_window == ActiveWindow::Menu {
                update_selected_listing(model, 1);
            }
        }

        Message::Left => {
            if model.calendar_state.enabled {
                update_date(model, -1);
                update_selected_listing(model, 0);
            }
        }

        Message::Right => {
            if model.calendar_state.enabled {
                update_date(model, 1);
                update_selected_listing(model, 0);
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

fn list_entries(selected_date: Date) -> Option<Vec<DirEntry>> {
    let dirname = format_date(selected_date);
    let mut path = PathBuf::new();
    path.push("entries");
    path.push(dirname);

    let mut result = Vec::new();

    if let Ok(dir_entries) = read_dir(path) {
        for dir_entry in dir_entries.flatten() {
            result.push(dir_entry);
        }
        Some(result)
    } else {
        None
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
        ActiveWindow::TextPopup => {
            let area = Rect {
                width: 20,
                height: 4,
                x: 10,
                y: MENU_LISTING_HEIGHT * ((model.menu_state.selected_file_idx as u16) + 1),
            };

            let popup_textarea_block = Block::bordered().border_type(BorderType::Rounded);
            model.popup_textarea.set_block(popup_textarea_block);
            f.render_widget(model.popup_textarea.widget(), area);
        }
    };

    model.editbox_textarea.set_block(textarea_block);

    let limit_in_view: usize = (layout[0].as_size().height / MENU_LISTING_HEIGHT - 1).into();

    let menu_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(MENU_LISTING_HEIGHT); limit_in_view])
        .split(menu_block.inner(layout[0]));

    for (idx, menu_listing) in model.menu_state.listings.iter().enumerate() {
        let mut entry_block = Block::new();
        if model.menu_state.selected_file_idx == idx as i8 {
            entry_block = entry_block.style(Style::new().bg(Color::LightBlue));
        }

        f.render_widget(
            Paragraph::new(String::from(&menu_listing.filename))
                .block(entry_block)
                .centered(),
            menu_layout[idx],
        );
    }

    menu_block = menu_block.title(format!(
        "{date} - {listings} entries",
        date = format_date(model.calendar_state.selected_date),
        listings = model.menu_state.listings.len()
    ));
    f.render_widget(menu_block, layout[0]);
    f.render_widget(model.editbox_textarea.widget(), layout[1]);

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
