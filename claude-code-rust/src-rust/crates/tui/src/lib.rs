// cc-tui: Terminal UI using ratatui + crossterm for the Claude Code Rust port.
//
// This crate provides the interactive terminal interface including:
// - Message display with syntax highlighting
// - Input prompt with history
// - Streaming response rendering
// - Tool execution progress display
// - Permission dialogs
// - Cost/token tracking display

use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};

pub use app::App;

// ---------------------------------------------------------------------------
// Public re-exports for slash command helpers
// ---------------------------------------------------------------------------

pub use input::{is_slash_command, parse_slash_command};

// ---------------------------------------------------------------------------
// app module – main application state and event loop
// ---------------------------------------------------------------------------
pub mod app {
    use crate::render;
    use cc_core::config::Config;
    use cc_core::cost::CostTracker;
    use cc_core::types::{Message, Role};
    use cc_query::QueryEvent;
    use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::CrosstermBackend;
    use ratatui::Terminal;
    use std::io::Stdout;
    use std::sync::Arc;
    use tracing::debug;

    // -----------------------------------------------------------------------
    // Supporting types
    // -----------------------------------------------------------------------

    /// Status of an active or completed tool call.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ToolStatus {
        Running,
        Done,
        Error,
    }

    /// Represents an active or completed tool invocation visible in the UI.
    #[derive(Debug, Clone)]
    pub struct ToolUseBlock {
        pub id: String,
        pub name: String,
        pub status: ToolStatus,
        pub output_preview: Option<String>,
    }

    /// A single option inside a permission request dialog.
    #[derive(Debug, Clone)]
    pub struct PermissionOption {
        pub label: String,
        pub key: char,
    }

    /// State for an in-flight permission request popup.
    #[derive(Debug, Clone)]
    pub struct PermissionRequest {
        pub tool_use_id: String,
        pub tool_name: String,
        pub description: String,
        pub selected_option: usize,
        pub options: Vec<PermissionOption>,
    }

    impl PermissionRequest {
        /// Create a standard Allow / Allow-always / Deny dialog.
        pub fn standard(tool_use_id: String, tool_name: String, description: String) -> Self {
            Self {
                tool_use_id,
                tool_name,
                description,
                selected_option: 0,
                options: vec![
                    PermissionOption {
                        label: "Allow once".to_string(),
                        key: 'y',
                    },
                    PermissionOption {
                        label: "Allow always".to_string(),
                        key: 'a',
                    },
                    PermissionOption {
                        label: "Deny".to_string(),
                        key: 'n',
                    },
                ],
            }
        }
    }

    /// State for Ctrl+R history search mode.
    #[derive(Debug, Clone)]
    pub struct HistorySearch {
        pub query: String,
        /// Indices into `input_history` that match the current query.
        pub matches: Vec<usize>,
        /// Which match is currently highlighted.
        pub selected: usize,
    }

    impl HistorySearch {
        pub fn new() -> Self {
            Self {
                query: String::new(),
                matches: Vec::new(),
                selected: 0,
            }
        }

        /// Re-compute matches against the given history slice.
        pub fn update_matches(&mut self, history: &[String]) {
            let q = self.query.to_lowercase();
            self.matches = history
                .iter()
                .enumerate()
                .filter_map(|(i, s)| {
                    if s.to_lowercase().contains(&q) {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();
            // Clamp selected to valid range
            if !self.matches.is_empty() && self.selected >= self.matches.len() {
                self.selected = self.matches.len() - 1;
            }
        }

        /// Return the currently selected history entry, if any.
        pub fn current_entry<'a>(&self, history: &'a [String]) -> Option<&'a str> {
            self.matches
                .get(self.selected)
                .and_then(|&i| history.get(i))
                .map(String::as_str)
        }
    }

    // -----------------------------------------------------------------------
    // App struct
    // -----------------------------------------------------------------------

    /// The top-level TUI application.
    pub struct App {
        // Core state
        pub config: Config,
        pub cost_tracker: Arc<CostTracker>,
        pub messages: Vec<Message>,
        pub input: String,
        pub input_history: Vec<String>,
        pub history_index: Option<usize>,
        pub scroll_offset: usize,
        pub is_streaming: bool,
        pub streaming_text: String,
        pub status_message: Option<String>,
        pub should_quit: bool,
        pub show_help: bool,

        // Extended state
        pub tool_use_blocks: Vec<ToolUseBlock>,
        pub permission_request: Option<PermissionRequest>,
        pub frame_count: u64,
        pub token_count: u32,
        pub cost_usd: f64,
        pub model_name: String,
        pub agent_status: Vec<(String, String)>,
        pub history_search: Option<HistorySearch>,

        // Cursor position within input (byte offset)
        pub cursor_pos: usize,
    }

    impl App {
        pub fn new(config: Config, cost_tracker: Arc<CostTracker>) -> Self {
            let model_name = config.effective_model().to_string();
            Self {
                config,
                cost_tracker,
                messages: Vec::new(),
                input: String::new(),
                input_history: Vec::new(),
                history_index: None,
                scroll_offset: 0,
                is_streaming: false,
                streaming_text: String::new(),
                status_message: None,
                should_quit: false,
                show_help: false,
                tool_use_blocks: Vec::new(),
                permission_request: None,
                frame_count: 0,
                token_count: 0,
                cost_usd: 0.0,
                model_name,
                agent_status: Vec::new(),
                history_search: None,
                cursor_pos: 0,
            }
        }

        /// Update the active model name (also updates cost tracker).
        pub fn set_model(&mut self, model: String) {
            self.cost_tracker.set_model(&model);
            self.model_name = model;
        }

        /// Add a message directly (e.g. from a non-streaming source).
        pub fn add_message(&mut self, role: Role, text: String) {
            self.messages.push(match role {
                Role::User => Message::user(text),
                Role::Assistant => Message::assistant(text),
            });
        }

        /// Take the current input buffer, push it to history, and return it.
        pub fn take_input(&mut self) -> String {
            let input = std::mem::take(&mut self.input);
            self.cursor_pos = 0;
            if !input.is_empty() {
                self.input_history.push(input.clone());
                self.history_index = None;
            }
            input
        }

        // -------------------------------------------------------------------
        // Event handling
        // -------------------------------------------------------------------

        /// Process a keyboard event. Returns `true` when the input should be
        /// submitted (Enter pressed with no blocking dialog).
        pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
            // History-search mode intercepts most keys
            if self.history_search.is_some() {
                return self.handle_history_search_key(key);
            }

            // Permission dialog mode intercepts most keys
            if self.permission_request.is_some() {
                self.handle_permission_key(key);
                return false;
            }

            match key.code {
                // ---- Quit / cancel ----------------------------------------
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.is_streaming {
                        self.is_streaming = false;
                        self.streaming_text.clear();
                        self.tool_use_blocks.clear();
                        self.status_message = Some("Cancelled.".to_string());
                    } else {
                        self.should_quit = true;
                    }
                }
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.input.is_empty() {
                        self.should_quit = true;
                    }
                }

                // ---- History search ----------------------------------------
                KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let mut hs = HistorySearch::new();
                    hs.update_matches(&self.input_history);
                    self.history_search = Some(hs);
                }

                // ---- Help overlay ------------------------------------------
                KeyCode::F(1) => {
                    self.show_help = !self.show_help;
                }
                KeyCode::Char('?') if key.modifiers.is_empty() && !self.is_streaming => {
                    self.show_help = !self.show_help;
                }

                // ---- Text entry (blocked while streaming) ------------------
                KeyCode::Char(c) if !self.is_streaming => {
                    // Insert at cursor position
                    let byte_pos = self.char_boundary_at(self.cursor_pos);
                    self.input.insert(byte_pos, c);
                    self.cursor_pos += c.len_utf8();
                }
                KeyCode::Backspace if !self.is_streaming => {
                    if self.cursor_pos > 0 {
                        // Remove the character immediately before the cursor
                        let end = self.char_boundary_at(self.cursor_pos);
                        // Find the start of the preceding character
                        let start = self.prev_char_boundary(end);
                        self.input.drain(start..end);
                        self.cursor_pos = start;
                    }
                }
                KeyCode::Delete if !self.is_streaming => {
                    let start = self.char_boundary_at(self.cursor_pos);
                    if start < self.input.len() {
                        let end = self.next_char_boundary(start);
                        self.input.drain(start..end);
                    }
                }
                KeyCode::Left if !self.is_streaming => {
                    if self.cursor_pos > 0 {
                        let end = self.char_boundary_at(self.cursor_pos);
                        self.cursor_pos = self.prev_char_boundary(end);
                    }
                }
                KeyCode::Right if !self.is_streaming => {
                    let start = self.char_boundary_at(self.cursor_pos);
                    if start < self.input.len() {
                        self.cursor_pos = self.next_char_boundary(start);
                    }
                }
                KeyCode::Home if !self.is_streaming => {
                    self.cursor_pos = 0;
                }
                KeyCode::End if !self.is_streaming => {
                    self.cursor_pos = self.input.len();
                }

                // ---- Submit ------------------------------------------------
                KeyCode::Enter if !self.is_streaming => {
                    return true;
                }

                // ---- Input history navigation ------------------------------
                KeyCode::Up => {
                    if !self.input_history.is_empty() {
                        let idx = match self.history_index {
                            Some(i) => i.saturating_sub(1),
                            None => self.input_history.len().saturating_sub(1),
                        };
                        self.history_index = Some(idx);
                        self.input = self.input_history[idx].clone();
                        self.cursor_pos = self.input.len();
                    }
                }
                KeyCode::Down => {
                    if let Some(idx) = self.history_index {
                        if idx + 1 < self.input_history.len() {
                            let new_idx = idx + 1;
                            self.history_index = Some(new_idx);
                            self.input = self.input_history[new_idx].clone();
                            self.cursor_pos = self.input.len();
                        } else {
                            self.history_index = None;
                            self.input.clear();
                            self.cursor_pos = 0;
                        }
                    }
                }

                // ---- Scroll ------------------------------------------------
                KeyCode::PageUp => {
                    self.scroll_offset = self.scroll_offset.saturating_add(10);
                }
                KeyCode::PageDown => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(10);
                }

                _ => {}
            }
            false
        }

        /// Handle a key event while in history-search mode.
        /// Returns `true` if the selected history entry should be submitted.
        fn handle_history_search_key(&mut self, key: KeyEvent) -> bool {
            let hs = match self.history_search.as_mut() {
                Some(h) => h,
                None => return false,
            };
            match key.code {
                KeyCode::Esc => {
                    self.history_search = None;
                }
                KeyCode::Enter => {
                    // Copy selected match into input
                    if let Some(entry) = hs.current_entry(&self.input_history) {
                        self.input = entry.to_string();
                        self.cursor_pos = self.input.len();
                    }
                    self.history_search = None;
                }
                KeyCode::Up => {
                    if hs.selected > 0 {
                        hs.selected -= 1;
                    }
                }
                KeyCode::Down => {
                    let max = hs.matches.len().saturating_sub(1);
                    if hs.selected < max {
                        hs.selected += 1;
                    }
                }
                KeyCode::Backspace => {
                    hs.query.pop();
                    let history = self.input_history.clone();
                    if let Some(hs) = self.history_search.as_mut() {
                        hs.update_matches(&history);
                    }
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    hs.query.push(c);
                    let history = self.input_history.clone();
                    if let Some(hs) = self.history_search.as_mut() {
                        hs.update_matches(&history);
                    }
                }
                _ => {}
            }
            false
        }

        /// Handle a key event while a permission dialog is active.
        fn handle_permission_key(&mut self, key: KeyEvent) {
            let pr = match self.permission_request.as_mut() {
                Some(p) => p,
                None => return,
            };

            match key.code {
                KeyCode::Char(c) => {
                    // Check if it's a digit (1-based option index)
                    if let Some(digit) = c.to_digit(10) {
                        let idx = (digit as usize).saturating_sub(1);
                        if idx < pr.options.len() {
                            pr.selected_option = idx;
                        }
                    } else {
                        // Try to match by key char
                        for (i, opt) in pr.options.iter().enumerate() {
                            if opt.key == c {
                                pr.selected_option = i;
                                // Auto-confirm on y/n/a shortcut
                                self.permission_request = None;
                                return;
                            }
                        }
                    }
                }
                KeyCode::Enter => {
                    self.permission_request = None;
                }
                KeyCode::Up => {
                    let pr = self.permission_request.as_mut().unwrap();
                    if pr.selected_option > 0 {
                        pr.selected_option -= 1;
                    }
                }
                KeyCode::Down => {
                    let pr = self.permission_request.as_mut().unwrap();
                    if pr.selected_option + 1 < pr.options.len() {
                        pr.selected_option += 1;
                    }
                }
                KeyCode::Esc => {
                    // Deny by default on Esc
                    self.permission_request = None;
                }
                _ => {}
            }
        }

        // -------------------------------------------------------------------
        // Query event handling
        // -------------------------------------------------------------------

        /// Process a query event from the agentic loop.
        pub fn handle_query_event(&mut self, event: QueryEvent) {
            match event {
                QueryEvent::Stream(stream_evt) => {
                    self.is_streaming = true;
                    match stream_evt {
                        cc_api::StreamEvent::ContentBlockDelta { delta, .. } => match delta {
                            cc_api::streaming::ContentDelta::TextDelta { text } => {
                                self.streaming_text.push_str(&text);
                            }
                            cc_api::streaming::ContentDelta::ThinkingDelta { thinking } => {
                                debug!(len = thinking.len(), "Thinking delta received");
                            }
                            _ => {}
                        },
                        cc_api::StreamEvent::MessageStop => {
                            self.is_streaming = false;
                            if !self.streaming_text.is_empty() {
                                let text = std::mem::take(&mut self.streaming_text);
                                self.messages.push(Message::assistant(text));
                            }
                        }
                        _ => {}
                    }
                }

                QueryEvent::ToolStart { tool_name, tool_id } => {
                    self.is_streaming = true;
                    self.status_message = Some(format!("Running {}…", tool_name));
                    // Replace or add the block
                    if let Some(existing) = self.tool_use_blocks.iter_mut().find(|b| b.id == tool_id) {
                        existing.status = ToolStatus::Running;
                        existing.output_preview = None;
                    } else {
                        self.tool_use_blocks.push(ToolUseBlock {
                            id: tool_id,
                            name: tool_name,
                            status: ToolStatus::Running,
                            output_preview: None,
                        });
                    }
                }

                QueryEvent::ToolEnd { tool_name: _, tool_id, result, is_error } => {
                    let preview = if result.len() > 120 {
                        format!("{}…", &result[..120])
                    } else {
                        result.clone()
                    };
                    if let Some(block) = self.tool_use_blocks.iter_mut().find(|b| b.id == tool_id) {
                        block.status = if is_error { ToolStatus::Error } else { ToolStatus::Done };
                        block.output_preview = Some(preview);
                    }
                    if is_error {
                        self.status_message = Some(format!("Tool error: {}", result));
                    } else {
                        self.status_message = None;
                    }
                }

                QueryEvent::TurnComplete { turn, stop_reason } => {
                    debug!(turn, stop_reason, "Turn complete");
                    self.is_streaming = false;
                    // Flush any remaining streaming text
                    if !self.streaming_text.is_empty() {
                        let text = std::mem::take(&mut self.streaming_text);
                        self.messages.push(Message::assistant(text));
                    }
                    // Clear active tool blocks after turn ends
                    self.tool_use_blocks.retain(|b| b.status != ToolStatus::Running);
                }

                QueryEvent::Status(msg) => {
                    self.status_message = Some(msg);
                }

                QueryEvent::Error(msg) => {
                    self.is_streaming = false;
                    self.streaming_text.clear();
                    // Show as a red system message
                    self.messages.push(Message::assistant(format!("Error: {}", msg)));
                    self.status_message = Some(format!("Error: {}", msg));
                }
            }
        }

        // -------------------------------------------------------------------
        // Main run loop
        // -------------------------------------------------------------------

        /// Run the TUI event loop. Returns `Some(input)` when the user submits
        /// a message, or `None` when the user quits.
        pub fn run(
            &mut self,
            terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        ) -> anyhow::Result<Option<String>> {
            loop {
                self.frame_count = self.frame_count.wrapping_add(1);

                // Sync cost/token counters from the shared tracker
                self.cost_usd = self.cost_tracker.total_cost_usd();
                self.token_count = self.cost_tracker.total_tokens() as u32;

                // Draw the frame
                terminal.draw(|f| render::render_app(f, self))?;

                // Poll for events with a short timeout so we can redraw for animation
                if event::poll(std::time::Duration::from_millis(50))? {
                    if let Event::Key(key) = event::read()? {
                        let should_submit = self.handle_key_event(key);
                        if self.should_quit {
                            return Ok(None);
                        }
                        if should_submit {
                            let input = self.take_input();
                            if !input.is_empty() {
                                return Ok(Some(input));
                            }
                        }
                    }
                }
            }
        }

        // -------------------------------------------------------------------
        // UTF-8 cursor helpers
        // -------------------------------------------------------------------

        /// Return the byte offset corresponding to the current cursor position,
        /// clamped to a valid char boundary.
        fn char_boundary_at(&self, pos: usize) -> usize {
            let len = self.input.len();
            let pos = pos.min(len);
            // Walk forward from `pos` until we hit a char boundary
            let mut p = pos;
            while p < len && !self.input.is_char_boundary(p) {
                p += 1;
            }
            p
        }

        /// Return the byte offset of the start of the character before `pos`.
        fn prev_char_boundary(&self, pos: usize) -> usize {
            if pos == 0 {
                return 0;
            }
            let mut p = pos - 1;
            while p > 0 && !self.input.is_char_boundary(p) {
                p -= 1;
            }
            p
        }

        /// Return the byte offset just past the character starting at `pos`.
        fn next_char_boundary(&self, pos: usize) -> usize {
            let len = self.input.len();
            if pos >= len {
                return len;
            }
            let mut p = pos + 1;
            while p < len && !self.input.is_char_boundary(p) {
                p += 1;
            }
            p
        }
    }
}

