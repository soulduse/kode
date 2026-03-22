use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::{fs, io};

/// Tree entry representing a visible item in the file explorer.
#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub path: PathBuf,
    pub name: String,
    pub depth: usize,
    pub is_dir: bool,
    pub expanded: bool,
}

/// Input mode for create/rename operations.
#[derive(Debug, Clone)]
pub enum InputMode {
    Create { parent_path: PathBuf, is_dir: bool },
    Rename { entry_index: usize, original_path: PathBuf },
}

/// File explorer state — a flat list of visible tree entries.
pub struct FileExplorer {
    pub id: usize,
    pub root: PathBuf,
    pub entries: Vec<TreeEntry>,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub input_mode: Option<InputMode>,
    pub input_buffer: String,
    pub confirm_delete: Option<usize>,
    expanded_dirs: HashSet<PathBuf>,
}

const HIDDEN_PREFIXES: &[&str] = &[".", "target", "node_modules", "__pycache__", ".git"];

impl FileExplorer {
    pub fn new(id: usize, root: PathBuf) -> Self {
        let mut explorer = Self {
            id,
            root: root.clone(),
            entries: Vec::new(),
            cursor: 0,
            scroll_offset: 0,
            input_mode: None,
            input_buffer: String::new(),
            confirm_delete: None,
            expanded_dirs: HashSet::new(),
        };
        explorer.rebuild_entries();
        explorer
    }

    /// Rebuild the flat entry list from the filesystem.
    pub fn rebuild_entries(&mut self) {
        self.entries.clear();
        self.build_tree(&self.root.clone(), 0);
        // Clamp cursor
        if !self.entries.is_empty() && self.cursor >= self.entries.len() {
            self.cursor = self.entries.len() - 1;
        }
    }

