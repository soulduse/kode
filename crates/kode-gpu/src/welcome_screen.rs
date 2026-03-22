use std::path::PathBuf;

use kode_core::event::{KeyCode, Modifiers};
use kode_workspace::recent_projects::{
    add_recent_project, default_recent_projects_path, display_path, load_recent_projects,
    remove_stale_projects, save_recent_projects, RecentProjectsList,
};

pub struct WelcomeScreen {
    pub recent: RecentProjectsList,
    pub selected: usize,
    pub scroll_offset: usize,
    pub hover_index: Option<usize>,
    pub cursor_pos: (f64, f64),
    recent_path: PathBuf,
}

pub enum WelcomeAction {
    None,
    OpenProject(PathBuf),
    OpenFolderPicker,
    Quit,
}

// Font sizes for the welcome screen
pub const TITLE_FONT_SIZE: f32 = 36.0;
pub const TITLE_LINE_HEIGHT: f32 = 44.0;
pub const SUBTITLE_FONT_SIZE: f32 = 16.0;
pub const SUBTITLE_LINE_HEIGHT: f32 = 22.0;
pub const BUTTON_FONT_SIZE: f32 = 16.0;
pub const BUTTON_LINE_HEIGHT: f32 = 22.0;
pub const PROJECT_NAME_FONT_SIZE: f32 = 16.0;
pub const PROJECT_NAME_LINE_HEIGHT: f32 = 22.0;
pub const PROJECT_PATH_FONT_SIZE: f32 = 13.0;
pub const PROJECT_PATH_LINE_HEIGHT: f32 = 18.0;
pub const HELP_FONT_SIZE: f32 = 12.0;
pub const HELP_LINE_HEIGHT: f32 = 16.0;
pub const HEADER_FONT_SIZE: f32 = 14.0;
pub const HEADER_LINE_HEIGHT: f32 = 20.0;

/// Pre-computed layout coordinates for rendering and hit-testing.
pub struct WelcomeLayout {
    pub title_y: f32,
    pub subtitle_y: f32,
    pub button_x: f32,
    pub button_y: f32,
    pub button_w: f32,
    pub button_h: f32,
    pub section_header_y: f32,
    pub divider_y: f32,
    pub list_start_y: f32,
    pub list_item_height: f32,
    pub content_x: f32,
    pub content_width: f32,
    pub visible_count: usize,
    pub max_bottom: f32,
    /// X position for the initial circle (left side of each project item)
    pub initial_x: f32,
    /// Width offset for project text after the initial circle
    pub text_offset: f32,
}

impl WelcomeLayout {
    pub fn compute(width: f32, height: f32, _line_height: f32) -> Self {
        let content_width = 600.0f32.min(width - 100.0);
        let content_x = (width - content_width) / 2.0;

        let title_y = height * 0.10;
        let subtitle_y = title_y + TITLE_LINE_HEIGHT + 8.0;
        let button_y = subtitle_y + SUBTITLE_LINE_HEIGHT + 32.0;
        let button_w = 260.0;
        let button_h = 44.0;
        let button_x = (width - button_w) / 2.0;

        let section_header_y = button_y + button_h + 40.0;
        let divider_y = section_header_y + HEADER_LINE_HEIGHT + 8.0;
        let list_start_y = divider_y + 12.0;

        // Each project item: name + path + generous padding
        let list_item_height = PROJECT_NAME_LINE_HEIGHT + PROJECT_PATH_LINE_HEIGHT + 16.0;
        let available = height - list_start_y - 40.0;
        let visible_count = (available / list_item_height).max(1.0) as usize;

        let initial_x = content_x;
        let text_offset = 44.0; // space for initial circle + gap

        Self {
            title_y,
            subtitle_y,
            button_x,
            button_y,
            button_w,
            button_h,
            section_header_y,
            divider_y,
            list_start_y,
            list_item_height,
            content_x,
            content_width,
            visible_count,
            max_bottom: height,
            initial_x,
            text_offset,
        }
    }

