use ratatui::{
    layout::*,
    prelude::*,
    style::{Color, Style},
    widgets::{
        calendar::{CalendarEventStore, Monthly},
        Block, BorderType, Borders, Paragraph,
    },
};

use crate::model::{ActiveWindow, ModelState};
const MENU_LISTING_HEIGHT: u16 = 2;

// ideally would be a pure function, but assigning blocks to TextAreas mutates the model
pub fn view(model: &mut ModelState, f: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
        .split(f.size());

    let mut textarea_block = Block::default().borders(Borders::ALL).title("textArea");

    let mut menu_block = Block::default().borders(Borders::ALL);

    match model.active_window() {
        ActiveWindow::Menu => {
            menu_block = menu_block.border_style(Style::default().fg(Color::LightBlue));
        }
        ActiveWindow::EditBox => {
            textarea_block = textarea_block.border_style(Style::default().fg(Color::LightBlue));
        }
        ActiveWindow::TextPopup => {
            let textarea_popup_area = Rect {
                width: 20,
                height: 4,
                x: 10,
                y: MENU_LISTING_HEIGHT * ((model.listings().len() as u16) + 1),
            };

            let popup_textarea_block = Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::LightBlue));
            model.popup_textarea.set_block(popup_textarea_block);
            f.render_widget(model.popup_textarea.widget(), textarea_popup_area);
        }
    };

    model.editbox_textarea.set_block(textarea_block);

    let limit_in_view: usize = (layout[0].as_size().height / MENU_LISTING_HEIGHT - 1).into();

    let menu_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(MENU_LISTING_HEIGHT); limit_in_view])
        .split(menu_block.inner(layout[0]));

    for (idx, menu_listing) in model.listings().iter().enumerate() {
        // info!("Rendering listing {:?} in view", menu_listing);
        let mut entry_block = Block::new();
        if model.selected_file_idx() == idx {
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
        date = model.selected_date_formatted(),
        listings = model.num_menu_listings()
    ));
    f.render_widget(menu_block, layout[0]);
    f.render_widget(model.editbox_textarea.widget(), layout[1]);

    if model.calendar_enabled() {
        let mut cal_event_store =
            CalendarEventStore::today(Style::new().light_blue().add_modifier(Modifier::UNDERLINED));

        cal_event_store.add(model.selected_date(), Style::new().yellow().bold());

        let area = Rect {
            width: 30,
            height: 12,
            x: 5,
            y: f.size().height - 12,
        };

        let cal_block = Block::bordered().border_type(BorderType::Rounded);

        let cal = Monthly::new(model.selected_date(), cal_event_store)
            .block(cal_block.clone())
            .show_month_header(Style::new().bold());

        let inner_area = cal_block.inner(area);

        f.render_widget(cal, inner_area);
    }
}
