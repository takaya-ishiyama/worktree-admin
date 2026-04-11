use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

pub(super) fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = app
        .worktrees
        .iter()
        .map(|wt| {
            let path = wt.path.display().to_string();
            let branch = if wt.is_detached {
                format!("(detached HEAD)")
            } else {
                format!("({})", wt.branch)
            };
            let commit = &wt.commit[..8.min(wt.commit.len())];

            let content = Line::from(vec![
                Span::styled(
                    wt.name(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(branch, Style::default().fg(Color::Green)),
                Span::raw(" "),
                Span::styled(commit, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(path, Style::default().fg(Color::Cyan)),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Worktrees"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.worktree_list_state);
}
