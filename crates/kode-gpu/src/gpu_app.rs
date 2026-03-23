use std::sync::Arc;

use glyphon::Color;
use kode_core::event::{KeyCode, Modifiers};
use kode_keymap::mode::Mode;
use kode_state::{AppState, create_app_view};
use kode_state::file_explorer::InputMode;
use kode_terminal::input::key_to_escape;
use kode_workspace::pane::PaneContent;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

use crate::input::translate_winit_key;
use crate::project::build_editor_state;
use crate::rect_pipeline::{RectInstance, RectPipeline};
use crate::surface::GpuSurface;
use crate::text_render::{KodeTextRenderer, PreparedTextArea};
use crate::welcome_screen::{
    WelcomeAction, WelcomeLayout, WelcomeScreen, BUTTON_FONT_SIZE, BUTTON_LINE_HEIGHT,
    HEADER_FONT_SIZE, HEADER_LINE_HEIGHT, HELP_FONT_SIZE, HELP_LINE_HEIGHT,
    PROJECT_NAME_FONT_SIZE, PROJECT_NAME_LINE_HEIGHT, PROJECT_PATH_FONT_SIZE,
    PROJECT_PATH_LINE_HEIGHT, SUBTITLE_FONT_SIZE, SUBTITLE_LINE_HEIGHT, TITLE_FONT_SIZE,
    TITLE_LINE_HEIGHT,
};

// Catppuccin Mocha colors — linearized for sRGB framebuffer
// GPU applies linear→sRGB gamma, so we store pow(sRGB/255, 2.2) values.
// This ensures the final screen output matches the intended hex colors.
const fn srgb(r: u8, g: u8, b: u8) -> [f32; 4] {
    // pow(x, 2.2) approximation via pow(x, 2) * pow(x, 0.2) ≈ x*x for dark values
    // For accuracy we precompute: linear = (sRGB/255)^2.2
    // Using x^2 as an approximation (close enough for dark theme colors)
    let rf = (r as f32) / 255.0;
    let gf = (g as f32) / 255.0;
    let bf = (b as f32) / 255.0;
    [rf * rf, gf * gf, bf * bf, 1.0]
}

const BASE: [f32; 4] = srgb(30, 30, 46);       // #1e1e2e
const MANTLE: [f32; 4] = srgb(24, 24, 37);      // #181825
const CRUST: [f32; 4] = srgb(17, 17, 27);       // #11111b
const SURFACE0: [f32; 4] = srgb(49, 50, 68);    // #313244
const SURFACE1: [f32; 4] = srgb(69, 71, 90);    // #45475a
const TEXT_COLOR: [f32; 4] = srgb(205, 214, 244); // #cdd6f4
const BLUE: [f32; 4] = srgb(137, 180, 250);     // #89b4fa
const GREEN: [f32; 4] = srgb(166, 227, 161);    // #a6e3a1
const YELLOW: [f32; 4] = srgb(249, 226, 175);   // #f9e2af
const MAUVE: [f32; 4] = srgb(203, 166, 247);    // #cba6f7
const RED: [f32; 4] = srgb(243, 139, 168);      // #f38ba8
const OVERLAY0: [f32; 4] = srgb(108, 112, 134);  // #6c7086
const OVERLAY1: [f32; 4] = srgb(127, 132, 156);  // #7f849c
const SUBTEXT0: [f32; 4] = srgb(166, 173, 200);  // #a6adc8

// Editor UI layout constants
const TAB_BAR_HEIGHT: f32 = 38.0;
const STATUS_BAR_HEIGHT: f32 = 28.0;
const EDITOR_FONT_SIZE: f32 = 14.0;
const EDITOR_LINE_HEIGHT: f32 = 22.0;
const LINE_NUM_FONT_SIZE: f32 = 12.0;
const LINE_NUM_LINE_HEIGHT: f32 = 22.0;
const EXPLORER_FONT_SIZE: f32 = 13.0;
const EXPLORER_LINE_HEIGHT: f32 = 26.0;
const EXPLORER_HEADER_FONT_SIZE: f32 = 11.0;
const EXPLORER_HEADER_LINE_HEIGHT: f32 = 16.0;
const TAB_FONT_SIZE: f32 = 13.0;
const TAB_LINE_HEIGHT: f32 = 18.0;
const STATUS_FONT_SIZE: f32 = 12.0;
const STATUS_LINE_HEIGHT: f32 = 16.0;
const PANE_TITLE_FONT_SIZE: f32 = 12.0;
const PANE_TITLE_LINE_HEIGHT: f32 = 16.0;
const GUTTER_CHARS: f32 = 5.0;
const GUTTER_PADDING: f32 = 12.0;

fn to_glyphon_color(c: [f32; 4]) -> Color {
    // Our color constants are in linear space (for sRGB framebuffer).
    // glyphon expects sRGB values, so convert linear→sRGB: sqrt(x) ≈ pow(x, 1/2.2)
    let to_srgb = |x: f32| -> u8 { (x.sqrt().clamp(0.0, 1.0) * 255.0) as u8 };
    Color::rgba(to_srgb(c[0]), to_srgb(c[1]), to_srgb(c[2]), (c[3] * 255.0) as u8)
}

/// Basic line-level syntax coloring.
fn syntax_line_color(line: &str) -> [f32; 4] {
    let trimmed = line.trim();
    // Comments
    if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") || trimmed.starts_with('*') {
        return OVERLAY0;
    }
    // String-heavy lines
    if trimmed.starts_with('"') || trimmed.starts_with('\'') || trimmed.starts_with("```") {
        return GREEN;
    }
    // Import/use/include statements
    if trimmed.starts_with("import ") || trimmed.starts_with("use ") || trimmed.starts_with("from ") || trimmed.starts_with("require") || trimmed.starts_with("include") {
        return MAUVE;
    }
    // Annotations/decorators
    if trimmed.starts_with('@') {
        return YELLOW;
    }
    // Keywords at start of line
    let keywords = ["fun ", "fn ", "func ", "def ", "class ", "struct ", "enum ", "interface ", "trait ",
        "pub ", "private ", "protected ", "internal ", "open ", "abstract ", "override ",
        "val ", "var ", "let ", "const ", "static ", "return ", "if ", "else ", "for ", "while ",
        "match ", "when ", "switch ", "case ", "package ", "module "];
    for kw in &keywords {
        if trimmed.starts_with(kw) {
            return BLUE;
        }
    }
    TEXT_COLOR
}

/// File extension to icon mapping for explorer.
fn file_icon(name: &str, is_dir: bool, expanded: bool) -> &'static str {
    if is_dir {
        return if expanded { "📂" } else { "📁" };
    }
    match name.rsplit('.').next().unwrap_or("") {
        "kt" | "kts" => "🇰",
        "java" => "☕",
        "rs" => "🦀",
        "py" => "🐍",
        "js" => "📜",
        "ts" | "tsx" => "🔷",
        "json" => "{ }",
        "xml" => "📋",
        "md" => "📝",
        "yml" | "yaml" => "⚙️",
        "toml" => "⚙️",
        "gradle" | "groovy" => "🐘",
        "sh" | "bash" | "zsh" => "🐚",
        "sql" => "🗃️",
        "html" | "htm" => "🌐",
        "css" | "scss" => "🎨",
        "png" | "jpg" | "svg" | "gif" => "🖼️",
        "lock" => "🔒",
        "properties" => "🔧",
        _ => "📄",
    }
}

/// The two screens of the application.
pub enum AppScreen {
    Welcome(WelcomeScreen),
    Editor(AppState),
}

/// In-file search state.
struct SearchState {
    active: bool,
    query: String,
    matches: Vec<(usize, usize, usize)>, // (line, start_col, end_col)
    current_match: usize,
}

impl SearchState {
    fn new() -> Self {
        Self { active: false, query: String::new(), matches: Vec::new(), current_match: 0 }
    }
}

/// GPU application state implementing winit's ApplicationHandler.
pub struct GpuApp {
    pub screen: AppScreen,
    window: Option<Arc<Window>>,
    surface: Option<GpuSurface>,
    rect_pipeline: Option<RectPipeline>,
    text_renderer: Option<KodeTextRenderer>,
    modifiers: winit::event::Modifiers,
    scale_factor: f64,
    cursor_pos: (f64, f64),
    should_quit: bool,
    search: SearchState,
    ime_composing: bool,
    ime_preedit: String,
}

impl GpuApp {
    pub fn new(screen: AppScreen) -> Self {
        Self {
            screen,
            window: None,
            surface: None,
            rect_pipeline: None,
            text_renderer: None,
            modifiers: winit::event::Modifiers::default(),
            scale_factor: 1.0,
            cursor_pos: (0.0, 0.0),
            should_quit: false,
            ime_composing: false,
            ime_preedit: String::new(),
            search: SearchState::new(),
        }
    }

