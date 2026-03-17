use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block as TuiBlock, Borders, List, ListItem, Paragraph},
    Frame,
};
use crossterm::event::{KeyEvent, KeyCode};

use crate::tui::Theme;
use crate::tui::widgets::{StatusBar, ToastManager};
use crate::recorder::SessionRecord;

const LOGO: &str = r#"
    _____
   /     \
  | () () |    abox
   \  ^  /    ─────────────────────
    |||||     sandbox for ai agents
    |||||"#;

const AGENTS: &[(&str, &str, bool)] = &[
    ("claude", "Claude Code", false),
    ("codex", "OpenAI Codex", false),
    ("opencode", "OpenCode", true),
    ("gemini", "Gemini CLI", true),
    ("aider", "Aider", false),
    ("goose", "Goose", false),
];

pub struct HomeScreen;

#[derive(Default, Clone)]
pub struct HomeState {
    pub selected: usize,
}

#[derive(Clone)]
struct RecentSession {
    name: String,
    agent: String,
    duration_ms: u64,
}

impl HomeScreen {
    pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, state: &HomeState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),
                Constraint::Length(1),
            ])
            .split(area);

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),
                Constraint::Min(8),
                Constraint::Length(3),
            ])
            .split(chunks[0]);

        let logo = Paragraph::new(LOGO)
            .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD));
        frame.render_widget(logo, main_chunks[0]);

        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(main_chunks[1]);

        render_agents(frame, body_chunks[0], theme, state);
        render_recent_sessions(frame, body_chunks[1], theme);

        let hints = Paragraph::new(
            Line::from(" ↑↓ Navigate │ Enter Launch │ S Sessions │ D Stats │ Q Quit")
                .style(theme.muted_style())
        );
        frame.render_widget(hints, main_chunks[2]);

        let session_count = count_sessions();
        let agent_count = AGENTS.iter().filter(|(_, _, i)| *i).count();
        let status_text = format!("Sessions: {} │ Agents: {}", session_count, agent_count);
        let status = StatusBar::new(
            &status_text,
            "v0.9.2 │ Space: 2.1GB",
        );
        status.render(frame, chunks[1], theme);
    }

    pub fn handle_key(key: KeyEvent, _toasts: &mut ToastManager, state: &mut HomeState) {
        let installed_count = AGENTS.iter().filter(|(_, _, i)| *i).count();
        if installed_count == 0 {
            return;
        }
        match key.code {
            KeyCode::Up => {
                if state.selected > 0 {
                    state.selected -= 1;
                }
            }
            KeyCode::Down => {
                state.selected = (state.selected + 1).min(installed_count.saturating_sub(1));
            }
            _ => {}
        }
    }

    pub fn tick() {}
}

fn render_agents(frame: &mut Frame, area: Rect, theme: &Theme, state: &HomeState) {
    let installed: Vec<_> = AGENTS.iter().filter(|(_, _, installed)| *installed).collect();
    let items: Vec<ListItem> = installed.iter().map(|(name, desc, _)| {
        ListItem::new(Line::from(format!("  [✓] {:<14} {}", name, desc)))
            .style(Style::default().fg(theme.success))
    }).collect();

    frame.render_widget(
        TuiBlock::default()
            .borders(Borders::ALL)
            .title(" detected agents ")
            .title_style(theme.title_style())
            .border_style(theme.focused_border_style()),
        area,
    );

    let list_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    use ratatui::widgets::ListState;
    let list = List::new(items)
        .highlight_symbol("▸ ")
        .highlight_style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD));
    let mut stateful = ListState::default();
    if !installed.is_empty() {
        stateful.select(Some(state.selected.min(installed.len() - 1)));
    }
    frame.render_stateful_widget(list, list_area, &mut stateful);
}

pub fn selected_agent(state: &HomeState) -> Option<&'static str> {
    let installed: Vec<_> = AGENTS.iter().filter(|(_, _, installed)| *installed).collect();
    installed.get(state.selected).map(|(name, _, _)| *name)
}

fn render_recent_sessions(frame: &mut Frame, area: Rect, theme: &Theme) {
    let sessions = load_recent_sessions();
    let max_duration = sessions.iter().map(|s| s.duration_ms).max().unwrap_or(1);

    let items: Vec<ListItem> = sessions.iter().map(|s| {
        let seconds = s.duration_ms as f64 / 1000.0;
        let bar_len = ((s.duration_ms as f64 / max_duration as f64) * 10.0).round() as usize;
        let bar = "█".repeat(bar_len.max(1));
        let line = Line::from(vec![
            Span::styled(format!("{: <18}", s.name), theme.primary_style()),
            Span::raw(" "),
            Span::styled(format!("{: <8}", s.agent), theme.muted_style()),
            Span::raw(" "),
            Span::styled(format!("{: >5.0}s ", seconds), theme.accent_style()),
            Span::styled(bar, theme.success_style()),
        ]);
        ListItem::new(line)
    }).collect();

    frame.render_widget(
        TuiBlock::default()
            .borders(Borders::ALL)
            .title(" recent sessions ")
            .title_style(theme.title_style())
            .border_style(theme.unfocused_border_style()),
        area,
    );

    let list_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    frame.render_widget(List::new(items), list_area);
}

fn load_recent_sessions() -> Vec<RecentSession> {
    let dir = crate::session::get_workspaces_dir();
    let mut sessions = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            let agent = name.split('-').next().unwrap_or("unknown").to_string();
            let logs = entry.path().join("logs");

            let mut duration_ms = 0u64;
            if let Ok(log_entries) = std::fs::read_dir(&logs) {
                let mut latest_path = None;
                let mut latest_time = None;
                for log in log_entries.filter_map(|e| e.ok()) {
                    let metadata = log.metadata().and_then(|m| m.modified()).ok();
                    let is_newer = match (latest_time, metadata) {
                        (None, Some(_)) => true,
                        (Some(prev), Some(next)) => next > prev,
                        _ => false,
                    };
                    if latest_path.is_none() || is_newer {
                        latest_path = Some(log.path());
                        latest_time = metadata;
                    }
                }
                if let Some(path) = latest_path {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        if let Ok(record) = serde_json::from_str::<SessionRecord>(&content) {
                            duration_ms = record.duration_ms.unwrap_or(0);
                        }
                    }
                }
            }

            sessions.push(RecentSession { name, agent, duration_ms });
        }
    }

    sessions.sort_by(|a, b| b.duration_ms.cmp(&a.duration_ms));
    sessions.truncate(6);
    sessions
}

fn count_sessions() -> usize {
    let home = std::env::var("HOME").unwrap_or_default();
    let dir = format!("{}/.agent-sandbox/workspaces", home);
    std::fs::read_dir(&dir).map(|d| d.count()).unwrap_or(0)
}
