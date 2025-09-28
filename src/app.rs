use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    widgets::ListState,
    DefaultTerminal,
};
use std::path::PathBuf;

use crate::worktree::{Worktree, WorktreeManager};
use crate::gitignore::{GitignoreManager, IgnoredItem};
use crate::ui;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum AppMode {
    #[default]
    Normal,
    CreateWorktree,
    SelectIgnoredFiles,
    Error,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum InputMode {
    #[default]
    BranchName,
    Path,
    SelectBranch,
}

pub struct App {
    pub running: bool,
    pub mode: AppMode,
    pub input_mode: InputMode,
    pub repo_path: PathBuf,
    
    pub worktrees: Vec<Worktree>,
    pub worktree_list_state: ListState,
    
    pub branch_input: String,
    pub path_input: String,
    pub create_from_existing: bool,
    pub available_branches: Vec<String>,
    pub branch_list_state: ListState,
    
    pub ignored_items: Vec<IgnoredItem>,
    pub ignored_files_list_state: ListState,
    
    pub error_message: Option<String>,
    
    worktree_manager: WorktreeManager,
    gitignore_manager: GitignoreManager,
    
    pending_worktree_path: Option<PathBuf>,
}

impl App {
    pub fn new() -> Result<Self> {
        let repo_path = std::env::current_dir()?;
        let worktree_manager = WorktreeManager::new(repo_path.clone());
        let gitignore_manager = GitignoreManager::new(repo_path.clone())?;
        let worktrees = worktree_manager.list()?;
        
        let mut worktree_list_state = ListState::default();
        if !worktrees.is_empty() {
            worktree_list_state.select(Some(0));
        }
        
        Ok(Self {
            running: true,
            mode: AppMode::Normal,
            input_mode: InputMode::BranchName,
            repo_path,
            worktrees,
            worktree_list_state,
            branch_input: String::new(),
            path_input: String::new(),
            create_from_existing: false,
            available_branches: Vec::new(),
            branch_list_state: ListState::default(),
            ignored_items: Vec::new(),
            ignored_files_list_state: ListState::default(),
            error_message: None,
            worktree_manager,
            gitignore_manager,
            pending_worktree_path: None,
        })
    }
    
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| ui::render(&mut self, frame))?;
            self.handle_events()?;
        }
        Ok(())
    }
    
    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                self.handle_key_event(key)?;
            }
            _ => {}
        }
        Ok(())
    }
    
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match self.mode {
            AppMode::Normal => self.handle_normal_mode(key)?,
            AppMode::CreateWorktree => self.handle_create_mode(key)?,
            AppMode::SelectIgnoredFiles => self.handle_select_ignored_mode(key)?,
            AppMode::Error => {
                self.mode = AppMode::Normal;
                self.error_message = None;
            }
        }
        Ok(())
    }
    
    fn handle_normal_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit(),
            KeyCode::Char('n') => self.start_create_worktree()?,
            KeyCode::Char('d') => self.delete_selected_worktree()?,
            KeyCode::Char('r') => self.refresh_worktrees()?,
            KeyCode::Char('p') => self.prune_worktrees()?,
            KeyCode::Up | KeyCode::Char('k') => self.move_selection_up(),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection_down(),
            _ => {}
        }
        Ok(())
    }
    
    fn handle_create_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.clear_create_inputs();
            }
            KeyCode::Tab => {
                match self.input_mode {
                    InputMode::BranchName => self.input_mode = InputMode::Path,
                    InputMode::Path => {
                        if self.create_from_existing {
                            self.input_mode = InputMode::SelectBranch;
                        } else {
                            self.input_mode = InputMode::BranchName;
                        }
                    }
                    InputMode::SelectBranch => self.input_mode = InputMode::BranchName,
                }
            }
            KeyCode::Enter => {
                if self.input_mode == InputMode::SelectBranch {
                    if let Some(selected) = self.branch_list_state.selected() {
                        self.branch_input = self.available_branches[selected].clone();
                        self.input_mode = InputMode::BranchName;
                    }
                } else if !self.branch_input.is_empty() {
                    self.create_worktree()?;
                }
            }
            KeyCode::Char(' ') if self.input_mode == InputMode::Path => {
                self.create_from_existing = !self.create_from_existing;
                if self.create_from_existing {
                    self.load_available_branches()?;
                }
            }
            KeyCode::Up if self.input_mode == InputMode::SelectBranch => {
                self.move_branch_selection_up();
            }
            KeyCode::Down if self.input_mode == InputMode::SelectBranch => {
                self.move_branch_selection_down();
            }
            KeyCode::Backspace => {
                match self.input_mode {
                    InputMode::BranchName => {
                        self.branch_input.pop();
                    }
                    InputMode::Path => {
                        self.path_input.pop();
                    }
                    _ => {}
                }
            }
            KeyCode::Char(c) => {
                match self.input_mode {
                    InputMode::BranchName => {
                        self.branch_input.push(c);
                    }
                    InputMode::Path => {
                        self.path_input.push(c);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    fn handle_select_ignored_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.ignored_items.clear();
                self.pending_worktree_path = None;
            }
            KeyCode::Enter => {
                self.copy_selected_ignored_files()?;
                self.mode = AppMode::Normal;
            }
            KeyCode::Char(' ') => self.toggle_ignored_file_selection(),
            KeyCode::Char('a') => self.toggle_all_ignored_files(),
            KeyCode::Up => self.move_ignored_selection_up(),
            KeyCode::Down => self.move_ignored_selection_down(),
            _ => {}
        }
        Ok(())
    }
    
    fn start_create_worktree(&mut self) -> Result<()> {
        self.mode = AppMode::CreateWorktree;
        self.input_mode = InputMode::BranchName;
        Ok(())
    }
    
    fn create_worktree(&mut self) -> Result<()> {
        let path = if self.path_input.is_empty() {
            None
        } else {
            Some(PathBuf::from(&self.path_input))
        };
        
        let result = if self.create_from_existing {
            self.worktree_manager.create_from_branch(
                &self.branch_input,
                path.as_deref(),
            )
        } else {
            self.worktree_manager.create(
                &self.branch_input,
                path.as_deref(),
            )
        };
        
        match result {
            Ok(_) => {
                let worktree_path = path.unwrap_or_else(|| {
                    self.repo_path.parent()
                        .unwrap_or(&self.repo_path)
                        .join(&self.branch_input)
                });
                
                self.pending_worktree_path = Some(worktree_path);
                self.load_ignored_files()?;
                
                if !self.ignored_items.is_empty() {
                    self.mode = AppMode::SelectIgnoredFiles;
                    self.ignored_files_list_state.select(Some(0));
                } else {
                    self.mode = AppMode::Normal;
                    self.refresh_worktrees()?;
                }
                
                self.clear_create_inputs();
            }
            Err(e) => {
                self.show_error(&e.to_string());
            }
        }
        
        Ok(())
    }
    
    fn delete_selected_worktree(&mut self) -> Result<()> {
        if let Some(selected) = self.worktree_list_state.selected() {
            if let Some(worktree) = self.worktrees.get(selected) {
                match self.worktree_manager.remove(&worktree.path) {
                    Ok(_) => self.refresh_worktrees()?,
                    Err(e) => self.show_error(&e.to_string()),
                }
            }
        }
        Ok(())
    }
    
    fn refresh_worktrees(&mut self) -> Result<()> {
        self.worktrees = self.worktree_manager.list()?;
        if self.worktrees.is_empty() {
            self.worktree_list_state.select(None);
        } else if self.worktree_list_state.selected().is_none() {
            self.worktree_list_state.select(Some(0));
        }
        Ok(())
    }
    
    fn prune_worktrees(&mut self) -> Result<()> {
        match self.worktree_manager.prune() {
            Ok(_) => self.refresh_worktrees()?,
            Err(e) => self.show_error(&e.to_string()),
        }
        Ok(())
    }
    
    fn load_available_branches(&mut self) -> Result<()> {
        self.available_branches = self.worktree_manager.get_branches()?;
        if !self.available_branches.is_empty() {
            self.branch_list_state.select(Some(0));
        }
        Ok(())
    }
    
    fn load_ignored_files(&mut self) -> Result<()> {
        self.ignored_items = self.gitignore_manager.get_ignored_files()?;
        Ok(())
    }
    
    fn copy_selected_ignored_files(&mut self) -> Result<()> {
        if let Some(ref target_path) = self.pending_worktree_path {
            let selected_items: Vec<IgnoredItem> = self.ignored_items
                .iter()
                .filter(|item| item.selected)
                .cloned()
                .collect();
            
            if !selected_items.is_empty() {
                match self.gitignore_manager.copy_selected_items(&selected_items, target_path) {
                    Ok(_) => {
                        self.refresh_worktrees()?;
                    }
                    Err(e) => {
                        self.show_error(&e.to_string());
                    }
                }
            } else {
                self.refresh_worktrees()?;
            }
        }
        
        self.ignored_items.clear();
        self.pending_worktree_path = None;
        Ok(())
    }
    
    fn move_selection_up(&mut self) {
        if let Some(selected) = self.worktree_list_state.selected() {
            if selected > 0 {
                self.worktree_list_state.select(Some(selected - 1));
            }
        }
    }
    
    fn move_selection_down(&mut self) {
        if let Some(selected) = self.worktree_list_state.selected() {
            if selected < self.worktrees.len() - 1 {
                self.worktree_list_state.select(Some(selected + 1));
            }
        }
    }
    
    fn move_branch_selection_up(&mut self) {
        if let Some(selected) = self.branch_list_state.selected() {
            if selected > 0 {
                self.branch_list_state.select(Some(selected - 1));
            }
        }
    }
    
    fn move_branch_selection_down(&mut self) {
        if let Some(selected) = self.branch_list_state.selected() {
            if selected < self.available_branches.len() - 1 {
                self.branch_list_state.select(Some(selected + 1));
            }
        }
    }
    
    fn move_ignored_selection_up(&mut self) {
        if let Some(selected) = self.ignored_files_list_state.selected() {
            if selected > 0 {
                self.ignored_files_list_state.select(Some(selected - 1));
            }
        }
    }
    
    fn move_ignored_selection_down(&mut self) {
        if let Some(selected) = self.ignored_files_list_state.selected() {
            if selected < self.ignored_items.len() - 1 {
                self.ignored_files_list_state.select(Some(selected + 1));
            }
        }
    }
    
    fn toggle_ignored_file_selection(&mut self) {
        if let Some(selected) = self.ignored_files_list_state.selected() {
            if let Some(item) = self.ignored_items.get_mut(selected) {
                item.selected = !item.selected;
            }
        }
    }
    
    fn toggle_all_ignored_files(&mut self) {
        let all_selected = self.ignored_items.iter().all(|item| item.selected);
        for item in &mut self.ignored_items {
            item.selected = !all_selected;
        }
    }
    
    fn clear_create_inputs(&mut self) {
        self.branch_input.clear();
        self.path_input.clear();
        self.create_from_existing = false;
        self.available_branches.clear();
        self.branch_list_state.select(None);
    }
    
    fn show_error(&mut self, message: &str) {
        self.error_message = Some(message.to_string());
        self.mode = AppMode::Error;
    }
    
    fn quit(&mut self) {
        self.running = false;
    }
}