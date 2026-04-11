mod dialogs;
mod footer;
mod header;
mod worktree_list;

use crate::app::{App, AppMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    header::render(app, frame, chunks[0]);
    worktree_list::render(app, frame, chunks[1]);
    footer::render(app, frame, chunks[2]);

    match app.mode {
        AppMode::CreateWorktree => dialogs::create::render(app, frame),
        AppMode::SelectIgnoredFiles => dialogs::ignored_files::render(app, frame),
        AppMode::Error => dialogs::error::render(app, frame),
        _ => {}
    }
}
