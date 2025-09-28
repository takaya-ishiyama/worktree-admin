mod app;
mod gitignore;
mod ui;
mod worktree;

use anyhow::Result;

fn main() -> Result<()> {
    color_eyre::install().unwrap();
    let terminal = ratatui::init();
    let app = app::App::new()?;
    let result = app.run(terminal);
    ratatui::restore();
    result
}