// ---------------------------------------------------------------------------
// input module – slash command utilities (public API)
// ---------------------------------------------------------------------------
pub mod input {
    /// Check whether a string looks like a slash command (e.g. "/help").
    pub fn is_slash_command(input: &str) -> bool {
        input.starts_with('/') && !input.starts_with("//")
    }

    /// Parse a slash command into `(command_name, args)`.
    /// Returns `("", "")` if the input is not a slash command.
    pub fn parse_slash_command(input: &str) -> (&str, &str) {
        if !is_slash_command(input) {
            return ("", "");
        }
        let without_slash = &input[1..];
        if let Some(space_idx) = without_slash.find(' ') {
            (
                &without_slash[..space_idx],
                without_slash[space_idx + 1..].trim(),
            )
        } else {
            (without_slash, "")
        }
    }
}

// ---------------------------------------------------------------------------
// render module – all ratatui rendering logic
// ---------------------------------------------------------------------------
pub mod render {
    use crate::app::{App, ToolStatus};
    use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
    use ratatui::Frame;
    use unicode_width::UnicodeWidthStr;

    // Braille spinner sequence
    const SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

    fn spinner_char(frame_count: u64) -> char {
        SPINNER[(frame_count as usize) % SPINNER.len()]
    }

    // -----------------------------------------------------------------------
    // Top-level layout
    // -----------------------------------------------------------------------

