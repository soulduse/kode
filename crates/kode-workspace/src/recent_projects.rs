use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const MAX_RECENT_PROJECTS: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub name: String,
    pub path: PathBuf,
    pub last_opened: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecentProjectsList {
    pub projects: Vec<RecentProject>,
}

/// Default path for recent projects file.
pub fn default_recent_projects_path() -> PathBuf {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".config")
        });
    config_dir.join("kode").join("recent_projects.json")
}

/// Load recent projects from a JSON file.
pub fn load_recent_projects(path: &Path) -> std::io::Result<RecentProjectsList> {
    let json = std::fs::read_to_string(path)?;
    serde_json::from_str(&json)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Save recent projects to a JSON file.
pub fn save_recent_projects(list: &RecentProjectsList, path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(list)?;
    std::fs::write(path, json)
}

/// Add or update a project in the recent list. Moves it to the top and caps at MAX entries.
pub fn add_recent_project(list: &mut RecentProjectsList, project_path: PathBuf) {
    // Remove existing entry with the same path
    list.projects.retain(|p| p.path != project_path);

    let name = project_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| project_path.to_string_lossy().to_string());

    let now = chrono_now();

    list.projects.insert(
        0,
        RecentProject {
            name,
            path: project_path,
            last_opened: now,
        },
    );

    list.projects.truncate(MAX_RECENT_PROJECTS);
}

/// Remove projects whose paths no longer exist on disk.
pub fn remove_stale_projects(list: &mut RecentProjectsList) {
    list.projects.retain(|p| p.path.exists());
}

/// Simple ISO 8601 timestamp without external crate.
fn chrono_now() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Convert to rough datetime components
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Days since epoch to Y-M-D (simplified, no leap second handling)
    let mut y = 1970i64;
    let mut remaining_days = days as i64;
    loop {
        let year_days = if is_leap(y) { 366 } else { 365 };
        if remaining_days < year_days {
            break;
        }
        remaining_days -= year_days;
        y += 1;
    }
    let month_days: [i64; 12] = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 0usize;
    for (i, &md) in month_days.iter().enumerate() {
        if remaining_days < md {
            m = i;
            break;
        }
        remaining_days -= md;
    }
    let d = remaining_days + 1;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y,
        m + 1,
        d,
        hours,
        minutes,
        seconds
    )
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

/// Format a path for display, replacing home dir with ~
pub fn display_path(path: &Path) -> String {
    if let Ok(home) = std::env::var("HOME") {
        let home_path = Path::new(&home);
        if let Ok(stripped) = path.strip_prefix(home_path) {
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn add_and_cap_recent_projects() {
        let mut list = RecentProjectsList::default();
        for i in 0..25 {
            add_recent_project(&mut list, PathBuf::from(format!("/project/{}", i)));
        }
        assert_eq!(list.projects.len(), MAX_RECENT_PROJECTS);
        assert_eq!(list.projects[0].path, PathBuf::from("/project/24"));
    }

    #[test]
    fn add_duplicate_moves_to_top() {
        let mut list = RecentProjectsList::default();
        add_recent_project(&mut list, PathBuf::from("/a"));
        add_recent_project(&mut list, PathBuf::from("/b"));
        add_recent_project(&mut list, PathBuf::from("/a"));
        assert_eq!(list.projects.len(), 2);
        assert_eq!(list.projects[0].path, PathBuf::from("/a"));
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("recent.json");

        let mut list = RecentProjectsList::default();
        add_recent_project(&mut list, PathBuf::from("/test/project"));

        save_recent_projects(&list, &path).unwrap();
        let loaded = load_recent_projects(&path).unwrap();
        assert_eq!(loaded.projects.len(), 1);
        assert_eq!(loaded.projects[0].name, "project");
    }

    #[test]
    fn remove_stale() {
        let dir = TempDir::new().unwrap();
        let existing = dir.path().to_path_buf();
        let missing = PathBuf::from("/nonexistent/path/xyz");

        let mut list = RecentProjectsList::default();
        add_recent_project(&mut list, existing.clone());
        add_recent_project(&mut list, missing);
        assert_eq!(list.projects.len(), 2);

        remove_stale_projects(&mut list);
        assert_eq!(list.projects.len(), 1);
        assert_eq!(list.projects[0].path, existing);
    }
}
