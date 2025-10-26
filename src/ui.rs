use ratatui::{
    Frame,
    layout::{Constraint, Direction, Margin, Rect},
    style::{Style, Stylize},
    symbols,
    text::Text,
    widgets::{Block, Borders, List, Paragraph, canvas::Line},
};

use crate::App;
pub fn render(frame: &mut Frame, app: &mut App) {
    let layout = ratatui::layout::Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(6)])
        .split(frame.area());

    create_upper_rect(app, frame, layout[0]);

    let layout = ratatui::layout::Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Length(1)])
        .split(layout[1]);

    let media_rect = layout[0];
    let controls_rect = layout[1];

    render_controls(app, frame, controls_rect);

    let layout = ratatui::layout::Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Fill(1)])
        .split(media_rect);
    let media_info_rect = layout[0];
    let media_progress_rect = layout[1];

    render_media_info(app, frame, media_info_rect);
    render_media_progressbar(app, frame, media_progress_rect);
}

fn create_upper_rect(app: &mut App, frame: &mut Frame, rect: Rect) {
    if app.upcoming_media_shown {
        let layout = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Length(15)])
            .split(rect);
        render_media_selection(app, frame, layout[0]);
        render_upcoming_media(app, frame, layout[1]);
    } else {
        render_media_selection(app, frame, rect);
    }
}

fn render_media_selection(app: &mut App, frame: &mut Frame, rect: Rect) {
    let border_set = match app.upcoming_media_shown {
        true => symbols::border::Set {
            top_right: symbols::line::NORMAL.horizontal_down,

            ..symbols::border::PLAIN
        },
        false => symbols::border::Set {
            ..symbols::border::PLAIN
        },
    };
    let media_select_block = Block::default()
        .border_set(border_set)
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT);

    // Lines
    //
    let list = List::default()
        .items(["HI"])
        .style(Style::new().white())
        .highlight_style(Style::new().italic())
        .highlight_symbol(">>")
        .block(media_select_block);
    frame.render_stateful_widget(list, rect, &mut app.select_handler.state());
}

fn render_upcoming_media(app: &App, frame: &mut Frame, rect: Rect) {
    let paragraph = Paragraph::new("Upcoming Media").block(
        Block::default()
            .border_set(symbols::border::Set {
                top_left: symbols::line::NORMAL.vertical_right,
                bottom_right: symbols::line::NORMAL.horizontal_up,
                ..symbols::border::PLAIN
            })
            .borders(Borders::RIGHT | Borders::TOP),
    );
    frame.render_widget(paragraph, rect);
}

fn render_media_info(app: &App, frame: &mut Frame, rect: Rect) {
    let paragraph = Paragraph::new("Media Info").block(
        Block::default()
            .border_set(symbols::border::Set {
                top_left: symbols::line::NORMAL.vertical_right,
                top_right: symbols::line::NORMAL.horizontal_down,
                bottom_right: symbols::line::NORMAL.horizontal_up,
                ..symbols::border::PLAIN
            })
            .borders(Borders::ALL),
    );
    frame.render_widget(paragraph, rect);
}

fn render_media_progressbar(app: &App, frame: &mut Frame, rect: Rect) {
    let paragraph = Paragraph::new("Media Progress").block(
        Block::default()
            .border_set(symbols::border::Set {
                top_left: symbols::line::NORMAL.horizontal_down,
                bottom_left: symbols::line::NORMAL.vertical_left,
                top_right: symbols::line::NORMAL.vertical_left,
                ..symbols::border::PLAIN
            })
            .borders(Borders::BOTTOM | Borders::TOP | Borders::RIGHT),
    );
    frame.render_widget(paragraph, rect);
}

fn render_controls(app: &App, frame: &mut Frame, rect: Rect) {
    let paragraph = Paragraph::new("Controls");
    frame.render_widget(paragraph, rect);
}
