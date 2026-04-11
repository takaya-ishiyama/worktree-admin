use super::centered_rect;
use crate::app::App;
use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

pub fn render(app: &App, frame: &mut Frame) {
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
