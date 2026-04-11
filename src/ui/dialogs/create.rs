use super::centered_rect_abs_height;
use crate::app::{App, InputMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

/// ブランチリストの最小・最大行数 (ボーダー込み)
const MIN_BRANCH_LIST_HEIGHT: u16 = 4; // ボーダー2 + 表示2行
const MAX_BRANCH_LIST_HEIGHT: u16 = 12; // ボーダー2 + 表示10行

pub fn render(app: &mut App, frame: &mut Frame) {
    let show_branches = app.create_from_existing && !app.available_branches.is_empty();

    // ブランチリストの高さ: ブランチ数 + ボーダー2行、min/max でクランプ
    let branch_list_height = if show_branches {
        (app.available_branches.len() as u16 + 2)
            .clamp(MIN_BRANCH_LIST_HEIGHT, MAX_BRANCH_LIST_HEIGHT)
    } else {
        0
    };

    // ダイアログ全体の高さ:
    //   外枠ボーダー上下2 + innerのmargin上下2 + 固定フィールド(BranchName3+Path3+Checkbox3) + リスト
    let dialog_height = 2 + 2 + 9 + branch_list_height;

    let area = centered_rect_abs_height(60, dialog_height, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title("Create New Worktree")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut constraints = vec![
        Constraint::Length(3), // Branch Name
        Constraint::Length(3), // Path
        Constraint::Length(3), // Checkbox
    ];
    if show_branches {
        constraints.push(Constraint::Length(branch_list_height));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
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

    if show_branches {
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
