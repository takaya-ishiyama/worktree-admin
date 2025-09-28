use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, Paragraph, Wrap,
    },
    Frame,
};
use crate::app::{App, AppMode, InputMode};

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
    
    render_header(app, frame, chunks[0]);
    render_main_content(app, frame, chunks[1]);
    render_footer(app, frame, chunks[2]);
    
    match app.mode {
        AppMode::CreateWorktree => render_create_dialog(app, frame),
        AppMode::SelectIgnoredFiles => render_ignored_files_dialog(app, frame),
        AppMode::Error => render_error_dialog(app, frame),
        _ => {}
    }
}

fn render_header(_app: &App, frame: &mut Frame, area: Rect) {
    let header = Paragraph::new("Worktree Admin - GitHub Worktree Management TUI")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, area);
}

fn render_main_content(app: &mut App, frame: &mut Frame, area: Rect) {
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
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(branch, Style::default().fg(Color::Green)),
                Span::raw(" "),
                Span::styled(commit, Style::default().fg(Color::DarkGray)),
                Span::raw(" "),
                Span::styled(path, Style::default().fg(Color::Blue)),
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

fn render_footer(app: &App, frame: &mut Frame, area: Rect) {
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

fn render_create_dialog(app: &mut App, frame: &mut Frame) {
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
        .block(Block::default().borders(Borders::ALL).title("Path (optional)"));
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
            .block(Block::default().borders(Borders::ALL).title("Available Branches"))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("> ");
        
        frame.render_stateful_widget(branches_list, chunks[3], &mut app.branch_list_state);
    }
}

fn render_ignored_files_dialog(app: &mut App, frame: &mut Frame) {
    let area = centered_rect(70, 70, frame.area());
    frame.render_widget(Clear, area);
    
    let block = Block::default()
        .title("Select Ignored Files to Copy")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(inner);
    
    let items: Vec<ListItem> = app
        .ignored_items
        .iter()
        .map(|item| {
            let checkbox = if item.selected { "[x]" } else { "[ ]" };
            let name = item.name();
            let path = item.relative_path(&app.repo_path);
            
            let style = if item.is_dir {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };
            
            ListItem::new(Line::from(vec![
                Span::raw(format!("{} ", checkbox)),
                Span::styled(name, style.add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(path, Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("> ");
    
    frame.render_stateful_widget(list, chunks[0], &mut app.ignored_files_list_state);
    
    let help_text = "[Space] Toggle  [a] Select All  [Enter] Confirm  [Esc] Cancel";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[1]);
}

fn render_error_dialog(app: &App, frame: &mut Frame) {
    let area = centered_rect(50, 20, frame.area());
    frame.render_widget(Clear, area);
    
    let block = Block::default()
        .title("Error")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));
    
    let error_text = app.error_message.as_deref().unwrap_or("Unknown error");
    let error = Paragraph::new(error_text)
        .style(Style::default().fg(Color::Red))
        .wrap(Wrap { trim: true })
        .block(block);
    
    frame.render_widget(error, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}