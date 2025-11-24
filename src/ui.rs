use ratatui::{
    Frame,
    layout::{Constraint, Direction, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, canvas::Line},
};

use crate::{
    App, CurrentScreen, FocusedWindowMain,
    events::{format_ms_to_duration_string, musicplayer::PlayerStatus},
    utils::{
        input::InputMode,
        selecthandler::{SelectHandler, SelectHandlerItem},
    },
};
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
    let layout = ratatui::layout::Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Fill(1)])
        .split(rect);
    let search_rect = layout[0];
    render_search(app, frame, search_rect);
    let bottom_rect = layout[1];

    if app.upcoming_media_shown {
        let layout = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Length(30)])
            .split(bottom_rect);
        render_media_selection(app, frame, layout[0]);
        render_queue(app, frame, layout[1]);
    } else {
        render_media_selection(app, frame, rect);
    }
}

fn render_search(app: &mut App, frame: &mut Frame, rect: Rect) {
    let input = Paragraph::new(app.search_handler.get_query())
        .style(match &app.current_screen {
            CurrentScreen::Main(focused_window) => match focused_window {
                FocusedWindowMain::Search => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            },
            _ => Style::default(),
        })
        .block(Block::bordered().title("Input"));
    frame.render_widget(input, rect);
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

    let (select_handler, is_focused) = match &mut app.current_screen {
        CurrentScreen::Main(focused_window) => match focused_window {
            FocusedWindowMain::Queue => (
                &mut app.search_handler.select_handler.lock().unwrap(),
                false,
            ),
            FocusedWindowMain::Main => {
                (&mut app.search_handler.select_handler.lock().unwrap(), true)
            }
            FocusedWindowMain::Search => {
                (&mut app.search_handler.select_handler.lock().unwrap(), true)
            }
        },
        _ => (
            &mut app.search_handler.select_handler.lock().unwrap(),
            false,
        ),
    };
    let selected_index = select_handler.state().selected();
    let block_title = "Media".to_string() + if is_focused { "(*)" } else { "" };
    let media_select_block = Block::default()
        .title(block_title)
        .border_set(border_set)
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT);
    let list = List::default()
        .items(
            select_handler
                .items()
                .iter()
                .enumerate()
                .map(|(index, song)| {
                    if Some(index) == selected_index && is_focused {
                        ListItem::new(song.title.clone()).style(Style::default().bg(Color::Yellow))
                    } else {
                        ListItem::new(song.title.clone()).style(Style::default())
                    }
                })
                .collect::<Vec<ListItem>>(),
        )
        .block(media_select_block);
    frame.render_stateful_widget(list, rect, &mut select_handler.state());
}

fn render_queue(app: &mut App, frame: &mut Frame, rect: Rect) {
    let selected_queue_index = app.queue_select_handler.state().selected();
    let is_focused = match &mut app.current_screen {
        CurrentScreen::Main(focused_window) => match focused_window {
            FocusedWindowMain::Queue => true,
            FocusedWindowMain::Main => false,
            FocusedWindowMain::Search => false,
        },
        _ => false,
    };
    let block_title = "Queue".to_string() + if is_focused { "(*)" } else { "" };
    let list = List::default()
        .items(
            app.queue_select_handler
                .items()
                .iter()
                .enumerate()
                .map(|(index, song)| {
                    if Some(index) == app.player_information.playing_index {
                        if Some(index) == selected_queue_index && is_focused {
                            ListItem::new(format!("🎶 {}", song.title.clone()))
                                .style(Style::default().light_green().bg(Color::Yellow))
                        } else {
                            ListItem::new(format!("🎶 {}", song.title.clone()))
                                .style(Style::default().light_green())
                        }
                    } else {
                        if Some(index) == selected_queue_index && is_focused {
                            ListItem::new(song.title.clone())
                                .style(Style::default().bg(Color::Yellow))
                        } else {
                            ListItem::new(song.title.clone()).style(Style::default())
                        }
                    }
                })
                .collect::<Vec<ListItem>>(),
        )
        .block(
            Block::default()
                .title(block_title)
                .border_set(symbols::border::Set {
                    top_left: symbols::line::NORMAL.vertical_right,
                    bottom_right: symbols::line::NORMAL.horizontal_up,
                    ..symbols::border::PLAIN
                })
                .borders(Borders::RIGHT | Borders::TOP),
        );
    frame.render_stateful_widget(list, rect, &mut app.queue_select_handler.state());
}

fn render_media_info(app: &App, frame: &mut Frame, rect: Rect) {
    let mut paragraph = Paragraph::new("");

    if let PlayerStatus::Playing(song) = &app.player_information.status {
        let song = song.clone();
        paragraph = Paragraph::new(format!(
            "{}\n{}\nPlaying",
            song.title,
            song.author.unwrap_or("".to_string())
        ))
    }
    if let PlayerStatus::Paused(song) = &app.player_information.status {
        let song = song.clone();
        paragraph = Paragraph::new(format!(
            "{}\n{}\nPaused",
            song.title,
            song.author.unwrap_or("".to_string())
        ))
    }
    let paragraph = paragraph.block(
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
    let progress = match &app.player_information.status {
        PlayerStatus::Playing(song) => {
            format!(
                "{}/{}",
                format_ms_to_duration_string(app.player_information.passed_time),
                format_ms_to_duration_string(song.total_time as u64)
            )
        }
        PlayerStatus::Paused(song) => {
            format!(
                "{}/{}",
                format_ms_to_duration_string(app.player_information.passed_time),
                format_ms_to_duration_string(song.total_time as u64)
            )
        }
        _ => "No Audio".to_string(),
    };
    let paragraph = Paragraph::new(progress).block(
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
