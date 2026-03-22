use std::sync::Arc;

use glyphon::Color;
use kode_core::event::{KeyCode, Modifiers};
use kode_keymap::mode::Mode;
use kode_state::{AppState, create_app_view};
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
use crate::welcome_screen::{WelcomeAction, WelcomeLayout, WelcomeScreen};

// Catppuccin Mocha colors
const BASE: [f32; 4] = [0.118, 0.118, 0.180, 1.0]; // #1e1e2e
const MANTLE: [f32; 4] = [0.110, 0.110, 0.165, 1.0]; // #181825
const CRUST: [f32; 4] = [0.067, 0.067, 0.125, 1.0]; // #11111b
const SURFACE0: [f32; 4] = [0.192, 0.196, 0.267, 1.0]; // #313244
const SURFACE1: [f32; 4] = [0.271, 0.278, 0.353, 1.0]; // #45475a
const TEXT_COLOR: [f32; 4] = [0.804, 0.839, 0.957, 1.0]; // #cdd6f4
const BLUE: [f32; 4] = [0.537, 0.706, 0.980, 1.0]; // #89b4fa
const GREEN: [f32; 4] = [0.651, 0.890, 0.631, 1.0]; // #a6e3a1
const YELLOW: [f32; 4] = [0.976, 0.886, 0.686, 1.0]; // #f9e2af
const MAUVE: [f32; 4] = [0.796, 0.651, 0.969, 1.0]; // #cba6f7
const OVERLAY0: [f32; 4] = [0.424, 0.439, 0.549, 1.0]; // #6c7086

fn to_glyphon_color(c: [f32; 4]) -> Color {
    Color::rgba(
        (c[0] * 255.0) as u8,
        (c[1] * 255.0) as u8,
        (c[2] * 255.0) as u8,
        (c[3] * 255.0) as u8,
    )
}

/// The two screens of the application.
pub enum AppScreen {
    Welcome(WelcomeScreen),
    Editor(AppState),
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

        // "Open Project" button background
        rects.push(RectInstance {
            pos: [layout.button_x, layout.button_y],
            size: [layout.button_w, layout.button_h],
            color: SURFACE1,
        });

        // Divider line
        rects.push(RectInstance {
            pos: [layout.content_x, layout.divider_y],
            size: [layout.content_width, 1.0],
            color: SURFACE0,
        });

        // Selected project highlight
        let project_count = ws.recent.projects.len();
        if project_count > 0 && ws.selected < project_count {
            let vis_idx = ws.selected.saturating_sub(ws.scroll_offset);
            let sel_y = layout.list_start_y + vis_idx as f32 * layout.list_item_height;
            if sel_y + layout.list_item_height <= layout.max_bottom {
                rects.push(RectInstance {
                    pos: [layout.content_x - 8.0, sel_y],
                    size: [layout.content_width + 16.0, layout.list_item_height],
                    color: SURFACE0,
                });
            }
        }

        // Hover highlight (if different from selection)
        if let Some(hover_idx) = ws.hover_index {
            if hover_idx != ws.selected && hover_idx < project_count {
                let hover_vis = hover_idx.saturating_sub(ws.scroll_offset);
                let hover_y = layout.list_start_y + hover_vis as f32 * layout.list_item_height;
                if hover_y + layout.list_item_height <= layout.max_bottom {
                    rects.push(RectInstance {
                        pos: [layout.content_x - 8.0, hover_y],
                        size: [layout.content_width + 16.0, layout.list_item_height],
                        color: [SURFACE0[0], SURFACE0[1], SURFACE0[2], 0.5],
                    });
                }
            }
        }

        // ===== Text =====

        // Title "kode" — centered
        let title = "kode";
        let title_buf = text_r.create_buffer(title, width_f);
        let title_w = title.len() as f32 * text_r.cell_width;
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

        // Subtitle — centered
        let subtitle = "A Rust-native IDE";
        let subtitle_buf = text_r.create_buffer(subtitle, width_f);
        let subtitle_w = subtitle.len() as f32 * text_r.cell_width;
        text_areas.push(PreparedTextArea {
            buffer: subtitle_buf,
            left: (width_f - subtitle_w) / 2.0,
            top: layout.subtitle_y,
            bounds_left: 0.0,
            bounds_top: 0.0,
            bounds_right: width_f,
            bounds_bottom: height_f,
            color: to_glyphon_color(OVERLAY0),
        });