    fn build_tree(&mut self, dir: &Path, depth: usize) {
        let items = match read_dir_sorted(dir) {
            Ok(items) => items,
            Err(_) => return,
        };

        for (path, name, is_dir) in items {
            if should_hide(&name) {
                continue;
            }

            let expanded = is_dir && self.expanded_dirs.contains(&path);
            self.entries.push(TreeEntry {
                path: path.clone(),
                name,
                depth,
                is_dir,
                expanded,
            });

            if expanded {
                self.build_tree(&path, depth + 1);
            }
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_cursor_down(&mut self) {
        if !self.entries.is_empty() && self.cursor < self.entries.len() - 1 {
            self.cursor += 1;
        }
    }

    /// Toggle expand/collapse for a directory.
    pub fn toggle_expand(&mut self) {
        if let Some(entry) = self.entries.get(self.cursor) {
            if entry.is_dir {
                let path = entry.path.clone();
                if self.expanded_dirs.contains(&path) {
                    self.expanded_dirs.remove(&path);
                } else {
                    self.expanded_dirs.insert(path);
                }
                self.rebuild_entries();
            }
        }
    }

    /// Collapse current dir, or jump to parent if on a file.
    pub fn collapse_current(&mut self) {
        if let Some(entry) = self.entries.get(self.cursor) {
            if entry.is_dir && entry.expanded {
                let path = entry.path.clone();
                self.expanded_dirs.remove(&path);
                self.rebuild_entries();
                return;
            }

            // Jump to parent directory
            if let Some(parent) = entry.path.parent() {
                let parent = parent.to_path_buf();
                if let Some(idx) = self.entries.iter().position(|e| e.path == parent) {
                    self.cursor = idx;
                }
            }
        }
    }

    /// Get the currently selected path.
    pub fn selected_path(&self) -> Option<&Path> {
        self.entries.get(self.cursor).map(|e| e.path.as_path())
    }

    /// Get the currently selected entry.
    pub fn selected_entry(&self) -> Option<&TreeEntry> {
        self.entries.get(self.cursor)
    }

    /// Start file creation prompt.
    pub fn start_create(&mut self, is_dir: bool) {
        let parent = if let Some(entry) = self.entries.get(self.cursor) {
            if entry.is_dir {
                entry.path.clone()
            } else {
                entry.path.parent().unwrap_or(&self.root).to_path_buf()
            }
        } else {
            self.root.clone()
        };

        self.input_mode = Some(InputMode::Create {
            parent_path: parent,
            is_dir,
        });
        self.input_buffer.clear();
    }

    /// Start rename prompt.
    pub fn start_rename(&mut self) {
        if let Some(entry) = self.entries.get(self.cursor) {
            let original_path = entry.path.clone();
            let name = entry.name.clone();
            self.input_mode = Some(InputMode::Rename {
                entry_index: self.cursor,
                original_path,
            });
            self.input_buffer = name;
        }
    }

    /// Confirm and execute the current input operation.
    /// Returns the path of the created/renamed item if it's a file.
    pub fn confirm_input(&mut self) -> io::Result<Option<PathBuf>> {
        let mode = match self.input_mode.take() {
            Some(m) => m,
            None => return Ok(None),
        };

        let name = self.input_buffer.clone();
        self.input_buffer.clear();

        if name.is_empty() {
            return Ok(None);
        }

        match mode {
            InputMode::Create { parent_path, is_dir } => {
                let new_path = parent_path.join(&name);
                if is_dir {
                    fs::create_dir_all(&new_path)?;
                    self.expanded_dirs.insert(parent_path);
                    self.rebuild_entries();
                    Ok(None)
                } else {
                    if let Some(parent) = new_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&new_path, "")?;
                    self.expanded_dirs.insert(parent_path);
                    self.rebuild_entries();
                    Ok(Some(new_path))
                }
            }
            InputMode::Rename { original_path, .. } => {
                let new_path = original_path.parent().unwrap_or(Path::new(".")).join(&name);
                fs::rename(&original_path, &new_path)?;
                self.rebuild_entries();
                if new_path.is_file() {
                    Ok(Some(new_path))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Cancel the current input operation.
    pub fn cancel_input(&mut self) {
        self.input_mode = None;
        self.input_buffer.clear();
        self.confirm_delete = None;
    }

    /// Request delete confirmation for the selected entry.
    pub fn request_delete(&mut self) {
        if !self.entries.is_empty() {
            self.confirm_delete = Some(self.cursor);
        }
    }

    /// Execute the pending delete operation.
    pub fn confirm_delete_yes(&mut self) -> io::Result<()> {
        if let Some(idx) = self.confirm_delete.take() {
            if let Some(entry) = self.entries.get(idx) {
                let path = entry.path.clone();
                if entry.is_dir {
                    fs::remove_dir_all(&path)?;
                } else {
                    fs::remove_file(&path)?;
                }
                self.rebuild_entries();
            }
        }
        Ok(())
    }

    /// Ensure cursor is visible by adjusting scroll.
    pub fn ensure_cursor_visible(&mut self, visible_lines: usize) {
        if visible_lines == 0 {
            return;
        }
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + visible_lines {
            self.scroll_offset = self.cursor - visible_lines + 1;
        }
    }
}

/// Read a directory and return sorted entries (dirs first, then files, alphabetical).
fn read_dir_sorted(dir: &Path) -> io::Result<Vec<(PathBuf, String, bool)>> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_dir = path.is_dir();

        if is_dir {
            dirs.push((path, name, true));
        } else {
            files.push((path, name, false));
        }
    }

    dirs.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
    files.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));

    dirs.extend(files);
    Ok(dirs)
}

fn should_hide(name: &str) -> bool {
    HIDDEN_PREFIXES.iter().any(|p| name == *p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn new_explorer_reads_current_dir() {
        let dir = std::env::temp_dir().join("kode_test_explorer_state");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("Cargo.toml"), "").unwrap();
        fs::write(dir.join("src/main.rs"), "").unwrap();

        let explorer = FileExplorer::new(0, dir.clone());
        assert!(!explorer.entries.is_empty());

        let names: Vec<&str> = explorer.entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"src"));
        assert!(names.contains(&"Cargo.toml"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn expand_collapse() {
        let dir = std::env::temp_dir().join("kode_test_expand_state");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/lib.rs"), "").unwrap();

        let mut explorer = FileExplorer::new(0, dir.clone());
        let initial_count = explorer.entries.len();

        let src_idx = explorer.entries.iter().position(|e| e.name == "src").unwrap();
        explorer.cursor = src_idx;
        explorer.toggle_expand();

        assert!(explorer.entries.len() > initial_count);
        assert!(explorer.entries.iter().any(|e| e.name == "lib.rs"));

        let src_idx = explorer.entries.iter().position(|e| e.name == "src").unwrap();
        explorer.cursor = src_idx;
        explorer.toggle_expand();

        assert_eq!(explorer.entries.len(), initial_count);

        let _ = fs::remove_dir_all(&dir);
    }
}
