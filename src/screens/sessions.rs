use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block as TuiBlock, Borders, List, ListItem, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};
use crossterm::event::{KeyEvent, KeyCode};

use crate::tui::Theme;
use crate::tui::widgets::ToastManager;
use agent_sandbox_core::recorder::SessionRecord;

#[derive(Default)]
pub struct SessionsState {
    pub selected: usize,
    pub query: String,
    pub searching: bool,
    pub list_state: ratatui::widgets::ListState,
}

pub struct SessionsScreen;

pub struct SessionItem {
    pub name: String,
    pub agent: String,
    pub started_at: u64,
    pub duration_ms: u64,
    pub actions: usize,
}

impl SessionsScreen {
    pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, state: &mut SessionsState) {
        let filtered = filtered_sessions(state);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(3)])
            .split(area);

        let header = Paragraph::new(Line::from(format!(" SESSIONS ({})", filtered.len())))
            .style(theme.title_style());
        frame.render_widget(header, chunks[0]);

        let search_label = if state.searching { "SEARCH:" } else { "Search:" };
        let search = Paragraph::new(Line::from(format!(" {} {}", search_label, state.query))).style(
            if state.searching {
                theme.accent_style()
            } else {
                theme.muted_style()
            },
        );
        frame.render_widget(
            search,
            Rect {
                x: chunks[0].x,
                y: chunks[0].y + 1,
                width: chunks[0].width,
                height: 1,
            },
        );

        let columns = Paragraph::new(Line::from(
            "  Name                        Agent      Date         Duration   Actions Tags"
        ))
        .style(theme.muted_style());
        frame.render_widget(
            columns,
            Rect {
                x: chunks[1].x + 1,
                y: chunks[1].y,
                width: chunks[1].width.saturating_sub(2),
                height: 1,
            },
        );

        let items: Vec<ListItem> = filtered
            .iter()
            .map(|s| {
                let date = format_date(s.started_at);
                let duration = format_duration(s.duration_ms);

                let mut spans = vec![Span::raw("  ")];
                spans.extend(highlight_owned_spans(format!("{:<28}", s.name), &state.query, theme));
                spans.push(Span::raw(" "));
                spans.extend(highlight_owned_spans(format!("{:<10}", s.agent), &state.query, theme));
                spans.push(Span::raw(format!(" {:<12} {:<10} {:<7} {:<6}", date, duration, s.actions, "-")));

                ListItem::new(Line::from(spans))
            })
            .collect();

        if !items.is_empty() {
            let sel = state.selected.min(items.len() - 1);
            state.list_state.select(Some(sel));
        }

        let list = List::new(items)
            .highlight_style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
            .highlight_symbol("▸ ");

        frame.render_widget(
            TuiBlock::default()
                .borders(Borders::ALL)
                .title(" sessions ")
                .title_style(theme.title_style())
                .border_style(theme.focused_border_style()),
            chunks[1],
        );

        let list_area = Rect {
            x: chunks[1].x + 1,
            y: chunks[1].y + 2,
            width: chunks[1].width.saturating_sub(2),
            height: chunks[1].height.saturating_sub(3),
        };
        frame.render_stateful_widget(list, list_area, &mut state.list_state);

        if filtered.len() > list_area.height as usize {
            let mut scrollbar_state = ScrollbarState::new(filtered.len()).position(state.selected);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            frame.render_stateful_widget(scrollbar, chunks[1], &mut scrollbar_state);
        }

        let hints = Paragraph::new(Line::from(" / Search │ ↑↓ Navigate │ Enter Details │ C Continue │ Esc Back"))
            .style(theme.muted_style());
        frame.render_widget(hints, chunks[2]);
    }

    pub fn handle_key(key: KeyEvent, state: &mut SessionsState, _toasts: &mut ToastManager) {
        let filtered_count = filtered_sessions(state).len();
        match key.code {
            KeyCode::Char('/') => state.searching = true,
            KeyCode::Esc => {
                state.searching = false;
                state.query.clear();
            }
            KeyCode::Up => {
                if state.selected > 0 {
                    state.selected -= 1;
                }
            }
            KeyCode::Down => {
                if filtered_count > 0 {
                    state.selected = (state.selected + 1).min(filtered_count - 1);
                }
            }
            KeyCode::Backspace => {
                if state.searching {
                    state.query.pop();
                }
            }
            KeyCode::Char(c) => {
                if state.searching {
                    state.query.push(c);
                }
            }
            _ => {}
        }
        // Sync list state
        if filtered_count > 0 {
            state.list_state.select(Some(state.selected.min(filtered_count - 1)));
        }
    }

    pub fn tick() {}
}

fn load_sessions() -> Vec<SessionItem> {
    let dir = crate::session::get_workspaces_dir();
    let mut sessions = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            let agent = name.split('-').next().unwrap_or("unknown").to_string();
            let logs = entry.path().join("logs");

            let (duration_ms, actions, started_at) = if let Some(record) = load_latest_record(&logs) {
                (record.duration_ms.unwrap_or(0), record.actions.len(), record.started_at)
            } else {
                (0u64, std::fs::read_dir(&logs).map(|d| d.count()).unwrap_or(0), 0u64)
            };

            sessions.push(SessionItem { name, agent, started_at, duration_ms, actions });
        }
    }

    sessions.sort_by(|a, b| b.name.cmp(&a.name));
    sessions
}

fn load_latest_record(logs: &std::path::Path) -> Option<SessionRecord> {
    if let Ok(log_entries) = std::fs::read_dir(logs) {
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
                    return Some(record);
                }
            }
        }
    }
    None
}

fn format_date(ts: u64) -> String {
    chrono::DateTime::from_timestamp(ts as i64, 0)
        .map(|t| t.format("%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "-".into())
}

fn format_duration(ms: u64) -> String {
    let secs = ms / 1000;
    let mins = secs / 60;
    let hrs = mins / 60;
    if hrs > 0 {
        format!("{}h{}m", hrs, mins % 60)
    } else if mins > 0 {
        format!("{}m{}s", mins, secs % 60)
    } else {
        format!("{}s", secs)
    }
}

fn highlight_owned_spans(text: String, query: &str, theme: &Theme) -> Vec<Span<'static>> {
    if query.is_empty() {
        return vec![Span::raw(text)];
    }

    let lower_text = text.to_lowercase();
    let lower_query = query.to_lowercase();
    if let Some(start) = lower_text.find(&lower_query) {
        let end = (start + lower_query.len()).min(text.len());
        vec![
            Span::raw(text[..start].to_string()),
            Span::styled(text[start..end].to_string(), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::raw(text[end..].to_string()),
        ]
    } else {
        vec![Span::raw(text)]
    }
}

pub fn filtered_sessions(state: &SessionsState) -> Vec<SessionItem> {
    let sessions = load_sessions();
    if state.query.is_empty() {
        sessions
    } else {
        let q = state.query.to_lowercase();
        sessions
            .into_iter()
            .filter(|s| s.name.to_lowercase().contains(&q) || s.agent.to_lowercase().contains(&q))
            .collect()
    }
}
