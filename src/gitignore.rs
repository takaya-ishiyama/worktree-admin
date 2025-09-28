use anyhow::{Context, Result};
use glob::glob;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct IgnoredItem {
    pub path: PathBuf,
    pub is_dir: bool,
    pub selected: bool,
}

impl IgnoredItem {
    pub fn name(&self) -> String {
        let name = self
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        if self.is_dir {
            format!("{}/", name)
        } else {
            name
        }
    }

    pub fn relative_path(&self, base: &Path) -> String {
        self.path
            .strip_prefix(base)
            .unwrap_or(&self.path)
            .display()
            .to_string()
    }
}

pub struct GitignoreManager {
    repo_path: PathBuf,
}

impl GitignoreManager {
    pub fn new(repo_path: PathBuf) -> Result<Self> {
        Ok(Self { repo_path })
    }

    pub fn get_ignored_files(&self) -> Result<Vec<IgnoredItem>> {
        let mut ignored_items = Vec::new();
        let mut visited = HashSet::new();

        let gitignore_path = self.repo_path.join(".gitignore");
        if !gitignore_path.exists() {
            return Ok(ignored_items);
        }

        let content = fs::read_to_string(&gitignore_path)?;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let pattern = if line.starts_with('/') {
                format!("{}{}", self.repo_path.display(), line)
            } else {
                format!("{}/{}", self.repo_path.display(), line)
            };

            if let Ok(paths) = glob(&pattern) {
                for entry in paths.flatten() {
                    if visited.insert(entry.clone()) {
                        let is_dir = entry.is_dir();
                        ignored_items.push(IgnoredItem {
                            path: entry,
                            is_dir,
                            selected: false,
                        });
                    }
                }
            }
        }

        ignored_items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.path.cmp(&b.path),
        });

        Ok(ignored_items)
    }

    pub fn copy_selected_items(&self, items: &[IgnoredItem], target_dir: &Path) -> Result<()> {
        for item in items.iter().filter(|i| i.selected) {
            let relative_path = item
                .path
                .strip_prefix(&self.repo_path)
                .unwrap_or(&item.path);
            let target_path = target_dir.join(relative_path);

            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {:?}", parent))?;
            }

            if item.is_dir {
                copy_dir_recursive(&item.path, &target_path)?;
            } else {
                fs::copy(&item.path, &target_path).with_context(|| {
                    format!("Failed to copy file: {:?} to {:?}", item.path, target_path)
                })?;
            }
        }

        Ok(())
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
