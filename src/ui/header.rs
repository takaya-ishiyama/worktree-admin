use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

pub(super) fn render(_app: &App, frame: &mut Frame, area: Rect) {
    let header = Paragraph::new("Worktree Admin - GitHub Worktree Management TUI")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, area);
}
