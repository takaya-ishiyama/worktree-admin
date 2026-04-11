use crate::app::{App, AppMode};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

pub(super) fn render(app: &App, frame: &mut Frame, area: Rect) {
    let keybinds = match app.mode {
        AppMode::Normal => {
            vec![
                ("↑/↓/j/k", "Navigate"),
                ("n", "New worktree"),
                ("d", "Delete"),
                ("r", "Refresh"),
                ("p", "Prune"),
                ("q", "Quit"),
            ]
        }
        AppMode::CreateWorktree => {
            vec![
                ("Tab", "Switch field"),
                ("Enter", "Next/Confirm"),
                ("Esc", "Cancel"),
            ]
        }
        AppMode::SelectIgnoredFiles => {
            vec![
                ("↑/↓", "Navigate"),
                ("Space", "Toggle selection"),
                ("a", "Select all"),
                ("Enter", "Confirm"),
                ("Esc", "Cancel"),
            ]
        }
        _ => vec![],
    };

    let help_text = keybinds
        .iter()
        .map(|(key, desc)| format!("[{}] {}", key, desc))
        .collect::<Vec<_>>()
        .join("  ");

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(help, area);
}