    /// Check if a point is inside the "Open Project" button.
    pub fn hit_button(&self, x: f32, y: f32) -> bool {
        x >= self.button_x
            && x <= self.button_x + self.button_w
            && y >= self.button_y
            && y <= self.button_y + self.button_h
    }

    /// Return the project index at a given y position, if any.
    pub fn hit_project(&self, y: f32, project_count: usize, scroll_offset: usize) -> Option<usize> {
        if y < self.list_start_y {
            return None;
        }
        let rel = y - self.list_start_y;
        let idx = (rel / self.list_item_height) as usize;
        let absolute = idx + scroll_offset;
        if absolute < project_count {
            Some(absolute)
        } else {
            None
        }
    }
}

impl WelcomeScreen {
    pub fn new() -> Self {
        let recent_path = default_recent_projects_path();
        let mut recent = load_recent_projects(&recent_path).unwrap_or_default();
        remove_stale_projects(&mut recent);
        // Save cleaned list
        let _ = save_recent_projects(&recent, &recent_path);

        Self {
            recent,
            selected: 0,
            scroll_offset: 0,
            hover_index: None,
            cursor_pos: (0.0, 0.0),
            recent_path,
        }
    }

    pub fn handle_key(&mut self, key: kode_core::event::KeyEvent) -> WelcomeAction {
        let count = self.recent.projects.len();

        match key.code {
            KeyCode::Char('q') => return WelcomeAction::Quit,
            KeyCode::Char('o') => return WelcomeAction::OpenFolderPicker,
            KeyCode::Char('j') | KeyCode::Down => {
                if count > 0 {
                    self.selected = (self.selected + 1).min(count - 1);
                    self.ensure_visible();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if count > 0 {
                    self.selected = self.selected.saturating_sub(1);
                    self.ensure_visible();
                }
            }
            KeyCode::Enter | KeyCode::Char('l') => {
                if count > 0 && self.selected < count {
                    let path = self.recent.projects[self.selected].path.clone();
                    return WelcomeAction::OpenProject(path);
                }
            }
            KeyCode::Char('d') => {
                if count > 0 && self.selected < count {
                    self.recent.projects.remove(self.selected);
                    if self.selected >= self.recent.projects.len() && self.selected > 0 {
                        self.selected -= 1;
                    }
                    let _ = save_recent_projects(&self.recent, &self.recent_path);
                }
            }
            KeyCode::Char('g') => {
                self.selected = 0;
                self.scroll_offset = 0;
            }
            KeyCode::Char('G') if key.modifiers.contains(Modifiers::SHIFT) => {
                if count > 0 {
                    self.selected = count - 1;
                    self.ensure_visible();
                }
            }
            _ => {}
        }
        WelcomeAction::None
    }

    pub fn handle_click(&mut self, x: f32, y: f32, layout: &WelcomeLayout) -> WelcomeAction {
        if layout.hit_button(x, y) {
            return WelcomeAction::OpenFolderPicker;
        }
        if let Some(idx) = layout.hit_project(y, self.recent.projects.len(), self.scroll_offset) {
            let path = self.recent.projects[idx].path.clone();
            return WelcomeAction::OpenProject(path);
        }
        WelcomeAction::None
    }

    pub fn update_hover(&mut self, _x: f32, y: f32, layout: &WelcomeLayout) {
        self.hover_index =
            layout.hit_project(y, self.recent.projects.len(), self.scroll_offset);
    }

    pub fn record_open(&mut self, path: &PathBuf) {
        add_recent_project(&mut self.recent, path.clone());
        let _ = save_recent_projects(&self.recent, &self.recent_path);
    }

    pub fn project_display_path(project: &kode_workspace::recent_projects::RecentProject) -> String {
        display_path(&project.path)
    }

    fn ensure_visible(&mut self) {
        // Will be adjusted when we know visible_count during render
        // For now, basic scrolling logic
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
        // Upper bound is checked in render when we have layout
    }

    pub fn ensure_visible_with_layout(&mut self, layout: &WelcomeLayout) {
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
        if self.selected >= self.scroll_offset + layout.visible_count {
            self.scroll_offset = self.selected - layout.visible_count + 1;
        }
    }
}
