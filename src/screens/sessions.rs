use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{
        Block as TuiBlock, Borders, List, ListItem, Paragraph, ListState, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};
use crossterm::event::{KeyEvent, KeyCode};

use crate::tui::Theme;
use crate::tui::widgets::ToastManager;

#[derive(Default)]
pub struct SessionsState {
    pub selected: usize,
    pub query: String,
    pub searching: bool,
}

pub struct SessionsScreen;

pub struct SessionItem {
    pub name: String,
    pub agent: String,
    pub actions: usize,
}

impl SessionsScreen {
    pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, state: &SessionsState) {
        let sessions = load_sessions();
        let filtered: Vec<_> = if state.query.is_empty() {
            sessions
        } else {
            let q = state.query.to_lowercase();
            sessions
                .into_iter()
                .filter(|s| s.name.to_lowercase().contains(&q) || s.agent.to_lowercase().contains(&q))
                .collect()
        };

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

        let items: Vec<ListItem> = filtered
            .iter()
            .map(|s| {
                ListItem::new(Line::from(format!(
                    "  {:<30} {:<12} {} actions",
                    s.name, s.agent, s.actions
                )))
            })
            .collect();

        let mut list_state = ListState::default();
        if !items.is_empty() {
            let sel = state.selected.min(items.len() - 1);
            list_state.select(Some(sel));
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
            y: chunks[1].y + 1,
            width: chunks[1].width.saturating_sub(2),
            height: chunks[1].height.saturating_sub(2),
        };
        frame.render_stateful_widget(list, list_area, &mut list_state);

        if filtered.len() > list_area.height as usize {
            let mut scrollbar_state = ScrollbarState::new(filtered.len()).position(state.selected);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            frame.render_stateful_widget(scrollbar, chunks[1], &mut scrollbar_state);
        }

        let hints = Paragraph::new(Line::from(" / Search │ ↑↓ Navigate │ Enter Details │ Esc Back"))
            .style(theme.muted_style());
        frame.render_widget(hints, chunks[2]);
    }

    pub fn handle_key(key: KeyEvent, state: &mut SessionsState, _toasts: &mut ToastManager) {
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
                state.selected = state.selected.saturating_add(1);
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
    }

    pub fn tick() {}
}

fn load_sessions() -> Vec<SessionItem> {
    let home = std::env::var("HOME").unwrap_or_default();
    let dir = format!("{}/.agent-sandbox/workspaces", home);
    let mut sessions = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            let agent = name.split('-').next().unwrap_or("unknown").to_string();
            let logs = entry.path().join("logs");
            let actions = std::fs::read_dir(&logs).map(|d| d.count()).unwrap_or(0);
            sessions.push(SessionItem { name, agent, actions });
        }
    }

    sessions.sort_by(|a, b| b.name.cmp(&a.name));
    sessions
}
