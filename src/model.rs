
use log::info;
use std::{
    fs::{read_dir, read_to_string, File},
    io::prelude::*,
    path::PathBuf,
};
use time::{format_description, Date, OffsetDateTime};
use tui_textarea::TextArea;

impl ModelState<'_> {
    pub fn done(&mut self) -> bool {
        self.running_state == RunningState::Done
    }

    pub fn terminate(&mut self) -> () {
        self.running_state = RunningState::Done;
    }

    pub fn active_window(&self) -> &ActiveWindow {
        &self.active_window
    }

    pub fn switch_window(&mut self, window: ActiveWindow) -> () {
        self.active_window = window
    }

    pub fn num_menu_listings(&self) -> usize {
        self.menu_state.listings.len()
    }

    /// reference to the currently selected MenuListing, if there is one
    pub fn selected_listing(&self) -> Option<&MenuListing> {
        return if self.menu_state.listings.len() != 0 {
            Some(
                self.menu_state
                    .listings
                    .get(self.menu_state.selected_file_idx as usize)
                    .unwrap(),
            )
        } else {
            None
        };
    }

    pub fn clear_editbox(&mut self) -> &mut Self {
        self.editbox_textarea = TextArea::default();
        self
    }

    pub fn clear_popup_textarea(&mut self) -> &mut Self {
        self.popup_textarea = TextArea::default();
        self
    }

    pub fn listings(&self) -> &Vec<MenuListing> {
        &self.menu_state.listings
    }

    pub fn select_next_listing(&mut self) -> &mut Self {
        self.update_selected_by_index(self.menu_state.selected_file_idx + 1);
        self
    }

    pub fn select_prev_listing(&mut self) -> &mut Self {
        self.update_selected_by_index(self.menu_state.selected_file_idx - 1);
        self
    }

    /// select different file by menu index, display its contents
    pub fn update_selected_by_index(&mut self, new_idx: i8) -> &mut Self {
        if new_idx >= 0 && new_idx < self.menu_state.listings.len() as i8 {
            if let Some(menu_listing) = self.menu_state.listings.get(new_idx as usize) {
                if let Ok(file) = read_to_string(&menu_listing.path) {
                    self.menu_state.selected_file_idx = new_idx;

                    let mut textarea_buffer = Vec::new();
                    for line in file.lines() {
                        textarea_buffer.push(line);
                    }
                    self.editbox_textarea = TextArea::from(textarea_buffer);
                }
            } else {
                self.editbox_textarea = TextArea::default();
            }
        }
        self
    }

    /// select different file by name, display contents
    pub fn update_selected_by_name(&mut self, filename: &str) -> &mut Self {
        for (idx, listing) in self.menu_state.listings.iter().enumerate() {
            if listing.filename == filename {
                return self.update_selected_by_index(idx as i8)
            }
        }
        self
    }

    pub fn selected_date(&self) -> Date {
        self.calendar_state.selected_date
    }

    pub fn refresh_menu(&mut self) -> &mut Self {
        let dirname = Self::format_date(self.calendar_state.selected_date);
        let mut path = PathBuf::new();
        path.push("entries");
        path.push(dirname);

        return if let Ok(dir_entries) = read_dir(path) {
            let new_listings = Vec::from(
                dir_entries
                    .map(|e| {
                        let entry = e.unwrap();
                        MenuListing {
                            path: entry.path(),
                            filename: String::from(entry.file_name().to_str().unwrap()),
                        }
                    })
                    .collect::<Vec<MenuListing>>(),
            );

            self.menu_state = MenuState {
                listings: new_listings,
                ..self.menu_state
            };
            info!(
                "Refreshed; current menu state: {:?}",
                self.menu_state.listings
            );
            self
        } else {
            // if no entries found, clear the menu and editbox
            self.menu_state = MenuState {
                listings: Vec::new(),
                ..self.menu_state
            };

            self.clear_editbox()
        };
    }

    pub fn popup_textarea_content(&self) -> String {
        self.popup_textarea.lines()[0].to_owned()
    }

    pub fn selected_file_idx(&self) -> usize {
        self.menu_state.selected_file_idx as usize
    }

    pub fn save_selected_file(&mut self) -> &mut Self {
        let bytes: Vec<u8> = self
            .editbox_textarea
            .lines()
            .iter()
            .flat_map(|s| s.as_bytes().iter().copied())
            .collect();

        let selected_listing = self.selected_listing().unwrap();
        if let Ok(mut file) = File::create(&selected_listing.path) {
            file.write_all(&bytes)
                .expect("failed to save textarea content");
            file.flush().expect("failed to flush");
        }

        self
    }

    pub fn selected_date_formatted(&self) -> String {
        Self::format_date(self.calendar_state.selected_date).to_owned()
    }

    pub fn save_new_file(&mut self) -> &mut Self {
        let file_path: PathBuf = [
            "entries",
            self.selected_date_formatted().as_str(),
            &self.popup_textarea_content(),
        ]
        .iter()
        .collect();

        let prefix = file_path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();
        if File::create(&file_path).is_ok() {
            let filename = String::from(file_path.file_name().unwrap().to_str().unwrap());
            let update_name = filename.clone();

            self.menu_state.listings.push(MenuListing {
                filename,
                path: file_path,
            });

            self.refresh_menu().update_selected_by_name(&update_name)
        } else {
            info!("Failed to create file");
            self
        }
    }

    pub fn input_editbox(&mut self, input: tui_textarea::Input) -> &mut Self {
        self.editbox_textarea.input(input);
        self
    }

    pub fn input_popup(&mut self, input: tui_textarea::Input) -> &mut Self {
        self.popup_textarea.input(input);
        self
    }

    pub fn calendar_enabled(&self) -> bool {
        self.calendar_state.enabled
    }

    pub fn select_prev_day(&mut self) -> &mut Self {
        self.update_date(-1)
    }

    pub fn select_next_day(&mut self) -> &mut Self {
        self.update_date(1)
    }

    pub fn select_next_week(&mut self) -> &mut Self {
        self.update_date(7)
    }

    pub fn select_prev_week(&mut self) -> &mut Self {
        self.update_date(-7)
    }

    fn update_date(&mut self, offset_in_days: i64) -> &mut Self {
        return if let Some(result_date) = self
            .calendar_state
            .selected_date
            .checked_add(time::Duration::days(offset_in_days))
        {
            if self.calendar_state.selected_date.month() == result_date.month() {
                self.calendar_state = CalendarState {
                    selected_date: result_date,
                    enabled: true,
                };

                // select top of list on successful date change
                self.refresh_menu().update_selected_by_index(0)
            } else {
                self
            }
        } else {
            self
        };
    }

    fn format_date(date: Date) -> String {
        let format = format_description::parse("[year]-[month]-[day]").unwrap();
        date.format(&format).unwrap()
    }

    pub fn toggle_calendar(&mut self) -> () {
        self.calendar_state.enabled = !self.calendar_state.enabled;
    }
}

#[derive(Debug, Default, Clone)]
pub struct ModelState<'a> {
    pub running_state: RunningState,
    active_window: ActiveWindow,
    calendar_state: CalendarState,
    menu_state: MenuState,

    // would ideally be char buffers
    pub editbox_textarea: TextArea<'a>,
    pub popup_textarea: TextArea<'a>,
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub enum ActiveWindow {
    #[default]
    Menu,
    EditBox,
    TextPopup,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Default, Clone)]
pub struct MenuState {
    selected_file_idx: i8,
    listings: Vec<MenuListing>,
}

#[derive(Debug, Default, Clone)]
pub struct MenuListing {
    path: PathBuf,
    pub filename: String,
}