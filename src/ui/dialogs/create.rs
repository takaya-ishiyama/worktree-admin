use super::centered_rect;
use crate::app::{App, InputMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = centered_rect(60, 40, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title("Create New Worktree")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(inner);

    let branch_style = if app.input_mode == InputMode::BranchName {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let branch_input = Paragraph::new(app.branch_input.as_str())
        .style(branch_style)
        .block(Block::default().borders(Borders::ALL).title("Branch Name"));
    frame.render_widget(branch_input, chunks[0]);

    let path_style = if app.input_mode == InputMode::Path {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let path_input = Paragraph::new(app.path_input.as_str())
        .style(path_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Path (optional)"),
        );
    frame.render_widget(path_input, chunks[1]);

    let create_from_style = if app.create_from_existing {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Gray)
    };

    let checkbox = Paragraph::new(format!(
        "[{}] Create from existing branch",
        if app.create_from_existing { "x" } else { " " }
    ))
    .style(create_from_style);
    frame.render_widget(checkbox, chunks[2]);

    if app.create_from_existing && !app.available_branches.is_empty() {
        let branch_items: Vec<ListItem> = app
            .available_branches
            .iter()
            .map(|b| ListItem::new(b.as_str()))
            .collect();

        let branches_list = List::new(branch_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Available Branches"),
            )
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("> ");

        frame.render_stateful_widget(branches_list, chunks[3], &mut app.branch_list_state);
    }
}
