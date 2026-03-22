use std::collections::HashMap;
use std::path::PathBuf;

use kode_core::config::Config;
use kode_core::error::KodeResult;
use kode_core::event::KodeEvent;
use kode_core::geometry::Rect;
use kode_editor::document::Document;
use kode_keymap::mode::Mode;
use kode_keymap::parser::{Action, KeyParser, ParseResult};
use kode_keymap::workspace_keys::WorkspaceAction;
use kode_lsp::LspManager;
use kode_plugin::PluginManager;
use kode_terminal::Terminal;
use kode_workspace::layout::{Direction, FocusDirection, LayoutNode};
use kode_state::FileExplorer;
use kode_workspace::pane::{Pane, PaneContent, PaneId};
use kode_workspace::persistence;
use kode_workspace::session::Session;
use kode_workspace::tab::Tab;

use crate::cli::Args;

/// Main application state.
pub struct App {
    pub config: Config,
    pub documents: HashMap<usize, Document>,
    pub terminals: HashMap<usize, Terminal>,
    pub panes: HashMap<PaneId, Pane>,
    pub session: Session,
    pub key_parser: KeyParser,
    pub lsp_manager: LspManager,
    pub plugin_manager: PluginManager,
    pub focused_pane: PaneId,
    next_doc_id: usize,
    next_term_id: usize,
    next_pane_id: usize,
    next_tab_id: usize,
    running: bool,
    zoomed_pane: Option<PaneId>,
    viewport: Rect,
}

impl App {
    pub fn new(config: Config) -> Self {
        let pane = Pane::editor(0, 0);
        let tab = Tab::new(0, "main".into(), 0);
        let session = Session::new(tab);

        let mut documents = HashMap::new();
        documents.insert(0, Document::new());

        let mut panes = HashMap::new();
        let mut p = pane;
        p.focused = true;
        panes.insert(0, p);

        Self {
            config,
            documents,
            terminals: HashMap::new(),
            panes,
            session,
            key_parser: KeyParser::new(),
            lsp_manager: LspManager::new(),
            plugin_manager: {
                let mut pm = PluginManager::new();
                pm.discover(&PluginManager::default_plugin_dir());
                pm
            },
            focused_pane: 0,
            next_doc_id: 1,
            next_term_id: 0,
            next_pane_id: 1,
            next_tab_id: 1,
            running: true,
            zoomed_pane: None,
            viewport: Rect::new(0.0, 0.0, 800.0, 600.0),
        }
    }

    pub fn open_file(&mut self, path: PathBuf) -> KodeResult<usize> {
        let doc = Document::from_file(path).map_err(kode_core::error::KodeError::Io)?;
        let id = self.next_doc_id;
        self.next_doc_id += 1;
        self.documents.insert(id, doc);
        Ok(id)
    }