    /// Render the entire application into the current frame.
    pub fn render_app(frame: &mut Frame, app: &App) {
        let size = frame.area();

        // Three-row vertical layout: messages | input | status bar
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(5),
                Constraint::Length(1),
            ])
            .split(size);

        render_messages(frame, app, chunks[0]);
        render_input(frame, app, chunks[1]);
        render_status_bar(frame, app, chunks[2]);

        // Overlays (rendered on top)
        if let Some(ref pr) = app.permission_request {
            render_permission_dialog(frame, pr, size);
        }

        if app.show_help {
            render_help_overlay(frame, size);
        }

        if let Some(ref hs) = app.history_search {
            render_history_search(frame, hs, app, size);
        }
    }

    // -----------------------------------------------------------------------
    // Messages pane
    // -----------------------------------------------------------------------

    fn render_messages(frame: &mut Frame, app: &App, area: Rect) {
        let mut lines: Vec<Line> = Vec::new();

        for msg in &app.messages {
            let (prefix, style) = match msg.role {
                cc_core::types::Role::User => (
                    "You",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                cc_core::types::Role::Assistant => (
                    "Claude",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            };

            // Role header
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", prefix), style),
                Span::styled(
                    "─".repeat(area.width.saturating_sub(prefix.len() as u16 + 4) as usize),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));

            let text = msg.get_all_text();
            lines.extend(render_markdown_lines(&text, area.width as usize));
            lines.push(Line::from(""));
        }

        // Active tool-use blocks
        for block in &app.tool_use_blocks {
            let (icon, icon_style) = match block.status {
                ToolStatus::Running => (
                    format!("{}", spinner_char(app.frame_count)),
                    Style::default().fg(Color::Yellow),
                ),
                ToolStatus::Done => ("✓".to_string(), Style::default().fg(Color::Green)),
                ToolStatus::Error => ("✗".to_string(), Style::default().fg(Color::Red)),
            };

            let mut spans = vec![
                Span::styled(format!("  {} ", icon), icon_style),
                Span::styled(
                    format!("[{}]", block.name),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ];
            let suffix = match block.status {
                ToolStatus::Running => " running…".to_string(),
                ToolStatus::Done => " done".to_string(),
                ToolStatus::Error => " error".to_string(),
            };
            spans.push(Span::styled(
                suffix,
                Style::default().fg(Color::DarkGray),
            ));
            lines.push(Line::from(spans));

            if let Some(ref preview) = block.output_preview {
                let trimmed = preview.lines().next().unwrap_or("").to_string();
                if !trimmed.is_empty() {
                    lines.push(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(trimmed, Style::default().fg(Color::DarkGray)),
                    ]));
                }
            }
        }

        // In-flight streaming text
        if !app.streaming_text.is_empty() {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} Claude ", spinner_char(app.frame_count)),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "─".repeat(area.width.saturating_sub(12) as usize),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
            for raw_line in app.streaming_text.lines() {
                lines.push(Line::from(vec![Span::styled(
                    format!("  {}", raw_line),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::ITALIC),
                )]));
            }
        }

        // Compute total virtual height and apply scroll clamping
        let content_height = lines.len() as u16;
        let visible_height = area.height.saturating_sub(2);
        let max_scroll = content_height.saturating_sub(visible_height) as usize;
        let scroll = app.scroll_offset.min(max_scroll);

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Claude Code ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .wrap(Wrap { trim: false })
            .scroll((scroll as u16, 0));

        frame.render_widget(paragraph, area);
    }

    // -----------------------------------------------------------------------
    // Markdown-aware line renderer
    // -----------------------------------------------------------------------

    /// Convert a Markdown-ish string into a list of styled `Line` values.
    /// Handles:
    /// - ``` fenced code blocks (yellow border style)
    /// - `inline code` spans (yellow)
    /// - **bold** spans
    /// - Heading lines (# / ## / ###)
    /// - Plain text (wrapped to `width`)
    fn render_markdown_lines(text: &str, width: usize) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();
        let mut in_code_block = false;
        let mut code_lang = String::new();

        for raw in text.lines() {
            // Fenced code block open/close
            if raw.trim_start().starts_with("```") {
                if in_code_block {
                    // Close
                    lines.push(Line::from(vec![Span::styled(
                        "  └─────────────────────────────────────────────────".to_string(),
                        Style::default().fg(Color::Yellow),
                    )]));
                    in_code_block = false;
                    code_lang.clear();
                } else {
                    // Open
                    in_code_block = true;
                    code_lang = raw.trim_start().trim_start_matches('`').trim().to_string();
                    let lang_label = if code_lang.is_empty() {
                        String::new()
                    } else {
                        format!(" {} ", code_lang)
                    };
                    lines.push(Line::from(vec![Span::styled(
                        format!("  ┌─────────────────────{}", lang_label),
                        Style::default().fg(Color::Yellow),
                    )]));
                }
                continue;
            }

            if in_code_block {
                lines.push(Line::from(vec![
                    Span::styled("  │ ", Style::default().fg(Color::Yellow)),
                    Span::styled(raw.to_string(), Style::default().fg(Color::White)),
                ]));
                continue;
            }

            // Heading
            if raw.starts_with("### ") {
                lines.push(Line::from(vec![Span::styled(
                    format!("  {}", &raw[4..]),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )]));
                continue;
            }
            if raw.starts_with("## ") {
                lines.push(Line::from(vec![Span::styled(
                    format!("  {}", &raw[3..]),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )]));
                continue;
            }
            if raw.starts_with("# ") {
                lines.push(Line::from(vec![Span::styled(
                    format!("  {}", &raw[2..]),
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )]));
                continue;
            }

            // Plain text — parse inline markup and word-wrap
            let padded = format!("  {}", raw);
            let effective_width = width.saturating_sub(4);
            for wrapped_line in word_wrap(&padded, effective_width) {
                let spans = parse_inline_spans(wrapped_line);
                lines.push(Line::from(spans));
            }
        }

        // Unclosed code block (defensive)
        if in_code_block {
            lines.push(Line::from(vec![Span::styled(
                "  └─────────────────────────────────────────────────".to_string(),
                Style::default().fg(Color::Yellow),
            )]));
        }

        lines
    }

    /// Parse inline markup (`**bold**`, `` `code` ``) into styled spans.
    fn parse_inline_spans(text: String) -> Vec<Span<'static>> {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut remaining = text.as_str();

        while !remaining.is_empty() {
            // Look for the next markup delimiter
            let bold_pos = remaining.find("**");
            let code_pos = remaining.find('`');

            match (bold_pos, code_pos) {
                (None, None) => {
                    spans.push(Span::raw(remaining.to_string()));
                    break;
                }
                (Some(b), Some(c)) if c < b => {
                    // Inline code comes first
                    if c > 0 {
                        spans.push(Span::raw(remaining[..c].to_string()));
                    }
                    let after_tick = &remaining[c + 1..];
                    if let Some(end) = after_tick.find('`') {
                        spans.push(Span::styled(
                            after_tick[..end].to_string(),
                            Style::default().fg(Color::Yellow),
                        ));
                        remaining = &after_tick[end + 1..];
                    } else {
                        spans.push(Span::raw(remaining[c..].to_string()));
                        break;
                    }
                }
                (Some(b), _) => {
                    // Bold comes first
                    if b > 0 {
                        spans.push(Span::raw(remaining[..b].to_string()));
                    }
                    let after_stars = &remaining[b + 2..];
                    if let Some(end) = after_stars.find("**") {
                        spans.push(Span::styled(
                            after_stars[..end].to_string(),
                            Style::default().add_modifier(Modifier::BOLD),
                        ));
                        remaining = &after_stars[end + 2..];
                    } else {
                        spans.push(Span::raw(remaining[b..].to_string()));
                        break;
                    }
                }
                (None, Some(c)) => {
                    // Only inline code
                    if c > 0 {
                        spans.push(Span::raw(remaining[..c].to_string()));
                    }
                    let after_tick = &remaining[c + 1..];
                    if let Some(end) = after_tick.find('`') {
                        spans.push(Span::styled(
                            after_tick[..end].to_string(),
                            Style::default().fg(Color::Yellow),
                        ));
                        remaining = &after_tick[end + 1..];
                    } else {
                        spans.push(Span::raw(remaining[c..].to_string()));
                        break;
                    }
                }
            }
        }

        if spans.is_empty() {
            spans.push(Span::raw(String::new()));
        }
        spans
    }

    /// Naive word-wrap: split `text` into lines of at most `width` display columns.
    fn word_wrap(text: &str, width: usize) -> Vec<String> {
        if width == 0 || UnicodeWidthStr::width(text) <= width {
            return vec![text.to_string()];
        }

        let mut result = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0usize;

        for word in text.split_whitespace() {
            let word_w = UnicodeWidthStr::width(word);
            if current_width == 0 {
                current_line.push_str(word);
                current_width = word_w;
            } else if current_width + 1 + word_w <= width {
                current_line.push(' ');
                current_line.push_str(word);
                current_width += 1 + word_w;
            } else {
                result.push(std::mem::take(&mut current_line));
                current_line.push_str(word);
                current_width = word_w;
            }
        }
        if !current_line.is_empty() {
            result.push(current_line);
        }
        if result.is_empty() {
            result.push(text.to_string());
        }
        result
    }

    // -----------------------------------------------------------------------
    // Input pane
    // -----------------------------------------------------------------------

    fn render_input(frame: &mut Frame, app: &App, area: Rect) {
        // Decide border colour
        let border_style = if app.is_streaming {
            Style::default().fg(Color::DarkGray)
        } else if crate::input::is_slash_command(&app.input) {
            Style::default().fg(Color::Magenta)
        } else {
            Style::default().fg(Color::Cyan)
        };

        // Title hints
        let title = if app.is_streaming {
            " Streaming… (Ctrl+C to cancel) "
        } else if app.history_index.is_some() {
            " Input (history) "
        } else {
            " Input "
        };

        // Build the displayed text: prompt prefix + input content
        let prompt_prefix = if app.is_streaming { " … " } else { " > " };

        // Show slash command hint below the input when relevant
        let hint_line = if crate::input::is_slash_command(&app.input) {
            let (cmd, args) = crate::input::parse_slash_command(&app.input);
            format!("  /{} {}", cmd, args)
        } else {
            String::new()
        };

        let mut input_lines: Vec<Line> = Vec::new();
        input_lines.push(Line::from(vec![
            Span::styled(prompt_prefix, Style::default().fg(Color::Cyan)),
            Span::raw(app.input.clone()),
        ]));
        if !hint_line.is_empty() {
            input_lines.push(Line::from(vec![Span::styled(
                hint_line,
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::DIM),
            )]));
        }

        let paragraph = Paragraph::new(input_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(border_style),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);

        // Position the terminal cursor inside the input box
        if !app.is_streaming && app.permission_request.is_none() && app.history_search.is_none() {
            let cursor_col = " > ".len() + UnicodeWidthStr::width(app.input.as_str());
            let cursor_x = area.x + 1 + cursor_col.min((area.width as usize).saturating_sub(2)) as u16;
            let cursor_y = area.y + 1;
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }

    // -----------------------------------------------------------------------
    // Status bar
    // -----------------------------------------------------------------------

    fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
        let spinner = if app.is_streaming {
            format!("{} ", spinner_char(app.frame_count))
        } else {
            String::new()
        };

        let model = &app.model_name;
        let cost = app.cost_usd;
        let tokens = app.token_count;

        let right_info = format!("{}{} | ${:.4} | {} tok", spinner, model, cost, tokens);

        let left_info = if let Some(ref msg) = app.status_message {
            format!(" {}", msg)
        } else {
            String::new()
        };

        // Pad the left side so the right info is right-aligned
        let total_width = area.width as usize;
        let right_len = right_info.len();
        let left_len = left_info.len();
        let gap = total_width
            .saturating_sub(right_len + left_len)
            .saturating_sub(1);

        let bar_text = format!("{}{}{} ", left_info, " ".repeat(gap), right_info);

        let bar = Paragraph::new(bar_text)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));

        frame.render_widget(bar, area);
    }

    // -----------------------------------------------------------------------
    // Permission dialog overlay
    // -----------------------------------------------------------------------

    fn render_permission_dialog(
        frame: &mut Frame,
        pr: &crate::app::PermissionRequest,
        area: Rect,
    ) {
        // Center a dialog of fixed size
        let dialog_width = 60u16.min(area.width.saturating_sub(4));
        let dialog_height = (5 + pr.options.len() as u16 + 2).min(area.height.saturating_sub(4));

        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        // Clear background
        frame.render_widget(Clear, dialog_area);

        let mut lines: Vec<Line> = Vec::new();

        // Tool name header
        lines.push(Line::from(vec![
            Span::raw("  Tool: "),
            Span::styled(
                pr.tool_name.clone(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));

        // Description (word-wrapped)
        for desc_line in word_wrap(&pr.description, dialog_width as usize - 4) {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::raw(desc_line),
            ]));
        }
        lines.push(Line::from(""));

        // Options list
        for (i, opt) in pr.options.iter().enumerate() {
            let is_selected = i == pr.selected_option;
            let prefix = if is_selected { "  ► " } else { "    " };
            let key_style = if is_selected {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            lines.push(Line::from(vec![
                Span::raw(prefix),
                Span::styled(format!("[{}]", opt.key), key_style),
                Span::raw(format!(" {}", opt.label)),
            ]));
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Permission Required ")
            .border_style(Style::default().fg(Color::Yellow));

        let para = Paragraph::new(lines).block(block);
        frame.render_widget(para, dialog_area);
    }

    // -----------------------------------------------------------------------
    // Help overlay
    // -----------------------------------------------------------------------

    fn render_help_overlay(frame: &mut Frame, area: Rect) {
        let help_width = 50u16.min(area.width.saturating_sub(4));
        let help_height = 20u16.min(area.height.saturating_sub(4));
        let help_area = centered_rect(help_width, help_height, area);

        frame.render_widget(Clear, help_area);

        let lines = vec![
            Line::from(vec![Span::styled(
                " Key Bindings",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )]),
            Line::from(""),
            kb_line("Enter", "Submit message"),
            kb_line("Ctrl+C", "Cancel streaming / Quit"),
            kb_line("Ctrl+D", "Quit (empty input)"),
            kb_line("Up / Down", "Navigate input history"),
            kb_line("Ctrl+R", "Search input history"),
            kb_line("PageUp / PageDown", "Scroll messages"),
            kb_line("F1 / ?", "Toggle this help"),
            Line::from(""),
            Line::from(vec![Span::styled(
                " Permission Dialog",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )]),
            Line::from(""),
            kb_line("1 / 2 / 3", "Select option"),
            kb_line("y / a / n", "Allow / Always / Deny"),
            kb_line("Enter", "Confirm selection"),
            kb_line("Esc", "Deny (close dialog)"),
            Line::from(""),
            Line::from(vec![Span::styled(
                " press F1 or ? to close ",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )]),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .border_style(Style::default().fg(Color::Cyan));

        let para = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Left);
        frame.render_widget(para, help_area);
    }

    fn kb_line<'a>(key: &str, desc: &str) -> Line<'a> {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{:<20}", key),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(desc.to_string()),
        ])
    }

    // -----------------------------------------------------------------------
    // History search overlay
    // -----------------------------------------------------------------------

    fn render_history_search(
        frame: &mut Frame,
        hs: &crate::app::HistorySearch,
        app: &App,
        area: Rect,
    ) {
        let dialog_width = 60u16.min(area.width.saturating_sub(4));
        let visible_matches = 8usize;
        let dialog_height =
            (4 + visible_matches.min(hs.matches.len().max(1)) as u16).min(area.height.saturating_sub(4));
        let dialog_area = centered_rect(dialog_width, dialog_height, area);

        frame.render_widget(Clear, dialog_area);

        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(vec![
            Span::raw("  Search: "),
            Span::styled(
                hs.query.clone(),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(""));

        if hs.matches.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "  (no matches)",
                Style::default().fg(Color::DarkGray),
            )]));
        } else {
            // Show up to `visible_matches` entries, centred on selected
            let start = hs.selected.saturating_sub(visible_matches / 2);
            let end = (start + visible_matches).min(hs.matches.len());
            let start = end.saturating_sub(visible_matches).min(start);

            for (display_idx, &hist_idx) in hs.matches[start..end].iter().enumerate() {
                let real_idx = start + display_idx;
                let is_selected = real_idx == hs.selected;
                let entry = app
                    .input_history
                    .get(hist_idx)
                    .map(String::as_str)
                    .unwrap_or("");

                let truncated = if UnicodeWidthStr::width(entry) > (dialog_width as usize - 6) {
                    let mut s = entry.to_string();
                    s.truncate(dialog_width as usize - 9);
                    format!("{}…", s)
                } else {
                    entry.to_string()
                };

                let (prefix, style) = if is_selected {
                    (
                        "  ► ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    ("    ", Style::default().fg(Color::White))
                };
                lines.push(Line::from(vec![
                    Span::raw(prefix),
                    Span::styled(truncated, style),
                ]));
            }
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" History Search (Esc to cancel) ")
            .border_style(Style::default().fg(Color::Cyan));

        let para = Paragraph::new(lines).block(block);
        frame.render_widget(para, dialog_area);
    }

    // -----------------------------------------------------------------------
    // Geometry helpers
    // -----------------------------------------------------------------------

    /// Compute a centered `Rect` of the given `width` × `height` inside `area`.
    fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
        let x = area.x + area.width.saturating_sub(width) / 2;
        let y = area.y + area.height.saturating_sub(height) / 2;
        Rect {
            x,
            y,
            width: width.min(area.width),
            height: height.min(area.height),
        }
    }

}