        // Button text — centered in button
        let btn_text = "Open Project  (o)";
        let btn_buf = text_r.create_buffer(btn_text, layout.button_w);
        let btn_text_w = btn_text.len() as f32 * text_r.cell_width;
        text_areas.push(PreparedTextArea {
            buffer: btn_buf,
            left: layout.button_x + (layout.button_w - btn_text_w) / 2.0,
            top: layout.button_y + 6.0,
            bounds_left: layout.button_x,
            bounds_top: layout.button_y,
            bounds_right: layout.button_x + layout.button_w,
            bounds_bottom: layout.button_y + layout.button_h,
            color: to_glyphon_color(TEXT_COLOR),
        });

        // Section header
        if project_count > 0 {
            let header = "Recent Projects";
            let header_buf = text_r.create_buffer(header, layout.content_width);
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
            let visible_end = (ws.scroll_offset + layout.visible_count).min(project_count);
            for i in ws.scroll_offset..visible_end {
                let vis_idx = i - ws.scroll_offset;
                let item_y = layout.list_start_y + vis_idx as f32 * layout.list_item_height;
                if item_y + layout.list_item_height > layout.max_bottom {
                    break;
                }

                let project = &ws.recent.projects[i];

                // Project name
                let name_prefix = if i == ws.selected { "▸ " } else { "  " };
                let name_text = format!("{}{}", name_prefix, project.name);
                let name_color = if i == ws.selected {
                    to_glyphon_color(BLUE)
                } else {
                    to_glyphon_color(TEXT_COLOR)
                };
                let name_buf = text_r.create_buffer(&name_text, layout.content_width);
                text_areas.push(PreparedTextArea {
                    buffer: name_buf,
                    left: layout.content_x,
                    top: item_y + 2.0,
                    bounds_left: layout.content_x,
                    bounds_top: item_y,
                    bounds_right: layout.content_x + layout.content_width,
                    bounds_bottom: item_y + layout.list_item_height,
                    color: name_color,
                });

                // Project path
                let path_text = format!(
                    "  {}",
                    WelcomeScreen::project_display_path(project)
                );
                let path_buf = text_r.create_buffer(&path_text, layout.content_width);
                text_areas.push(PreparedTextArea {
                    buffer: path_buf,
                    left: layout.content_x,
                    top: item_y + line_h + 2.0,
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
            let msg_buf = text_r.create_buffer(msg, layout.content_width);
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

        // Help text at bottom
        let help = "j/k: navigate  Enter: open  o: open folder  d: remove  q: quit";
        let help_buf = text_r.create_buffer(help, width_f);
        let help_w = help.len() as f32 * text_r.cell_width;
        text_areas.push(PreparedTextArea {
            buffer: help_buf,
            left: (width_f - help_w) / 2.0,
            top: height_f - line_h - 8.0,
            bounds_left: 0.0,
            bounds_top: 0.0,
            bounds_right: width_f,
            bounds_bottom: height_f,
            color: to_glyphon_color(OVERLAY0),
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
        let line_h = text_r.line_height;
        let cell_w = text_r.cell_width;

        // ===== Collect rectangles =====
        let mut rects = Vec::new();

        // Tab bar background
        rects.push(RectInstance {
            pos: [0.0, 0.0],
            size: [width_f, line_h + 4.0],
            color: MANTLE,
        });

        // Status bar background
        let status_y = height_f - line_h - 4.0;
        rects.push(RectInstance {
            pos: [0.0, status_y],
            size: [width_f, line_h + 4.0],
            color: MANTLE,
        });

        // Mode indicator in status bar
        let mode_name = app_view.mode.display_name();
        let mode_color = match app_view.mode {
            Mode::Normal => BLUE,
            Mode::Insert => GREEN,
            Mode::Visual | Mode::VisualLine => MAUVE,
            Mode::Command => YELLOW,
            _ => SURFACE1,
        };
        rects.push(RectInstance {
            pos: [0.0, status_y],
            size: [mode_name.len() as f32 * cell_w + 16.0, line_h + 4.0],
            color: mode_color,
        });

        // Pane backgrounds and borders
        for (pane_id, kode_rect) in &app_view.pane_rects {
            let px = kode_rect.x();
            let py = kode_rect.y() + line_h + 4.0;
            let pw = kode_rect.width();
            let ph = kode_rect.height().min(height_f - line_h * 2.0 - 8.0);

            rects.push(RectInstance {
                pos: [px, py],
                size: [pw, ph],
                color: BASE,
            });

            rects.push(RectInstance {
                pos: [px, py],
                size: [pw, 1.0],
                color: SURFACE0,
            });

            if px > 0.0 {
                rects.push(RectInstance {
                    pos: [px, py],
                    size: [1.0, ph],
                    color: SURFACE0,
                });
            }

            if let Some(pane) = app_view.panes.get(pane_id) {
                match pane.content {
                    PaneContent::Editor(doc_id) => {
                        let gutter_w = cell_w * 4.0;
                        rects.push(RectInstance {
                            pos: [px, py + 1.0],
                            size: [gutter_w, ph - 1.0],
                            color: MANTLE,
                        });

                        if let Some(doc) = app_view.documents.get(&doc_id) {
                            let cursor_line = doc.cursors.primary().line();
                            let cursor_y = py + 1.0 + (cursor_line as f32) * line_h;
                            if cursor_y < py + ph {
                                rects.push(RectInstance {
                                    pos: [px + gutter_w, cursor_y],
                                    size: [pw - gutter_w, line_h],
                                    color: SURFACE0,
                                });

                                let cursor_col = doc.cursors.primary().col();
                                rects.push(RectInstance {
                                    pos: [
                                        px + gutter_w + 4.0 + cursor_col as f32 * cell_w,
                                        cursor_y,
                                    ],
                                    size: [cell_w, line_h],
                                    color: TEXT_COLOR,
                                });
                            }
                        }
                    }
                    PaneContent::FileExplorer(explorer_id) => {
                        if let Some(explorer) = app_view.explorers.get(&explorer_id) {
                            let visible_idx =
                                explorer.cursor.saturating_sub(explorer.scroll_offset);
                            let cursor_y = py + 1.0 + visible_idx as f32 * line_h;
                            if cursor_y < py + ph && pane.focused {
                                rects.push(RectInstance {
                                    pos: [px, cursor_y],
                                    size: [pw, line_h],
                                    color: SURFACE0,
                                });
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

        let tab = app_view.session.active_tab();
        let tab_text = format!("  {}  ", tab.name);
        let tab_buf = text_r.create_buffer(&tab_text, width_f);
        text_areas.push(PreparedTextArea {
            buffer: tab_buf,
            left: 4.0,
            top: 2.0,
            bounds_left: 0.0,
            bounds_top: 0.0,
            bounds_right: width_f,
            bounds_bottom: line_h + 4.0,
            color: to_glyphon_color(TEXT_COLOR),
        });

        let status_text = format!(" {} ", mode_name.to_uppercase());
        let status_buf = text_r.create_buffer(&status_text, width_f);
        text_areas.push(PreparedTextArea {
            buffer: status_buf,
            left: 4.0,
            top: status_y + 2.0,
            bounds_left: 0.0,
            bounds_top: status_y,
            bounds_right: width_f,
            bounds_bottom: height_f,
            color: to_glyphon_color(CRUST),
        });

        for (pane_id, kode_rect) in &app_view.pane_rects {
            let px = kode_rect.x();
            let py = kode_rect.y() + line_h + 4.0;
            let pw = kode_rect.width();
            let ph = kode_rect.height().min(height_f - line_h * 2.0 - 8.0);

            if let Some(pane) = app_view.panes.get(pane_id) {
                match pane.content {
                    PaneContent::Editor(doc_id) => {
                        if let Some(doc) = app_view.documents.get(&doc_id) {
                            let gutter_w = cell_w * 4.0;
                            let visible_lines = (ph / line_h) as usize;

                            let title = doc.title();
                            let title_buf =
                                text_r.create_buffer(&format!(" {} ", title), pw);
                            text_areas.push(PreparedTextArea {
                                buffer: title_buf,
                                left: px + gutter_w,
                                top: py - line_h + 2.0,
                                bounds_left: px,
                                bounds_top: py - line_h,
                                bounds_right: px + pw,
                                bounds_bottom: py,
                                color: to_glyphon_color(OVERLAY0),
                            });

                            for i in 0..visible_lines.min(doc.buffer.line_count()) {
                                let line_y = py + 1.0 + i as f32 * line_h;
                                if line_y + line_h > py + ph {
                                    break;
                                }

                                let line_num = format!("{:>3} ", i + 1);
                                let gutter_buf = text_r.create_buffer(&line_num, gutter_w);
                                text_areas.push(PreparedTextArea {
                                    buffer: gutter_buf,
                                    left: px + 2.0,
                                    top: line_y,
                                    bounds_left: px,
                                    bounds_top: py,
                                    bounds_right: px + gutter_w,
                                    bounds_bottom: py + ph,
                                    color: to_glyphon_color(OVERLAY0),
                                });

                                let line_text =
                                    doc.buffer.line_to_string(i).unwrap_or_default();
                                if !line_text.is_empty() {
                                    let trimmed = line_text.trim_end_matches('\n');
                                    let code_buf = text_r
                                        .create_buffer(trimmed, pw - gutter_w - 8.0);
                                    text_areas.push(PreparedTextArea {
                                        buffer: code_buf,
                                        left: px + gutter_w + 4.0,
                                        top: line_y,
                                        bounds_left: px + gutter_w,
                                        bounds_top: py,
                                        bounds_right: px + pw,
                                        bounds_bottom: py + ph,
                                        color: to_glyphon_color(TEXT_COLOR),
                                    });
                                }
                            }
                        }
                    }
                    PaneContent::FileExplorer(explorer_id) => {
                        if let Some(explorer) = app_view.explorers.get(&explorer_id) {
                            let visible_lines = (ph / line_h) as usize;

                            let title_buf = text_r.create_buffer(" explorer ", pw);
                            text_areas.push(PreparedTextArea {
                                buffer: title_buf,
                                left: px + 4.0,
                                top: py - line_h + 2.0,
                                bounds_left: px,
                                bounds_top: py - line_h,
                                bounds_right: px + pw,
                                bounds_bottom: py,
                                color: to_glyphon_color(OVERLAY0),
                            });

                            for i in 0..visible_lines {
                                let entry_idx = explorer.scroll_offset + i;
                                if entry_idx >= explorer.entries.len() {
                                    break;
                                }
                                let entry = &explorer.entries[entry_idx];
                                let entry_y = py + 1.0 + i as f32 * line_h;
                                if entry_y + line_h > py + ph {
                                    break;
                                }

                                let indent = "  ".repeat(entry.depth);
                                let arrow = if entry.is_dir {
                                    if entry.expanded {
                                        "▾ "
                                    } else {
                                        "▸ "
                                    }
                                } else {
                                    "  "
                                };
                                let display =
                                    format!("{}{}{}", indent, arrow, entry.name);
                                let color = if entry.is_dir {
                                    to_glyphon_color(BLUE)
                                } else {
                                    to_glyphon_color(TEXT_COLOR)
                                };

                                let entry_buf =
                                    text_r.create_buffer(&display, pw - 4.0);
                                text_areas.push(PreparedTextArea {
                                    buffer: entry_buf,
                                    left: px + 4.0,
                                    top: entry_y,
                                    bounds_left: px,
                                    bounds_top: py,
                                    bounds_right: px + pw,
                                    bounds_bottom: py + ph,
                                    color,
                                });
                            }
                        }
                    }
                    PaneContent::Terminal(_) => {
                        let term_buf =
                            text_r.create_buffer("Terminal (GPU pending)", pw);
                        text_areas.push(PreparedTextArea {
                            buffer: term_buf,
                            left: px + 8.0,
                            top: py + 8.0,
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

    // ─────────────────────────────────────────────
    //  Key Handling
    // ─────────────────────────────────────────────

    fn handle_key(&mut self, key: kode_core::event::KeyEvent) {
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
                        let (cx, cy) = self.cursor_pos;
                        let action = ws.handle_click(cx as f32, cy as f32, &layout);
                        self.process_welcome_action(action);
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(key) = translate_winit_key(&event, &self.modifiers) {
                    self.handle_key(key);

                    if self.should_quit {
                        event_loop.exit();
                        return;
                    }

                    if let AppScreen::Editor(state) = &self.screen {
                        if !state.is_running() {
                            event_loop.exit();
                            return;
                        }
                    }

                    if let Some(window) = &self.window {
                        window.request_redraw();
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