    pub fn mode(&self) -> Mode {
        self.key_parser.mode()
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Handle a workspace action (tmux-style commands).
    pub fn handle_workspace_action(&mut self, action: WorkspaceAction) {
        match action {
            WorkspaceAction::SplitVertical => {
                self.split_pane(Direction::Vertical);
            }
            WorkspaceAction::SplitHorizontal => {
                self.split_pane(Direction::Horizontal);
            }
            WorkspaceAction::FocusLeft => self.focus_direction(FocusDirection::Left),
            WorkspaceAction::FocusRight => self.focus_direction(FocusDirection::Right),
            WorkspaceAction::FocusUp => self.focus_direction(FocusDirection::Up),
            WorkspaceAction::FocusDown => self.focus_direction(FocusDirection::Down),
            WorkspaceAction::ResizeLeft(_) => {
                self.resize_focused(-0.05);
            }
            WorkspaceAction::ResizeRight(_) => {
                self.resize_focused(0.05);
            }
            WorkspaceAction::ResizeUp(_) => {
                self.resize_focused(-0.05);
            }
            WorkspaceAction::ResizeDown(_) => {
                self.resize_focused(0.05);
            }
            WorkspaceAction::NewTab => {
                self.new_tab();
            }
            WorkspaceAction::NextTab => {
                self.session.next_tab();
            }
            WorkspaceAction::PrevTab => {
                self.session.prev_tab();
            }
            WorkspaceAction::CloseTab => {
                let idx = self.session.active_tab;
                self.session.close_tab(idx);
            }
            WorkspaceAction::ClosePane => {
                self.close_focused_pane();
            }
            WorkspaceAction::ZoomPane => {
                self.toggle_zoom();
            }
            WorkspaceAction::NewTerminal => {
                self.spawn_terminal_pane();
            }
            WorkspaceAction::ToggleExplorer => {
                // Toggle handled in TUI mode only
                tracing::info!("Explorer toggle (GPU mode not supported yet)");
            }
            WorkspaceAction::Detach => {
                self.save_and_quit();
            }
            WorkspaceAction::Equalize => {
                kode_workspace::resize::equalize_panes(
                    &mut self.session.active_tab_mut().layout,
                );
            }
        }
    }

    /// Handle a key event through the key parser.
    pub fn handle_key_event(&mut self, key: kode_core::event::KeyEvent) {
        match self.key_parser.parse(key) {
            ParseResult::Complete(Action::Workspace(action)) => {
                self.handle_workspace_action(action);
            }
            ParseResult::Complete(Action::Command(cmd)) => {
                tracing::debug!("Editor command: {:?}", cmd);
                // TODO: dispatch to focused editor pane
            }
            ParseResult::Complete(Action::Motion(motion)) => {
                tracing::debug!("Motion: {:?}", motion);
                // TODO: apply motion to focused editor
            }
            ParseResult::Complete(Action::ChangeMode(mode)) => {
                tracing::debug!("Mode changed to: {}", mode.display_name());
            }
            ParseResult::Complete(Action::CommandLine(cmd)) => {
                self.handle_command_line(&cmd);
            }
            ParseResult::Complete(_) => {}
            ParseResult::Pending => {}
            ParseResult::None => {}
        }
    }

    fn handle_command_line(&mut self, cmd: &str) {
        match cmd {
            "w" => {
                if let Some(pane) = self.panes.get(&self.focused_pane) {
                    if let PaneContent::Editor(doc_id) = pane.content {
                        if let Some(doc) = self.documents.get_mut(&doc_id) {
                            let _ = doc.save();
                        }
                    }
                }
            }
            "q" => self.quit(),
            "wq" => {
                self.handle_command_line("w");
                self.quit();
            }
            "beans" => {
                tracing::info!("Requesting Spring beans...");
                // Spring beans will be fetched via LspManager's spring/beans method
            }
            "endpoints" => {
                tracing::info!("Requesting Spring endpoints...");
                // Spring endpoints will be fetched via spring/endpoints method
            }
            "plugins" => {
                let plugins = self.plugin_manager.list_plugins();
                for p in &plugins {
                    tracing::info!(
                        "  {} v{} — {}",
                        p.name,
                        p.version,
                        p.description.as_deref().unwrap_or("")
                    );
                }
                if plugins.is_empty() {
                    tracing::info!("No plugins loaded");
                }
            }
            cmd if cmd.starts_with("plugin-enable ") => {
                let name = cmd.strip_prefix("plugin-enable ").unwrap_or("");
                if self.plugin_manager.enable_plugin(name) {
                    tracing::info!("Plugin '{}' enabled", name);
                } else {
                    tracing::warn!("Plugin '{}' not found", name);
                }
            }
            cmd if cmd.starts_with("plugin-disable ") => {
                let name = cmd.strip_prefix("plugin-disable ").unwrap_or("");
                if self.plugin_manager.disable_plugin(name) {
                    tracing::info!("Plugin '{}' disabled", name);
                } else {
                    tracing::warn!("Plugin '{}' not found", name);
                }
            }
            cmd if cmd.starts_with("gradle ") => {
                let task = cmd.strip_prefix("gradle ").unwrap_or("");
                tracing::info!("Running Gradle task: {}", task);
                // Gradle task will be executed via spring/runTask method
            }
            _ => {
                tracing::warn!("Unknown command: :{}", cmd);
            }
        }
    }

    fn split_pane(&mut self, direction: Direction) {
        let new_doc_id = self.next_doc_id;
        self.next_doc_id += 1;
        self.documents.insert(new_doc_id, Document::new());

        let new_pane_id = self.next_pane_id;
        self.next_pane_id += 1;
        self.panes
            .insert(new_pane_id, Pane::editor(new_pane_id, new_doc_id));

        self.session
            .active_tab_mut()
            .layout
            .split(self.focused_pane, new_pane_id, direction);

        self.set_focus(new_pane_id);
    }

    fn spawn_terminal_pane(&mut self) {
        let term_id = self.next_term_id;
        self.next_term_id += 1;

        match Terminal::spawn(term_id, 24, 80) {
            Ok(terminal) => {
                self.terminals.insert(term_id, terminal);

                let pane_id = self.next_pane_id;
                self.next_pane_id += 1;
                self.panes
                    .insert(pane_id, Pane::terminal(pane_id, term_id));

                self.session
                    .active_tab_mut()
                    .layout
                    .split(self.focused_pane, pane_id, Direction::Horizontal);

                self.set_focus(pane_id);
            }
            Err(e) => {
                tracing::error!("Failed to spawn terminal: {}", e);
            }
        }
    }

    fn focus_direction(&mut self, dir: FocusDirection) {
        let rects = self
            .session
            .active_tab()
            .layout
            .compute_rects(self.viewport);

        if let Some(target) =
            kode_workspace::layout::find_pane_in_direction(self.focused_pane, dir, &rects)
        {
            self.set_focus(target);
        }
    }

    fn set_focus(&mut self, pane_id: PaneId) {
        if let Some(old) = self.panes.get_mut(&self.focused_pane) {
            old.focused = false;
        }
        if let Some(new) = self.panes.get_mut(&pane_id) {
            new.focused = true;
        }
        self.focused_pane = pane_id;
    }

    fn resize_focused(&mut self, delta: f32) {
        kode_workspace::resize::resize_pane(
            &mut self.session.active_tab_mut().layout,
            self.focused_pane,
            delta,
        );
    }

    fn close_focused_pane(&mut self) {
        let layout = &mut self.session.active_tab_mut().layout;
        let pane_ids = layout.pane_ids();

        if pane_ids.len() <= 1 {
            return; // Don't close the last pane
        }

        // Find a neighbor to focus before removing
        let rects = layout.compute_rects(self.viewport);
        let next_focus = kode_workspace::layout::find_pane_in_direction(
            self.focused_pane,
            FocusDirection::Left,
            &rects,
        )
        .or_else(|| {
            kode_workspace::layout::find_pane_in_direction(
                self.focused_pane,
                FocusDirection::Right,
                &rects,
            )
        });

        // Remove the pane from layout
        let removed = self.focused_pane;
        layout.remove(removed);

        // Clean up pane resources
        if let Some(pane) = self.panes.remove(&removed) {
            match pane.content {
                PaneContent::Editor(doc_id) => {
                    self.documents.remove(&doc_id);
                }
                PaneContent::Terminal(term_id) => {
                    self.terminals.remove(&term_id);
                }
                PaneContent::BeanExplorer | PaneContent::EndpointExplorer | PaneContent::FileExplorer(_) => {
                    // No resources to clean up
                }
            }
        }

        if let Some(next) = next_focus {
            self.set_focus(next);
        } else if let Some(&first) = self
            .session
            .active_tab()
            .layout
            .pane_ids()
            .first()
        {
            self.set_focus(first);
        }
    }

    fn new_tab(&mut self) {
        let doc_id = self.next_doc_id;
        self.next_doc_id += 1;
        self.documents.insert(doc_id, Document::new());

        let pane_id = self.next_pane_id;
        self.next_pane_id += 1;
        self.panes.insert(pane_id, Pane::editor(pane_id, doc_id));

        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;
        let tab = Tab::new(tab_id, format!("tab-{}", tab_id), pane_id);
        self.session.add_tab(tab);
        self.set_focus(pane_id);
    }

    fn toggle_zoom(&mut self) {
        if self.zoomed_pane.is_some() {
            self.zoomed_pane = None;
        } else {
            self.zoomed_pane = Some(self.focused_pane);
        }
    }

    fn save_and_quit(&mut self) {
        let editor_files: HashMap<usize, Option<PathBuf>> = self
            .documents
            .iter()
            .map(|(id, doc)| (*id, doc.file_path.clone()))
            .collect();

        let terminal_cwds: HashMap<usize, PathBuf> = self
            .terminals
            .iter()
            .map(|(id, term)| (*id, term.cwd.clone()))
            .collect();

        let state =
            persistence::save_session(&self.session, &self.panes, &editor_files, &terminal_cwds);

        let path = persistence::default_session_path();
        match persistence::save_to_file(&state, &path) {
            Ok(()) => tracing::info!("Session saved to {}", path.display()),
            Err(e) => tracing::error!("Failed to save session: {}", e),
        }

        self.quit();
    }

    /// Get pane layout rects, respecting zoom.
    pub fn pane_rects(&self) -> Vec<(PaneId, Rect)> {
        if let Some(zoomed) = self.zoomed_pane {
            vec![(zoomed, self.viewport)]
        } else {
            self.session.active_tab().layout.compute_rects(self.viewport)
        }
    }
}

/// Run the application in TUI mode.
pub fn run_tui(args: Args) -> KodeResult<()> {
    let config = if let Some(ref path) = args.config {
        Config::load(path)?
    } else {
        Config::default()
    };

    let mut documents = HashMap::new();
    let mut focused_doc = 0;

    // Open files from CLI arguments, or create empty doc
    if args.files.is_empty() {
        documents.insert(0, Document::new());
    } else {
        for (idx, path) in args.files.iter().enumerate() {
            match Document::from_file(path.clone()) {
                Ok(doc) => {
                    documents.insert(idx, doc);
                    focused_doc = idx;
                }
                Err(e) => {
                    tracing::warn!("Failed to open {}: {}", path.display(), e);
                    if documents.is_empty() {
                        documents.insert(idx, Document::new());
                    }
                }
            }
        }
    }

    // Create explorer
    let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let mut explorers = HashMap::new();
    explorers.insert(0, FileExplorer::new(0, root));

    // Explorer pane (id=0) on the left, editor pane (id=1) on the right
    let explorer_pane_id: PaneId = 0;
    let editor_pane_id: PaneId = 1;

    let mut panes = HashMap::new();
    panes.insert(explorer_pane_id, Pane::file_explorer(explorer_pane_id, 0));
    let mut editor_pane = Pane::editor(editor_pane_id, focused_doc);
    editor_pane.focused = true;
    panes.insert(editor_pane_id, editor_pane);

    // Layout: 25% explorer | 75% editor
    let layout = LayoutNode::Split {
        direction: Direction::Vertical,
        ratio: 0.25,
        first: Box::new(LayoutNode::Leaf(explorer_pane_id)),
        second: Box::new(LayoutNode::Leaf(editor_pane_id)),
    };
    let mut tab = Tab::new(0, "main".into(), editor_pane_id);
    tab.layout = layout;
    let session = Session::new(tab);

    let app_state = kode_tui::app_bridge::create_app_state(
        documents,
        HashMap::new(),
        explorers,
        panes,
        session,
        editor_pane_id,
    );

    let mut tui_app = kode_tui::TuiApp::new(app_state);
    kode_tui::event_loop::run(&mut tui_app)
}

/// Run the application in GPU mode with a native window.
pub fn run(args: Args) -> KodeResult<()> {
    use kode_gpu::gpu_app::AppScreen;

    if args.files.is_empty() {
        // No files specified — show Welcome Screen
        let welcome = kode_gpu::welcome_screen::WelcomeScreen::new();
        kode_gpu::run_gpu(AppScreen::Welcome(welcome))
    } else {
        // Files specified — go straight to editor
        let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        // Record project in recent list
        let recent_path = kode_workspace::recent_projects::default_recent_projects_path();
        let mut recent = kode_workspace::recent_projects::load_recent_projects(&recent_path)
            .unwrap_or_default();
        kode_workspace::recent_projects::add_recent_project(&mut recent, root.clone());
        let _ = kode_workspace::recent_projects::save_recent_projects(&recent, &recent_path);

        let state = kode_gpu::project::build_editor_state(root);
        kode_gpu::run_gpu(AppScreen::Editor(state))
    }
}