// ---------------------------------------------------------------------------
// Terminal initialization / teardown helpers (public API)
// ---------------------------------------------------------------------------

/// Set up the terminal for TUI mode (raw mode + alternate screen).
pub fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore the terminal to its original state.
pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use app::{App, HistorySearch, PermissionRequest, ToolStatus, ToolUseBlock};
    use cc_core::config::Config;
    use cc_core::cost::CostTracker;
    use cc_core::types::Role;

    fn make_app() -> App {
        App::new(Config::default(), CostTracker::new())
    }

    // ---- input helpers ---------------------------------------------------

    #[test]
    fn test_is_slash_command() {
        assert!(input::is_slash_command("/help"));
        assert!(input::is_slash_command("/compact args"));
        assert!(!input::is_slash_command("//comment"));
        assert!(!input::is_slash_command("hello"));
        assert!(!input::is_slash_command(""));
    }

    #[test]
    fn test_parse_slash_command_no_args() {
        let (cmd, args) = input::parse_slash_command("/help");
        assert_eq!(cmd, "help");
        assert_eq!(args, "");
    }

    #[test]
    fn test_parse_slash_command_with_args() {
        let (cmd, args) = input::parse_slash_command("/compact  --force ");
        assert_eq!(cmd, "compact");
        assert_eq!(args, "--force");
    }

    #[test]
    fn test_parse_slash_command_non_slash() {
        let (cmd, args) = input::parse_slash_command("hello world");
        assert_eq!(cmd, "");
        assert_eq!(args, "");
    }

    // ---- App::take_input ------------------------------------------------

    #[test]
    fn test_take_input_pushes_history() {
        let mut app = make_app();
        app.input = "hello".to_string();
        let result = app.take_input();
        assert_eq!(result, "hello");
        assert_eq!(app.input, "");
        assert_eq!(app.input_history, vec!["hello"]);
        assert_eq!(app.cursor_pos, 0);
    }

    #[test]
    fn test_take_input_empty_does_not_push_history() {
        let mut app = make_app();
        let result = app.take_input();
        assert_eq!(result, "");
        assert!(app.input_history.is_empty());
    }

    // ---- add_message / set_model ----------------------------------------

    #[test]
    fn test_add_message() {
        let mut app = make_app();
        app.add_message(Role::User, "hi".to_string());
        assert_eq!(app.messages.len(), 1);
        assert_eq!(app.messages[0].role, Role::User);
    }

    #[test]
    fn test_set_model() {
        let mut app = make_app();
        app.set_model("claude-opus-4-5".to_string());
        assert_eq!(app.model_name, "claude-opus-4-5");
    }

    // ---- key handling ----------------------------------------------------

    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_ctrl_c_quits_when_idle() {
        let mut app = make_app();
        app.handle_key_event(ctrl(KeyCode::Char('c')));
        assert!(app.should_quit);
    }

    #[test]
    fn test_ctrl_c_cancels_streaming() {
        let mut app = make_app();
        app.is_streaming = true;
        app.streaming_text = "partial".to_string();
        app.handle_key_event(ctrl(KeyCode::Char('c')));
        assert!(!app.is_streaming);
        assert!(!app.should_quit);
        assert!(app.streaming_text.is_empty());
    }

    #[test]
    fn test_ctrl_d_quits_on_empty_input() {
        let mut app = make_app();
        app.handle_key_event(ctrl(KeyCode::Char('d')));
        assert!(app.should_quit);
    }

    #[test]
    fn test_ctrl_d_does_not_quit_with_input() {
        let mut app = make_app();
        app.input = "abc".to_string();
        app.handle_key_event(ctrl(KeyCode::Char('d')));
        assert!(!app.should_quit);
    }

    #[test]
    fn test_enter_returns_true() {
        let mut app = make_app();
        let submit = app.handle_key_event(key(KeyCode::Enter));
        assert!(submit);
    }

    #[test]
    fn test_enter_blocked_while_streaming() {
        let mut app = make_app();
        app.is_streaming = true;
        let submit = app.handle_key_event(key(KeyCode::Enter));
        assert!(!submit);
    }

    #[test]
    fn test_char_input_appends() {
        let mut app = make_app();
        app.handle_key_event(key(KeyCode::Char('h')));
        app.handle_key_event(key(KeyCode::Char('i')));
        assert_eq!(app.input, "hi");
    }

    #[test]
    fn test_backspace_removes_char() {
        let mut app = make_app();
        app.input = "hello".to_string();
        app.cursor_pos = 5;
        app.handle_key_event(key(KeyCode::Backspace));
        assert_eq!(app.input, "hell");
    }

    #[test]
    fn test_history_navigation() {
        let mut app = make_app();
        app.input_history = vec!["first".to_string(), "second".to_string()];
        app.handle_key_event(key(KeyCode::Up));
        assert_eq!(app.input, "second");
        app.handle_key_event(key(KeyCode::Up));
        assert_eq!(app.input, "first");
        app.handle_key_event(key(KeyCode::Down));
        assert_eq!(app.input, "second");
        app.handle_key_event(key(KeyCode::Down));
        assert_eq!(app.input, "");
        assert!(app.history_index.is_none());
    }

    #[test]
    fn test_page_scroll() {
        let mut app = make_app();
        app.handle_key_event(key(KeyCode::PageUp));
        assert_eq!(app.scroll_offset, 10);
        app.handle_key_event(key(KeyCode::PageDown));
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn test_f1_toggles_help() {
        let mut app = make_app();
        assert!(!app.show_help);
        app.handle_key_event(key(KeyCode::F(1)));
        assert!(app.show_help);
        app.handle_key_event(key(KeyCode::F(1)));
        assert!(!app.show_help);
    }

    // ---- QueryEvent handling --------------------------------------------

    #[test]
    fn test_handle_status_event() {
        let mut app = make_app();
        app.handle_query_event(cc_query::QueryEvent::Status("working".to_string()));
        assert_eq!(app.status_message.as_deref(), Some("working"));
    }

    #[test]
    fn test_handle_error_event() {
        let mut app = make_app();
        app.is_streaming = true;
        app.handle_query_event(cc_query::QueryEvent::Error("oops".to_string()));
        assert!(!app.is_streaming);
        assert_eq!(app.messages.len(), 1);
        assert!(app.messages[0].get_all_text().contains("oops"));
    }

    #[test]
    fn test_handle_tool_start_and_end() {
        let mut app = make_app();
        app.handle_query_event(cc_query::QueryEvent::ToolStart {
            tool_name: "Bash".to_string(),
            tool_id: "t1".to_string(),
        });
        assert_eq!(app.tool_use_blocks.len(), 1);
        assert_eq!(app.tool_use_blocks[0].status, ToolStatus::Running);

        app.handle_query_event(cc_query::QueryEvent::ToolEnd {
            tool_name: "Bash".to_string(),
            tool_id: "t1".to_string(),
            result: "output".to_string(),
            is_error: false,
        });
        assert_eq!(app.tool_use_blocks[0].status, ToolStatus::Done);
    }

    #[test]
    fn test_handle_tool_end_error() {
        let mut app = make_app();
        app.tool_use_blocks.push(ToolUseBlock {
            id: "t2".to_string(),
            name: "Read".to_string(),
            status: ToolStatus::Running,
            output_preview: None,
        });
        app.handle_query_event(cc_query::QueryEvent::ToolEnd {
            tool_name: "Read".to_string(),
            tool_id: "t2".to_string(),
            result: "file not found".to_string(),
            is_error: true,
        });
        assert_eq!(app.tool_use_blocks[0].status, ToolStatus::Error);
        assert!(app.status_message.is_some());
    }

    #[test]
    fn test_turn_complete_flushes_streaming_text() {
        let mut app = make_app();
        app.is_streaming = true;
        app.streaming_text = "partial response".to_string();
        app.handle_query_event(cc_query::QueryEvent::TurnComplete {
            turn: 1,
            stop_reason: "end_turn".to_string(),
        });
        assert!(!app.is_streaming);
        assert!(app.streaming_text.is_empty());
        assert_eq!(app.messages.len(), 1);
        assert_eq!(app.messages[0].get_all_text(), "partial response");
    }

    // ---- HistorySearch --------------------------------------------------

    #[test]
    fn test_history_search_matches() {
        let history = vec!["git commit".to_string(), "git push".to_string(), "cargo build".to_string()];
        let mut hs = HistorySearch::new();
        hs.query = "git".to_string();
        hs.update_matches(&history);
        assert_eq!(hs.matches.len(), 2);
        assert_eq!(hs.matches[0], 0);
        assert_eq!(hs.matches[1], 1);
    }

    #[test]
    fn test_history_search_no_matches() {
        let history = vec!["hello".to_string()];
        let mut hs = HistorySearch::new();
        hs.query = "xyz".to_string();
        hs.update_matches(&history);
        assert!(hs.matches.is_empty());
    }

    // ---- PermissionRequest --------------------------------------------

    #[test]
    fn test_permission_request_standard() {
        let pr = PermissionRequest::standard(
            "tu1".to_string(),
            "Bash".to_string(),
            "Run a shell command".to_string(),
        );
        assert_eq!(pr.options.len(), 3);
        assert_eq!(pr.options[0].key, 'y');
        assert_eq!(pr.options[1].key, 'a');
        assert_eq!(pr.options[2].key, 'n');
    }
}