    fn render(&mut self) {
        match &self.screen {
            AppScreen::Welcome(_) => self.render_welcome(),
            AppScreen::Editor(_) => self.render_editor(),
        }
    }

    // ─────────────────────────────────────────────
    //  Welcome Screen Rendering
    // ─────────────────────────────────────────────

    fn render_welcome(&mut self) {
        let surface = match self.surface.as_ref() {
            Some(s) => s,
            None => return,
        };

        let output = match surface.surface.get_current_texture() {
            Ok(tex) => tex,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                let (w, h) = surface.size;
                if let Some(s) = self.surface.as_mut() {
                    s.resize(w, h);
                }
                return;
            }
            Err(_) => return,
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let (width, height) = surface.size;
        let width_f = width as f32;
        let height_f = height as f32;

        if let Some(rect_pl) = &self.rect_pipeline {
            rect_pl.update_screen_size(&surface.queue, width_f, height_f);
        }

        let text_r = self.text_renderer.as_mut().unwrap();
        let line_h = text_r.line_height;

        let ws = match &mut self.screen {
            AppScreen::Welcome(ws) => ws,
            _ => return,
        };

        let layout = WelcomeLayout::compute(width_f, height_f, line_h);
        ws.ensure_visible_with_layout(&layout);

        let mut rects = Vec::new();
        let mut text_areas: Vec<PreparedTextArea> = Vec::new();

        // ===== Background accent: subtle gradient-like panel behind content =====
        let panel_top = layout.title_y - 20.0;
        let panel_bottom = layout.button_y + layout.button_h + 20.0;
        rects.push(RectInstance::flat([layout.content_x - 40.0, panel_top], [layout.content_width + 80.0, panel_bottom - panel_top], [BASE[0], BASE[1], BASE[2], 0.4]));

        // ===== "Open Project" button — BLUE background, more prominent =====
        rects.push(RectInstance::flat([layout.button_x, layout.button_y], [layout.button_w, layout.button_h], BLUE));

        // Divider line
        rects.push(RectInstance::flat([layout.content_x, layout.divider_y], [layout.content_width, 1.0], SURFACE1));

        // Selected project highlight — rounded feel via slightly inset
        let project_count = ws.recent.projects.len();
        if project_count > 0 && ws.selected < project_count {
            let vis_idx = ws.selected.saturating_sub(ws.scroll_offset);
            let sel_y = layout.list_start_y + vis_idx as f32 * layout.list_item_height;
            if sel_y + layout.list_item_height <= layout.max_bottom {
                rects.push(RectInstance::flat([layout.content_x - 12.0, sel_y], [layout.content_width + 24.0, layout.list_item_height], SURFACE0));
                // Left accent bar for selected item
                rects.push(RectInstance::flat([layout.content_x - 12.0, sel_y + 4.0], [3.0, layout.list_item_height - 8.0], BLUE));
            }
        }

        // Hover highlight
        if let Some(hover_idx) = ws.hover_index {
            if hover_idx != ws.selected && hover_idx < project_count {
                let hover_vis = hover_idx.saturating_sub(ws.scroll_offset);
                let hover_y =
                    layout.list_start_y + hover_vis as f32 * layout.list_item_height;
                if hover_y + layout.list_item_height <= layout.max_bottom {
                    rects.push(RectInstance::flat([layout.content_x - 12.0, hover_y], [layout.content_width + 24.0, layout.list_item_height], [SURFACE0[0], SURFACE0[1], SURFACE0[2], 0.4]));
                }
            }
        }

        // ===== Initial circles for each visible project =====
        // Color palette for initials
        let initial_colors: &[[f32; 4]] = &[BLUE, GREEN, MAUVE, YELLOW,
            [0.976, 0.616, 0.478, 1.0], // Peach
            [0.537, 0.706, 0.980, 1.0], // Blue
            [0.796, 0.651, 0.969, 1.0], // Mauve
        ];
        if project_count > 0 {
            let visible_end =
                (ws.scroll_offset + layout.visible_count).min(project_count);
            for i in ws.scroll_offset..visible_end {
                let vis_idx = i - ws.scroll_offset;
                let item_y =
                    layout.list_start_y + vis_idx as f32 * layout.list_item_height;
                if item_y + layout.list_item_height > layout.max_bottom {
                    break;
                }
                // Circle background for initial letter
                let circle_size = 32.0;
                let circle_x = layout.initial_x;
                let circle_y = item_y + (layout.list_item_height - circle_size) / 2.0;
                let color_idx = i % initial_colors.len();
                rects.push(RectInstance::flat([circle_x, circle_y], [circle_size, circle_size], initial_colors[color_idx]));
            }
        }

        // ===== Text =====

        // Title "kode" — large, centered
        let title = "< kode />";
        let title_cell_w = TITLE_FONT_SIZE * 0.6; // approximate monospace ratio
        let title_buf =
            text_r.create_buffer_with_size(title, width_f, TITLE_FONT_SIZE, TITLE_LINE_HEIGHT);
        let title_w = title.len() as f32 * title_cell_w;
        text_areas.push(PreparedTextArea {
            buffer: title_buf,
            left: (width_f - title_w) / 2.0,
            top: layout.title_y,
            bounds_left: 0.0,
            bounds_top: 0.0,
            bounds_right: width_f,
            bounds_bottom: height_f,
            color: to_glyphon_color(BLUE),
        });

        // Subtitle — medium, centered
        let subtitle = "A Rust-native IDE";
        let sub_cell_w = SUBTITLE_FONT_SIZE * 0.6;
        let subtitle_buf = text_r.create_buffer_with_size(
            subtitle,
            width_f,
            SUBTITLE_FONT_SIZE,
            SUBTITLE_LINE_HEIGHT,
        );
        let subtitle_w = subtitle.len() as f32 * sub_cell_w;
        text_areas.push(PreparedTextArea {
            buffer: subtitle_buf,
            left: (width_f - subtitle_w) / 2.0,
            top: layout.subtitle_y,
            bounds_left: 0.0,
            bounds_top: 0.0,
            bounds_right: width_f,
            bounds_bottom: height_f,
            color: to_glyphon_color(TEXT_COLOR),
        });

        // Button text — white on blue, centered
        let btn_text = "Open Project  (o)";
        let btn_cell_w = BUTTON_FONT_SIZE * 0.6;
        let btn_buf = text_r.create_buffer_with_size(
            btn_text,
            layout.button_w,
            BUTTON_FONT_SIZE,
            BUTTON_LINE_HEIGHT,
        );
        let btn_text_w = btn_text.len() as f32 * btn_cell_w;
        text_areas.push(PreparedTextArea {
            buffer: btn_buf,
            left: layout.button_x + (layout.button_w - btn_text_w) / 2.0,
            top: layout.button_y + (layout.button_h - BUTTON_LINE_HEIGHT) / 2.0,
            bounds_left: layout.button_x,
            bounds_top: layout.button_y,
            bounds_right: layout.button_x + layout.button_w,
            bounds_bottom: layout.button_y + layout.button_h,
            color: to_glyphon_color(CRUST),
        });

        // Section header
        if project_count > 0 {
            let header = "Recent Projects";
            let header_buf = text_r.create_buffer_with_size(
                header,
                layout.content_width,
                HEADER_FONT_SIZE,
                HEADER_LINE_HEIGHT,
            );
            text_areas.push(PreparedTextArea {
                buffer: header_buf,
                left: layout.content_x,
                top: layout.section_header_y,
                bounds_left: 0.0,
                bounds_top: 0.0,
                bounds_right: width_f,
                bounds_bottom: height_f,
                color: to_glyphon_color(OVERLAY0),
            });

            // Project list
            let visible_end =
                (ws.scroll_offset + layout.visible_count).min(project_count);
            for i in ws.scroll_offset..visible_end {
                let vis_idx = i - ws.scroll_offset;
                let item_y =
                    layout.list_start_y + vis_idx as f32 * layout.list_item_height;
                if item_y + layout.list_item_height > layout.max_bottom {
                    break;
                }

                let project = &ws.recent.projects[i];

                // Initial letter inside circle
                let initial = project
                    .name
                    .chars()
                    .next()
                    .unwrap_or('?')
                    .to_uppercase()
                    .to_string();
                let initial_font = 15.0;
                let initial_cell_w = initial_font * 0.6;
                let circle_size = 32.0;
                let circle_x = layout.initial_x;
                let circle_y = item_y + (layout.list_item_height - circle_size) / 2.0;
                let init_buf = text_r.create_buffer_with_size(
                    &initial,
                    circle_size,
                    initial_font,
                    circle_size,
                );
                text_areas.push(PreparedTextArea {
                    buffer: init_buf,
                    left: circle_x + (circle_size - initial_cell_w) / 2.0,
                    top: circle_y,
                    bounds_left: circle_x,
                    bounds_top: circle_y,
                    bounds_right: circle_x + circle_size,
                    bounds_bottom: circle_y + circle_size,
                    color: to_glyphon_color(CRUST),
                });

                // Project name — larger font
                let name_text = project.name.clone();
                let name_color = if i == ws.selected {
                    to_glyphon_color(BLUE)
                } else {
                    to_glyphon_color(TEXT_COLOR)
                };
                let name_buf = text_r.create_buffer_with_size(
                    &name_text,
                    layout.content_width - layout.text_offset,
                    PROJECT_NAME_FONT_SIZE,
                    PROJECT_NAME_LINE_HEIGHT,
                );
                text_areas.push(PreparedTextArea {
                    buffer: name_buf,
                    left: layout.content_x + layout.text_offset,
                    top: item_y + 6.0,
                    bounds_left: layout.content_x,
                    bounds_top: item_y,
                    bounds_right: layout.content_x + layout.content_width,
                    bounds_bottom: item_y + layout.list_item_height,
                    color: name_color,
                });

                // Project path — smaller font
                let path_text = WelcomeScreen::project_display_path(project);
                let path_buf = text_r.create_buffer_with_size(
                    &path_text,
                    layout.content_width - layout.text_offset,
                    PROJECT_PATH_FONT_SIZE,
                    PROJECT_PATH_LINE_HEIGHT,
                );
                text_areas.push(PreparedTextArea {
                    buffer: path_buf,
                    left: layout.content_x + layout.text_offset,
                    top: item_y + 6.0 + PROJECT_NAME_LINE_HEIGHT + 2.0,
                    bounds_left: layout.content_x,
                    bounds_top: item_y,
                    bounds_right: layout.content_x + layout.content_width,
                    bounds_bottom: item_y + layout.list_item_height,
                    color: to_glyphon_color(OVERLAY0),
                });
            }
        } else {
            // No recent projects message
            let msg = "No recent projects. Press 'o' to open a project.";
            let msg_buf = text_r.create_buffer_with_size(
                msg,
                layout.content_width,
                SUBTITLE_FONT_SIZE,
                SUBTITLE_LINE_HEIGHT,
            );
            text_areas.push(PreparedTextArea {
                buffer: msg_buf,
                left: layout.content_x,
                top: layout.section_header_y,
                bounds_left: 0.0,
                bounds_top: 0.0,
                bounds_right: width_f,
                bounds_bottom: height_f,
                color: to_glyphon_color(OVERLAY0),
            });
        }

        // Help text at bottom — small, subtle
        let help = "j/k: navigate   Enter: open   o: open folder   d: remove   q: quit";
        let help_cell_w = HELP_FONT_SIZE * 0.6;
        let help_buf = text_r.create_buffer_with_size(
            help,
            width_f,
            HELP_FONT_SIZE,
            HELP_LINE_HEIGHT,
        );
        let help_w = help.len() as f32 * help_cell_w;
        text_areas.push(PreparedTextArea {
            buffer: help_buf,
            left: (width_f - help_w) / 2.0,
            top: height_f - HELP_LINE_HEIGHT - 12.0,
            bounds_left: 0.0,
            bounds_top: 0.0,
            bounds_right: width_f,
            bounds_bottom: height_f,
            color: to_glyphon_color(SURFACE1),
        });

        // ===== Draw =====
        let mut encoder =
            surface
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("welcome-encoder"),
                });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("welcome-rect-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: MANTLE[0] as f64,
                            g: MANTLE[1] as f64,
                            b: MANTLE[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let Some(rect_pl) = &self.rect_pipeline {
                rect_pl.draw(&mut pass, &surface.queue, &rects);
            }
        }

        text_r.render_text(
            &surface.device,
            &surface.queue,
            &mut encoder,
            &view,
            width,
            height,
            text_areas,
        );

        surface.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    // ─────────────────────────────────────────────
    //  Editor Screen Rendering
    // ─────────────────────────────────────────────

    fn render_editor(&mut self) {
        let surface = match self.surface.as_ref() {
            Some(s) => s,
            None => return,
        };

        let output = match surface.surface.get_current_texture() {
            Ok(tex) => tex,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                let (w, h) = surface.size;
                if let Some(s) = self.surface.as_mut() {
                    s.resize(w, h);
                }
                return;
            }
            Err(e) => {
                tracing::error!("Surface error: {}", e);
                return;
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let (width, height) = surface.size;
        let width_f = width as f32;
        let height_f = height as f32;

        if let Some(rect_pl) = &self.rect_pipeline {
            rect_pl.update_screen_size(&surface.queue, width_f, height_f);
        }

        let state = match &self.screen {
            AppScreen::Editor(s) => s,
            _ => return,
        };

        let app_view = create_app_view(state);
        let text_r = self.text_renderer.as_mut().unwrap();
        let ed_line_h = EDITOR_LINE_HEIGHT;
        let ed_cell_w = EDITOR_FONT_SIZE * 0.6; // monospace ratio
        let gutter_w = ed_cell_w * GUTTER_CHARS + GUTTER_PADDING;

        // ===== Collect rectangles =====
        let mut rects = Vec::new();

        // Tab bar background
        rects.push(RectInstance::flat([0.0, 0.0], [width_f, TAB_BAR_HEIGHT], MANTLE));

        // Build document tabs — collect all open documents with file paths
        let active_doc_id = state.active_doc_id();
        let mut doc_tabs: Vec<(usize, String)> = app_view
            .documents
            .iter()
            .map(|(&doc_id, doc)| {
                let name = doc
                    .file_path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "untitled".to_string());
                (doc_id, name)
            })
            .collect();
        doc_tabs.sort_by_key(|(id, _)| *id);

        let tab_cell_w = TAB_FONT_SIZE * 0.6;
        let tab_padding = 24.0; // padding inside each tab
        let tab_gap = 2.0;
        let mut tab_x = 8.0;
        let mut tab_positions: Vec<(usize, f32, f32, String, bool)> = Vec::new(); // (doc_id, x, w, name, is_active)

        for (doc_id, name) in &doc_tabs {
            let text_w = (name.len() as f32 + 3.0) * tab_cell_w + tab_padding;
            let is_active = active_doc_id == Some(*doc_id);
            tab_positions.push((*doc_id, tab_x, text_w, name.clone(), is_active));

            if is_active {
                // Active tab background
                rects.push(RectInstance::rounded(
                    [tab_x, 6.0],
                    [text_w, TAB_BAR_HEIGHT - 6.0],
                    BASE,
                    [6.0, 6.0, 0.0, 0.0][0], // top corners only via visual trick
                ));
                // Blue indicator bar under active tab
                rects.push(RectInstance::flat(
                    [tab_x + 6.0, TAB_BAR_HEIGHT - 2.0],
                    [text_w - 12.0, 2.0],
                    BLUE,
                ));
            } else {
                // Inactive tab — subtle background on hover
                rects.push(RectInstance::rounded(
                    [tab_x, 6.0],
                    [text_w, TAB_BAR_HEIGHT - 6.0],
                    CRUST,
                    6.0,
                ));
            }

            tab_x += text_w + tab_gap;
        }

        // Status bar background
        let status_y = height_f - STATUS_BAR_HEIGHT;
        rects.push(RectInstance::flat([0.0, status_y], [width_f, STATUS_BAR_HEIGHT], MANTLE));

        // Mode indicator — rounded pill
        let mode_name = app_view.mode.display_name();
        let mode_color = match app_view.mode {
            Mode::Normal => BLUE,
            Mode::Insert => GREEN,
            Mode::Visual | Mode::VisualLine => MAUVE,
            Mode::Command => YELLOW,
            _ => SURFACE1,
        };
        let mode_cell_w = STATUS_FONT_SIZE * 0.6;
        let mode_text_w = (mode_name.len() + 2) as f32 * mode_cell_w;
        rects.push(RectInstance::rounded(
            [6.0, status_y + 5.0],
            [mode_text_w + 8.0, STATUS_BAR_HEIGHT - 10.0],
            mode_color,
            4.0,
        ));

        // Search bar
        let search_bar_h = if self.search.active { 34.0 } else { 0.0 };
        if self.search.active {
            rects.push(RectInstance::flat(
                [0.0, TAB_BAR_HEIGHT],
                [width_f, search_bar_h],
                SURFACE0,
            ));
        }

        // Pane backgrounds and borders
        let content_top = TAB_BAR_HEIGHT + search_bar_h;
        let content_bottom = status_y;
        let content_height = content_bottom - content_top;

        for (pane_id, kode_rect) in &app_view.pane_rects {
            let px = kode_rect.x();
            let py = kode_rect.y() + content_top;
            let pw = kode_rect.width();
            let ph = kode_rect.height().min(content_height);

            // Pane background
            rects.push(RectInstance::flat([px, py], [pw, ph], BASE));

            // Pane separator (vertical)
            if px > 0.0 {
                rects.push(RectInstance::flat([px - 1.0, py], [2.0, ph], CRUST));
            }

            if let Some(pane) = app_view.panes.get(pane_id) {
                // Active pane accent bar
                if pane.focused && px > 0.0 {
                    rects.push(RectInstance::flat([px, py + 4.0], [2.0, ph - 8.0], BLUE));
                }

                match pane.content {
                    PaneContent::Editor(doc_id) => {
                        // Gutter background
                        rects.push(RectInstance::flat(
                            [px, py],
                            [gutter_w, ph],
                            MANTLE,
                        ));
                        // Gutter/code separator
                        rects.push(RectInstance::flat(
                            [px + gutter_w - 1.0, py],
                            [1.0, ph],
                            SURFACE0,
                        ));

                        if let Some(doc) = app_view.documents.get(&doc_id) {
                            let scroll = doc.scroll_offset();
                            let cursor_line = doc.cursors.primary().line();
                            // Only render cursor if visible
                            if cursor_line >= scroll {
                                let visible_line = cursor_line - scroll;
                                let cursor_y = py + 4.0 + (visible_line as f32) * ed_line_h;
                                if cursor_y + ed_line_h < py + ph {
                                    // Active line highlight — rounded
                                    rects.push(RectInstance::rounded(
                                        [px + gutter_w + 2.0, cursor_y],
                                        [pw - gutter_w - 4.0, ed_line_h],
                                        SURFACE0,
                                        3.0,
                                    ));

                                    // Cursor bar (thin I-beam style)
                                    let cursor_col = doc.cursors.primary().col();
                                    rects.push(RectInstance::rounded(
                                        [
                                            px + gutter_w + 8.0 + cursor_col as f32 * ed_cell_w,
                                            cursor_y + 1.0,
                                        ],
                                        [2.0, ed_line_h - 2.0],
                                        TEXT_COLOR,
                                        1.0,
                                    ));
                                }
                            }

                            // Search match highlights
                            if self.search.active && !self.search.matches.is_empty() {
                                let scroll = doc.scroll_offset();
                                let visible_lines = ((ph - 8.0) / ed_line_h) as usize;
                                let end_line = scroll + visible_lines;
                                // Semi-transparent yellow for matches
                                let match_color = [YELLOW[0], YELLOW[1], YELLOW[2], 0.3];
                                let current_color = [YELLOW[0], YELLOW[1], YELLOW[2], 0.6];
                                for (idx, &(line, start_col, end_col)) in self.search.matches.iter().enumerate() {
                                    if line >= scroll && line < end_line {
                                        let vis_line = line - scroll;
                                        let hy = py + 4.0 + vis_line as f32 * ed_line_h;
                                        let hx = px + gutter_w + 8.0 + start_col as f32 * ed_cell_w;
                                        let hw = (end_col - start_col) as f32 * ed_cell_w;
                                        let color = if idx == self.search.current_match { current_color } else { match_color };
                                        rects.push(RectInstance::rounded([hx, hy], [hw.max(2.0), ed_line_h], color, 2.0));
                                    }
                                }
                            }
                        }
                    }
                    PaneContent::FileExplorer(explorer_id) => {
                        // Explorer header background
                        rects.push(RectInstance::flat(
                            [px, py],
                            [pw, EXPLORER_HEADER_LINE_HEIGHT + 12.0],
                            MANTLE,
                        ));

                        if let Some(explorer) = app_view.explorers.get(&explorer_id) {
                            let header_h = EXPLORER_HEADER_LINE_HEIGHT + 12.0;
                            let visible_idx =
                                explorer.cursor.saturating_sub(explorer.scroll_offset);
                            let cursor_y =
                                py + header_h + 4.0 + visible_idx as f32 * EXPLORER_LINE_HEIGHT;
                            if cursor_y + EXPLORER_LINE_HEIGHT < py + ph {
                                // Selected item highlight — always visible
                                let sel_bg = if pane.focused { SURFACE1 } else { SURFACE0 };
                                rects.push(RectInstance::rounded(
                                    [px + 4.0, cursor_y],
                                    [pw - 8.0, EXPLORER_LINE_HEIGHT],
                                    sel_bg,
                                    4.0,
                                ));
                                // Blue accent bar (left edge)
                                rects.push(RectInstance::rounded(
                                    [px + 4.0, cursor_y + 3.0],
                                    [3.0, EXPLORER_LINE_HEIGHT - 6.0],
                                    BLUE,
                                    1.5,
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // ===== Draw rects =====
        let mut encoder =
            surface
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("kode-encoder"),
                });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("rect-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: CRUST[0] as f64,
                            g: CRUST[1] as f64,
                            b: CRUST[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let Some(rect_pl) = &self.rect_pipeline {
                rect_pl.draw(&mut pass, &surface.queue, &rects);
            }
        }

        // ===== Collect and draw text =====
        let mut text_areas: Vec<PreparedTextArea> = Vec::new();

        // Tab texts — render all document tabs
        let close_btn_w = 20.0;
        for (_doc_id, tx, tw, name, is_active) in &tab_positions {
            let tab_label = format!("  {}  ", name);
            let tab_buf = text_r.create_buffer_with_size(
                &tab_label, tw - close_btn_w, TAB_FONT_SIZE, TAB_LINE_HEIGHT,
            );
            let color = if *is_active {
                to_glyphon_color(TEXT_COLOR)
            } else {
                to_glyphon_color(OVERLAY0)
            };
            text_areas.push(PreparedTextArea {
                buffer: tab_buf,
                left: *tx + 4.0,
                top: (TAB_BAR_HEIGHT - TAB_LINE_HEIGHT) / 2.0 + 2.0,
                bounds_left: *tx,
                bounds_top: 0.0,
                bounds_right: tx + tw - close_btn_w,
                bounds_bottom: TAB_BAR_HEIGHT,
                color,
            });

            // Close button "×"
            let close_buf = text_r.create_buffer_with_size(
                "×", close_btn_w, TAB_FONT_SIZE, TAB_LINE_HEIGHT,
            );
            text_areas.push(PreparedTextArea {
                buffer: close_buf,
                left: tx + tw - close_btn_w + 2.0,
                top: (TAB_BAR_HEIGHT - TAB_LINE_HEIGHT) / 2.0 + 2.0,
                bounds_left: tx + tw - close_btn_w,
                bounds_top: 0.0,
                bounds_right: tx + tw,
                bounds_bottom: TAB_BAR_HEIGHT,
                color: to_glyphon_color(OVERLAY1),
            });
        }

        // Search bar text
        if self.search.active {
            let search_label = format!(" 🔍 {}_ ", self.search.query);
            let match_info = if self.search.matches.is_empty() {
                if self.search.query.is_empty() { String::new() } else { "No results".to_string() }
            } else {
                format!("{} of {}", self.search.current_match + 1, self.search.matches.len())
            };
            let search_text = format!("{}  {}", search_label, match_info);
            let search_buf = text_r.create_buffer_with_size(
                &search_text, width_f, 13.0, 18.0,
            );
            text_areas.push(PreparedTextArea {
                buffer: search_buf,
                left: 12.0,
                top: TAB_BAR_HEIGHT + 8.0,
                bounds_left: 0.0,
                bounds_top: TAB_BAR_HEIGHT,
                bounds_right: width_f,
                bounds_bottom: TAB_BAR_HEIGHT + search_bar_h,
                color: to_glyphon_color(TEXT_COLOR),
            });
        }

        // Status bar: mode text
        let status_mode = format!(" {} ", mode_name.to_uppercase());
        let status_mode_buf = text_r.create_buffer_with_size(
            &status_mode, width_f, STATUS_FONT_SIZE, STATUS_LINE_HEIGHT,
        );
        text_areas.push(PreparedTextArea {
            buffer: status_mode_buf,
            left: 10.0,
            top: status_y + (STATUS_BAR_HEIGHT - STATUS_LINE_HEIGHT) / 2.0,
            bounds_left: 0.0,
            bounds_top: status_y,
            bounds_right: width_f,
            bounds_bottom: height_f,
            color: to_glyphon_color(CRUST),
        });

        // Status bar: file info (right-aligned)
        // Get info from focused pane
        let mut status_right = String::new();
        if let Some(pane) = app_view.panes.get(&state.focused_pane) {
            if let PaneContent::Editor(doc_id) = pane.content {
                if let Some(doc) = app_view.documents.get(&doc_id) {
                    let ln = doc.cursors.primary().line() + 1;
                    let col = doc.cursors.primary().col() + 1;
                    let title = doc.title();
                    status_right = format!("{}  Ln {}, Col {}  ", title, ln, col);
                }
            }
        }
        if !status_right.is_empty() {
            let sr_cell_w = STATUS_FONT_SIZE * 0.6;
            let sr_w = status_right.len() as f32 * sr_cell_w;
            let sr_buf = text_r.create_buffer_with_size(
                &status_right, width_f, STATUS_FONT_SIZE, STATUS_LINE_HEIGHT,
            );
            text_areas.push(PreparedTextArea {
                buffer: sr_buf,
                left: width_f - sr_w - 8.0,
                top: status_y + (STATUS_BAR_HEIGHT - STATUS_LINE_HEIGHT) / 2.0,
                bounds_left: 0.0,
                bounds_top: status_y,
                bounds_right: width_f,
                bounds_bottom: height_f,
                color: to_glyphon_color(OVERLAY0),
            });
        }

        // Pane content text
        for (pane_id, kode_rect) in &app_view.pane_rects {
            let px = kode_rect.x();
            let py = kode_rect.y() + content_top;
            let pw = kode_rect.width();
            let ph = kode_rect.height().min(content_height);

            if let Some(pane) = app_view.panes.get(pane_id) {
                match pane.content {
                    PaneContent::Editor(doc_id) => {
                        if let Some(doc) = app_view.documents.get(&doc_id) {
                            let visible_lines = ((ph - 8.0) / ed_line_h) as usize;
                            let scroll = doc.scroll_offset();
                            let line_count = doc.buffer.line_count();
                            let end = (scroll + visible_lines).min(line_count);

                            for i in scroll..end {
                                let visual_idx = i - scroll;
                                let line_y = py + 4.0 + visual_idx as f32 * ed_line_h;
                                if line_y + ed_line_h > py + ph {
                                    break;
                                }

                                // Line number — smaller, right-aligned
                                let line_num = format!("{:>4} ", i + 1);
                                let gutter_buf = text_r.create_buffer_with_size(
                                    &line_num, gutter_w, LINE_NUM_FONT_SIZE, LINE_NUM_LINE_HEIGHT,
                                );
                                text_areas.push(PreparedTextArea {
                                    buffer: gutter_buf,
                                    left: px + 4.0,
                                    top: line_y,
                                    bounds_left: px,
                                    bounds_top: py,
                                    bounds_right: px + gutter_w,
                                    bounds_bottom: py + ph,
                                    color: to_glyphon_color(OVERLAY0),
                                });

                                // Code line (no word wrap, with basic syntax coloring)
                                let line_text =
                                    doc.buffer.line_to_string(i).unwrap_or_default();
                                if !line_text.is_empty() {
                                    let trimmed = line_text.trim_end_matches('\n');
                                    let line_color = syntax_line_color(trimmed);
                                    let code_buf = text_r.create_buffer_no_wrap(
                                        trimmed,
                                        EDITOR_FONT_SIZE,
                                        EDITOR_LINE_HEIGHT,
                                    );
                                    text_areas.push(PreparedTextArea {
                                        buffer: code_buf,
                                        left: px + gutter_w + 8.0,
                                        top: line_y,
                                        bounds_left: px + gutter_w,
                                        bounds_top: py,
                                        bounds_right: px + pw,
                                        bounds_bottom: py + ph,
                                        color: to_glyphon_color(line_color),
                                    });
                                }
                            }

                            // IME preedit text overlay at cursor position
                            if !self.ime_preedit.is_empty() {
                                let cursor_line = doc.cursors.primary().line();
                                let cursor_col = doc.cursors.primary().col();
                                let scroll = doc.scroll_offset();
                                if cursor_line >= scroll {
                                    let visual_line = cursor_line - scroll;
                                    let cursor_y = py + 4.0 + visual_line as f32 * ed_line_h;
                                    let cursor_px = cursor_col as f32 * ed_cell_w;
                                    let preedit_buf = text_r.create_buffer_no_wrap(
                                        &self.ime_preedit, EDITOR_FONT_SIZE, EDITOR_LINE_HEIGHT,
                                    );
                                    text_areas.push(PreparedTextArea {
                                        buffer: preedit_buf,
                                        left: px + gutter_w + 8.0 + cursor_px,
                                        top: cursor_y,
                                        bounds_left: px + gutter_w,
                                        bounds_top: py,
                                        bounds_right: px + pw,
                                        bounds_bottom: py + ph,
                                        color: to_glyphon_color(YELLOW),
                                    });
                                }
                            }
                        }
                    }
                    PaneContent::FileExplorer(explorer_id) => {
                        if let Some(explorer) = app_view.explorers.get(&explorer_id) {
                            let header_h = EXPLORER_HEADER_LINE_HEIGHT + 12.0;

                            // Header: "EXPLORER"
                            let header_buf = text_r.create_buffer_with_size(
                                "EXPLORER",
                                pw,
                                EXPLORER_HEADER_FONT_SIZE,
                                EXPLORER_HEADER_LINE_HEIGHT,
                            );
                            text_areas.push(PreparedTextArea {
                                buffer: header_buf,
                                left: px + 12.0,
                                top: py + 6.0,
                                bounds_left: px,
                                bounds_top: py,
                                bounds_right: px + pw,
                                bounds_bottom: py + header_h,
                                color: to_glyphon_color(OVERLAY0),
                            });

                            let list_top = py + header_h + 4.0;
                            let available_h = ph - header_h - 4.0;
                            let visible_lines =
                                (available_h / EXPLORER_LINE_HEIGHT) as usize;

                            for i in 0..visible_lines {
                                let entry_idx = explorer.scroll_offset + i;
                                if entry_idx >= explorer.entries.len() {
                                    break;
                                }
                                let entry = &explorer.entries[entry_idx];
                                let entry_y = list_top + i as f32 * EXPLORER_LINE_HEIGHT;
                                if entry_y + EXPLORER_LINE_HEIGHT > py + ph {
                                    break;
                                }

                                let indent = "  ".repeat(entry.depth);
                                let icon = file_icon(&entry.name, entry.is_dir, entry.expanded);
                                let display =
                                    format!("{}{} {}", indent, icon, entry.name);
                                let color = if entry.is_dir {
                                    to_glyphon_color(BLUE)
                                } else {
                                    to_glyphon_color(TEXT_COLOR)
                                };

                                let entry_buf = text_r.create_buffer_with_size(
                                    &display,
                                    pw - 20.0,
                                    EXPLORER_FONT_SIZE,
                                    EXPLORER_LINE_HEIGHT,
                                );
                                text_areas.push(PreparedTextArea {
                                    buffer: entry_buf,
                                    left: px + 12.0,
                                    top: entry_y + 2.0,
                                    bounds_left: px,
                                    bounds_top: list_top,
                                    bounds_right: px + pw,
                                    bounds_bottom: py + ph,
                                    color,
                                });
                            }

                            // Input mode UI (create/rename)
                            if let Some(input_mode) = &explorer.input_mode {
                                let label = match input_mode {
                                    InputMode::Create { is_dir: true, .. } => "New dir: ",
                                    InputMode::Create { is_dir: false, .. } => "New file: ",
                                    InputMode::Rename { .. } => "Rename: ",
                                };
                                let input_y = py + ph - EXPLORER_LINE_HEIGHT - 8.0;
                                // Background
                                rects.push(RectInstance::flat(
                                    [px, input_y],
                                    [pw, EXPLORER_LINE_HEIGHT + 8.0],
                                    SURFACE0,
                                ));
                                let input_text = format!("{}{}_", label, explorer.input_buffer);
                                let input_buf = text_r.create_buffer_with_size(
                                    &input_text, pw - 16.0, EXPLORER_FONT_SIZE, EXPLORER_LINE_HEIGHT,
                                );
                                text_areas.push(PreparedTextArea {
                                    buffer: input_buf,
                                    left: px + 8.0,
                                    top: input_y + 4.0,
                                    bounds_left: px,
                                    bounds_top: input_y,
                                    bounds_right: px + pw,
                                    bounds_bottom: py + ph,
                                    color: to_glyphon_color(TEXT_COLOR),
                                });
                            }

                            // Confirm delete dialog
                            if let Some(idx) = explorer.confirm_delete {
                                let name = explorer.entries.get(idx)
                                    .map(|e| e.name.as_str())
                                    .unwrap_or("?");
                                let confirm_y = py + ph - EXPLORER_LINE_HEIGHT * 2.0 - 8.0;
                                // Red background
                                rects.push(RectInstance::rounded(
                                    [px + 4.0, confirm_y],
                                    [pw - 8.0, EXPLORER_LINE_HEIGHT * 2.0 + 4.0],
                                    RED,
                                    4.0,
                                ));
                                let confirm_text = format!("Delete {}? (y/n)", name);
                                let confirm_buf = text_r.create_buffer_with_size(
                                    &confirm_text, pw - 24.0, EXPLORER_FONT_SIZE, EXPLORER_LINE_HEIGHT,
                                );
                                text_areas.push(PreparedTextArea {
                                    buffer: confirm_buf,
                                    left: px + 12.0,
                                    top: confirm_y + 4.0,
                                    bounds_left: px,
                                    bounds_top: confirm_y,
                                    bounds_right: px + pw,
                                    bounds_bottom: py + ph,
                                    color: to_glyphon_color(CRUST),
                                });
                            }

                            // Keyboard shortcut hints at bottom
                            if explorer.input_mode.is_none() && explorer.confirm_delete.is_none() {
                                let hint = "a:new  A:dir  r:rename  d:delete";
                                let hint_y = py + ph - 20.0;
                                let hint_buf = text_r.create_buffer_with_size(
                                    hint, pw - 16.0, 10.0, 14.0,
                                );
                                text_areas.push(PreparedTextArea {
                                    buffer: hint_buf,
                                    left: px + 8.0,
                                    top: hint_y,
                                    bounds_left: px,
                                    bounds_top: hint_y,
                                    bounds_right: px + pw,
                                    bounds_bottom: py + ph,
                                    color: to_glyphon_color(OVERLAY0),
                                });
                            }
                        }
                    }
                    PaneContent::Terminal(_) => {
                        let term_buf =
                            text_r.create_buffer("Terminal (GPU pending)", pw);
                        text_areas.push(PreparedTextArea {
                            buffer: term_buf,
                            left: px + 12.0,
                            top: py + 12.0,
                            bounds_left: px,
                            bounds_top: py,
                            bounds_right: px + pw,
                            bounds_bottom: py + ph,
                            color: to_glyphon_color(OVERLAY0),
                        });
                    }
                    _ => {}
                }
            }
        }

        text_r.render_text(
            &surface.device,
            &surface.queue,
            &mut encoder,
            &view,
            width,
            height,
            text_areas,
        );

        surface.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn update_ime_cursor_area(&self) {
        if let (Some(window), AppScreen::Editor(state)) = (&self.window, &self.screen) {
            if let Some(pane) = state.panes.get(&state.focused_pane) {
                if let PaneContent::Editor(doc_id) = pane.content {
                    if let Some(doc) = state.documents.get(&doc_id) {
                        let ed_cell_w = EDITOR_FONT_SIZE * 0.6;
                        let gutter_w = ed_cell_w * GUTTER_CHARS + GUTTER_PADDING;
                        let pane_rects = state.pane_rects();
                        let content_top = TAB_BAR_HEIGHT
                            + if self.search.active { 34.0 } else { 0.0 };

                        if let Some((_, kode_rect)) = pane_rects.iter().find(|(id, _)| *id == state.focused_pane) {
                            let px = kode_rect.x();
                            let py = kode_rect.y() + content_top;
                            let cursor_line = doc.cursors.primary().line();
                            let cursor_col = doc.cursors.primary().col();
                            let scroll = doc.scroll_offset();
                            let visual_line = cursor_line.saturating_sub(scroll);

                            let cursor_x = px + gutter_w + 8.0 + cursor_col as f32 * ed_cell_w;
                            let cursor_y = py + 4.0 + visual_line as f32 * EDITOR_LINE_HEIGHT;

                            window.set_ime_cursor_area(
                                winit::dpi::LogicalPosition::new(cursor_x as f64, cursor_y as f64),
                                winit::dpi::LogicalSize::new(ed_cell_w as f64, EDITOR_LINE_HEIGHT as f64),
                            );
                        }
                    }
                }
            }
        }
    }

    // ─────────────────────────────────────────────
    //  Key Handling
    // ─────────────────────────────────────────────

    fn handle_key(&mut self, key: kode_core::event::KeyEvent) {
        // Cmd+F → toggle search
        if key.code == KeyCode::Char('f') && key.modifiers.contains(Modifiers::SUPER) {
            if let AppScreen::Editor(_) = &self.screen {
                self.search.active = !self.search.active;
                if !self.search.active {
                    self.search.query.clear();
                    self.search.matches.clear();
                }
                return;
            }
        }

        // Handle search input when active
        if self.search.active {
            if let AppScreen::Editor(state) = &mut self.screen {
                match key.code {
                    KeyCode::Escape => {
                        self.search.active = false;
                        self.search.query.clear();
                        self.search.matches.clear();
                    }
                    KeyCode::Enter => {
                        // Go to next match
                        if !self.search.matches.is_empty() {
                            self.search.current_match =
                                (self.search.current_match + 1) % self.search.matches.len();
                            let (line, col, _) = self.search.matches[self.search.current_match];
                            if let Some(doc) = state.focused_editor_doc_mut() {
                                doc.cursors.primary_mut().move_to(line, col);
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        self.search.query.pop();
                        Self::update_search_matches(&self.search.query, state, &mut self.search.matches, &mut self.search.current_match);
                    }
                    KeyCode::Char(c) if key.modifiers == Modifiers::NONE || key.modifiers == Modifiers::SHIFT => {
                        self.search.query.push(c);
                        Self::update_search_matches(&self.search.query, state, &mut self.search.matches, &mut self.search.current_match);
                        // Jump to first match
                        if !self.search.matches.is_empty() {
                            let (line, col, _) = self.search.matches[self.search.current_match];
                            if let Some(doc) = state.focused_editor_doc_mut() {
                                doc.cursors.primary_mut().move_to(line, col);
                            }
                        }
                    }
                    _ => {}
                }
                return;
            }
        }

        match &mut self.screen {
            AppScreen::Welcome(ws) => {
                let action = ws.handle_key(key);
                self.process_welcome_action(action);
            }
            AppScreen::Editor(state) => {
                Self::handle_editor_key(state, key);
            }
        }
    }

    fn update_search_matches(
        query: &str,
        state: &AppState,
        matches: &mut Vec<(usize, usize, usize)>,
        current_match: &mut usize,
    ) {
        matches.clear();
        *current_match = 0;
        if query.is_empty() {
            return;
        }
        let query_lower = query.to_lowercase();
        if let Some(doc) = state.panes.get(&state.focused_pane).and_then(|p| {
            match p.content {
                PaneContent::Editor(doc_id) => state.documents.get(&doc_id),
                _ => None,
            }
        }) {
            for line_idx in 0..doc.buffer.line_count() {
                if let Some(line_text) = doc.buffer.line_to_string(line_idx) {
                    let line_lower = line_text.to_lowercase();
                    let mut start = 0;
                    while let Some(pos) = line_lower[start..].find(&query_lower) {
                        let col = start + pos;
                        matches.push((line_idx, col, col + query.len()));
                        start = col + 1;
                    }
                }
            }
        }
    }

    fn process_welcome_action(&mut self, action: WelcomeAction) {
        match action {
            WelcomeAction::None => {}
            WelcomeAction::OpenProject(path) => {
                self.open_project(path);
            }
            WelcomeAction::OpenFolderPicker => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Open Project")
                    .pick_folder()
                {
                    self.open_project(path);
                }
            }
            WelcomeAction::Quit => {
                self.should_quit = true;
            }
        }
    }

    fn open_project(&mut self, path: std::path::PathBuf) {
        // Record in recent projects
        if let AppScreen::Welcome(ws) = &mut self.screen {
            ws.record_open(&path);
        }

        let mut state = build_editor_state(path.clone());

        // Apply current viewport size
        if let Some(surface) = &self.surface {
            let (w, h) = surface.size;
            state.set_viewport_pixels(w as f32, h as f32);
        }

        // Update window title
        if let Some(window) = &self.window {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            window.set_title(&format!("kode - {}", name));
        }

        self.screen = AppScreen::Editor(state);
    }

    fn handle_editor_key(state: &mut AppState, key: kode_core::event::KeyEvent) {
        // Cmd+S → save focused document
        if key.code == KeyCode::Char('s') && key.modifiers.contains(Modifiers::SUPER) {
            state.save_focused();
            return;
        }

        if let Some(pane) = state.panes.get(&state.focused_pane) {
            match pane.content {
                PaneContent::Terminal(term_id) => {
                    if key.code == KeyCode::Char('a')
                        && key.modifiers.contains(Modifiers::CTRL)
                    {
                        state.handle_key_event(key);
                        return;
                    }
                    if let Some(escape_bytes) = key_to_escape(&key) {
                        if let Some(terminal) = state.terminals.get_mut(&term_id) {
                            let _ = terminal.write_input(&escape_bytes);
                        }
                    }
                    return;
                }
                PaneContent::FileExplorer(explorer_id) => {
                    if key.code == KeyCode::Char('a')
                        && key.modifiers.contains(Modifiers::CTRL)
                    {
                        state.handle_key_event(key);
                        return;
                    }
                    handle_explorer_key(state, explorer_id, key);
                    return;
                }
                _ => {}
            }
        }
        state.handle_key_event(key);
        tracing::debug!("After handle_key_event, mode: {:?}", state.mode());
    }
}

impl ApplicationHandler for GpuApp {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {}

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let title = match &self.screen {
            AppScreen::Welcome(_) => "kode",
            AppScreen::Editor(_) => "kode",
        };

        let attrs = WindowAttributes::default()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(1200.0, 800.0));

        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        window.set_ime_allowed(true);
        self.scale_factor = window.scale_factor();

        let surface = pollster::block_on(GpuSurface::new(window.clone()));
        let rect_pipeline = RectPipeline::new(&surface.device, surface.format);
        let text_renderer =
            KodeTextRenderer::new(&surface.device, &surface.queue, surface.format);

        let size = window.inner_size();
        if let AppScreen::Editor(state) = &mut self.screen {
            state.set_viewport_pixels(size.width as f32, size.height as f32);
        }

        self.window = Some(window);
        self.rect_pipeline = Some(rect_pipeline);
        self.text_renderer = Some(text_renderer);
        self.surface = Some(surface);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(surface) = self.surface.as_mut() {
                    surface.resize(size.width, size.height);
                }
                if let AppScreen::Editor(state) = &mut self.screen {
                    state.set_viewport_pixels(size.width as f32, size.height as f32);
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
            }
            WindowEvent::ModifiersChanged(new_mods) => {
                self.modifiers = new_mods;
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.x, position.y);
                if let AppScreen::Welcome(ws) = &mut self.screen {
                    let surface = self.surface.as_ref();
                    if let (Some(surface), Some(text_r)) =
                        (surface, self.text_renderer.as_ref())
                    {
                        let (w, h) = surface.size;
                        let layout = WelcomeLayout::compute(
                            w as f32,
                            h as f32,
                            text_r.line_height,
                        );
                        ws.update_hover(position.x as f32, position.y as f32, &layout);
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                let (cx, cy) = self.cursor_pos;
                match &mut self.screen {
                    AppScreen::Welcome(ws) => {
                        let surface = self.surface.as_ref();
                        if let (Some(surface), Some(text_r)) =
                            (surface, self.text_renderer.as_ref())
                        {
                            let (w, h) = surface.size;
                            let layout = WelcomeLayout::compute(
                                w as f32,
                                h as f32,
                                text_r.line_height,
                            );
                            let action = ws.handle_click(cx as f32, cy as f32, &layout);
                            self.process_welcome_action(action);
                        }
                    }
                    AppScreen::Editor(state) => {
                        let surface = self.surface.as_ref();
                        if let Some(surface) = surface {
                            let (w, _h) = surface.size;
                            let click_x = cx as f32;
                            let click_y = cy as f32;
                            let search_h = if self.search.active { 34.0 } else { 0.0 };
                            let content_top = TAB_BAR_HEIGHT + search_h;
                            let status_y = _h as f32 - STATUS_BAR_HEIGHT;

                            // Tab bar clicks — switch document
                            if click_y < content_top {
                                let tab_cell_w = TAB_FONT_SIZE * 0.6;
                                let tab_padding = 24.0;
                                let tab_gap = 2.0;
                                let mut tx = 8.0f32;
                                let mut doc_tabs: Vec<(usize, String)> = state
                                    .documents
                                    .iter()
                                    .map(|(&doc_id, doc)| {
                                        let name = doc
                                            .file_path
                                            .as_ref()
                                            .and_then(|p| p.file_name())
                                            .map(|n| n.to_string_lossy().to_string())
                                            .unwrap_or_else(|| "untitled".to_string());
                                        (doc_id, name)
                                    })
                                    .collect();
                                doc_tabs.sort_by_key(|(id, _)| *id);

                                for (doc_id, name) in &doc_tabs {
                                    let tw =
                                        (name.len() as f32 + 3.0) * tab_cell_w + tab_padding;
                                    if click_x >= tx && click_x < tx + tw {
                                        let close_btn_w = 20.0;
                                        if click_x > tx + tw - close_btn_w {
                                            // Close this tab
                                            state.close_document(*doc_id);
                                        } else {
                                            // Switch to this document
                                            if let Some((&pane_id, _)) =
                                                state.panes.iter().find(|(_, p)| {
                                                    matches!(p.content, PaneContent::Editor(_))
                                                })
                                            {
                                                if let Some(pane) = state.panes.get_mut(&pane_id) {
                                                    pane.content = PaneContent::Editor(*doc_id);
                                                }
                                                state.set_focus(pane_id);
                                            }
                                        }
                                        break;
                                    }
                                    tx += tw + tab_gap;
                                }
                            }

                            // Content area clicks
                            if click_y > content_top && click_y < status_y {
                                let pane_rects = state.pane_rects();
                                for (pane_id, kode_rect) in &pane_rects {
                                    let px = kode_rect.x();
                                    let py = kode_rect.y() + content_top;
                                    let pw = kode_rect.width();
                                    let ph = kode_rect.height().min(status_y - content_top);

                                    if click_x >= px
                                        && click_x < px + pw
                                        && click_y >= py
                                        && click_y < py + ph
                                    {
                                        // Focus this pane
                                        state.set_focus(*pane_id);

                                        if let Some(pane) =
                                            state.panes.get(pane_id).cloned()
                                        {
                                            match pane.content {
                                                PaneContent::FileExplorer(explorer_id) => {
                                                    let header_h =
                                                        EXPLORER_HEADER_LINE_HEIGHT + 12.0;
                                                    let rel_y = click_y - py - header_h - 4.0;
                                                    if rel_y >= 0.0 {
                                                        let clicked_idx = (rel_y
                                                            / EXPLORER_LINE_HEIGHT)
                                                            as usize;
                                                        if let Some(explorer) =
                                                            state.explorers.get_mut(&explorer_id)
                                                        {
                                                            let abs_idx = clicked_idx
                                                                + explorer.scroll_offset;
                                                            if abs_idx < explorer.entries.len() {
                                                                explorer.cursor = abs_idx;
                                                                let entry = explorer.entries
                                                                    [abs_idx]
                                                                    .clone();
                                                                if entry.is_dir {
                                                                    explorer.toggle_expand();
                                                                } else {
                                                                    let path = entry.path.clone();
                                                                    state
                                                                        .open_file_from_explorer(
                                                                            path,
                                                                        );
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                PaneContent::Editor(doc_id) => {
                                                    let ed_cell_w = EDITOR_FONT_SIZE * 0.6;
                                                    let gutter_w = ed_cell_w * GUTTER_CHARS
                                                        + GUTTER_PADDING;
                                                    let rel_x = click_x - px - gutter_w - 8.0;
                                                    let rel_y = click_y - py - 4.0;
                                                    if rel_x >= 0.0 && rel_y >= 0.0 {
                                                        let scroll_off = state.documents.get(&doc_id)
                                                            .map(|d| d.scroll_offset()).unwrap_or(0);
                                                        let line =
                                                            (rel_y / EDITOR_LINE_HEIGHT) as usize + scroll_off;
                                                        let col =
                                                            (rel_x / ed_cell_w) as usize;
                                                        if let Some(doc) =
                                                            state.documents.get_mut(&doc_id)
                                                        {
                                                            let max_line =
                                                                doc.buffer.line_count()
                                                                    .saturating_sub(1);
                                                            let target_line = line.min(max_line);
                                                            let line_len = doc
                                                                .buffer
                                                                .line_len(target_line);
                                                            let target_col = col.min(line_len);
                                                            doc.cursors.primary_mut().move_to(
                                                                target_line,
                                                                target_col,
                                                            );
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let lines = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        // Typically ±1.0 per notch, clamp to ±3
                        (-y as i32).clamp(-3, 3)
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        // macOS trackpad: accumulate pixels, 1 line per ~40px
                        let raw = -(pos.y / 40.0);
                        if raw.abs() < 0.5 { 0 } else { raw as i32 }
                    }
                };
                if lines != 0 {
                    if let AppScreen::Editor(state) = &mut self.screen {
                        let (cx, cy) = self.cursor_pos;
                        let click_x = cx as f32;
                        let click_y = cy as f32;
                        let search_h = if self.search.active { 34.0 } else { 0.0 };
                        let content_top = TAB_BAR_HEIGHT + search_h;
                        let surface = self.surface.as_ref();
                        if let Some(surface) = surface {
                            let status_y = surface.size.1 as f32 - STATUS_BAR_HEIGHT;
                            if click_y > content_top && click_y < status_y {
                                let pane_rects = state.pane_rects();
                                for (pane_id, kode_rect) in &pane_rects {
                                    let px = kode_rect.x();
                                    let py = kode_rect.y() + content_top;
                                    let pw = kode_rect.width();
                                    let ph = kode_rect.height().min(status_y - content_top);
                                    if click_x >= px && click_x < px + pw
                                        && click_y >= py && click_y < py + ph
                                    {
                                        if let Some(pane) = state.panes.get(pane_id) {
                                            match pane.content {
                                                PaneContent::Editor(doc_id) => {
                                                    if let Some(doc) = state.documents.get_mut(&doc_id) {
                                                        let max = doc.buffer.line_count().saturating_sub(1);
                                                        let current = doc.scroll_offset() as i32;
                                                        let new_offset = (current + lines).clamp(0, max as i32) as usize;
                                                        doc.set_scroll_offset(new_offset);
                                                    }
                                                }
                                                PaneContent::FileExplorer(explorer_id) => {
                                                    if let Some(explorer) = state.explorers.get_mut(&explorer_id) {
                                                        let max = explorer.entries.len().saturating_sub(1);
                                                        let current = explorer.scroll_offset as i32;
                                                        let new_offset = (current + lines).clamp(0, max as i32) as usize;
                                                        explorer.scroll_offset = new_offset;
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Skip character input events during IME composing to avoid duplicates
                if self.ime_composing && event.state == winit::event::ElementState::Pressed {
                    if let winit::keyboard::Key::Character(_) = &event.logical_key {
                        // IME will handle this via Ime::Commit
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                        return;
                    }
                }
                if let Some(key) = translate_winit_key(&event, &self.modifiers) {
                    self.handle_key(key);

                    if self.should_quit {
                        event_loop.exit();
                        return;
                    }

                    // Auto-scroll to keep cursor visible after key input
                    if let AppScreen::Editor(state) = &mut self.screen {
                        if let Some(surface) = self.surface.as_ref() {
                            let status_y = surface.size.1 as f32 - STATUS_BAR_HEIGHT;
                            let content_h = status_y - TAB_BAR_HEIGHT;
                            let visible_lines = (content_h / EDITOR_LINE_HEIGHT) as usize;
                            if let Some(doc) = state.focused_editor_doc_mut() {
                                doc.ensure_cursor_visible(visible_lines);
                            }
                        }
                    }

                    if let AppScreen::Editor(state) = &self.screen {
                        if !state.is_running() {
                            event_loop.exit();
                            return;
                        }
                    }

                    self.update_ime_cursor_area();
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::Ime(ime_event) => {
                match ime_event {
                    winit::event::Ime::Enabled => {
                        self.ime_composing = false;
                    }
                    winit::event::Ime::Disabled => {
                        self.ime_composing = false;
                    }
                    winit::event::Ime::Commit(text) => {
                        self.ime_composing = false;
                        self.ime_preedit.clear();
                        // IME committed text (e.g., composed Korean/Japanese/Chinese characters)
                        if let AppScreen::Editor(state) = &mut self.screen {
                            if self.search.active {
                                self.search.query.push_str(&text);
                                Self::update_search_matches(&self.search.query, state, &mut self.search.matches, &mut self.search.current_match);
                                if !self.search.matches.is_empty() {
                                    let (line, col, _) = self.search.matches[self.search.current_match];
                                    if let Some(doc) = state.focused_editor_doc_mut() {
                                        doc.cursors.primary_mut().move_to(line, col);
                                    }
                                }
                            } else if state.mode() == Mode::Insert {
                                if let Some(doc) = state.focused_editor_doc_mut() {
                                    for ch in text.chars() {
                                        doc.insert_char(ch);
                                    }
                                }
                            }

                            // Handle explorer input mode
                            if let Some(pane) = state.panes.get(&state.focused_pane) {
                                if let PaneContent::FileExplorer(explorer_id) = pane.content {
                                    if let Some(explorer) = state.explorers.get_mut(&explorer_id) {
                                        if explorer.input_mode.is_some() {
                                            explorer.input_buffer.push_str(&text);
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                    winit::event::Ime::Preedit(text, _cursor) => {
                        self.ime_composing = !text.is_empty();
                        self.ime_preedit = text;
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let AppScreen::Editor(state) = &mut self.screen {
                    for terminal in state.terminals.values_mut() {
                        let _ = terminal.process_output();
                    }
                }
                self.render();
            }
            _ => {}
        }
    }
}

/// Handle explorer key events.
fn handle_explorer_key(
    state: &mut AppState,
    explorer_id: usize,
    key: kode_core::event::KeyEvent,
) {
    let in_input = state
        .explorers
        .get(&explorer_id)
        .map(|e| e.input_mode.is_some())
        .unwrap_or(false);
    let in_confirm = state
        .explorers
        .get(&explorer_id)
        .map(|e| e.confirm_delete.is_some())
        .unwrap_or(false);

    if in_confirm {
        match key.code {
            KeyCode::Char('y') => {
                if let Some(e) = state.explorers.get_mut(&explorer_id) {
                    let _ = e.confirm_delete_yes();
                }
            }
            _ => {
                if let Some(e) = state.explorers.get_mut(&explorer_id) {
                    e.cancel_input();
                }
            }
        }
        return;
    }

    if in_input {
        match key.code {
            KeyCode::Escape => {
                if let Some(e) = state.explorers.get_mut(&explorer_id) {
                    e.cancel_input();
                }
            }
            KeyCode::Enter => {
                let result = state
                    .explorers
                    .get_mut(&explorer_id)
                    .and_then(|e| e.confirm_input().ok())
                    .flatten();
                if let Some(path) = result {
                    state.open_file_from_explorer(path);
                }
            }
            KeyCode::Backspace => {
                if let Some(e) = state.explorers.get_mut(&explorer_id) {
                    e.input_buffer.pop();
                }
            }
            KeyCode::Char(c) => {
                if let Some(e) = state.explorers.get_mut(&explorer_id) {
                    e.input_buffer.push(c);
                }
            }
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if let Some(e) = state.explorers.get_mut(&explorer_id) {
                e.move_cursor_down();
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if let Some(e) = state.explorers.get_mut(&explorer_id) {
                e.move_cursor_up();
            }
        }
        KeyCode::Char('l') | KeyCode::Enter => {
            let action = state.explorers.get(&explorer_id).and_then(|e| {
                e.selected_entry().map(|entry| {
                    if entry.is_dir {
                        None
                    } else {
                        Some(entry.path.clone())
                    }
                })
            });
            match action {
                Some(None) => {
                    if let Some(e) = state.explorers.get_mut(&explorer_id) {
                        e.toggle_expand();
                    }
                }
                Some(Some(path)) => {
                    state.open_file_from_explorer(path);
                }
                None => {}
            }
        }
        KeyCode::Char('h') | KeyCode::Backspace => {
            if let Some(e) = state.explorers.get_mut(&explorer_id) {
                e.collapse_current();
            }
        }
        KeyCode::Char('a') => {
            if let Some(e) = state.explorers.get_mut(&explorer_id) {
                e.start_create(false);
            }
        }
        KeyCode::Char('A') => {
            if let Some(e) = state.explorers.get_mut(&explorer_id) {
                e.start_create(true);
            }
        }
        KeyCode::Char('d') => {
            if let Some(e) = state.explorers.get_mut(&explorer_id) {
                e.request_delete();
            }
        }
        KeyCode::Char('r') => {
            if let Some(e) = state.explorers.get_mut(&explorer_id) {
                e.start_rename();
            }
        }
        KeyCode::Char('q') | KeyCode::Escape => {
            state.toggle_explorer();
        }
        _ => {}
    }
}
