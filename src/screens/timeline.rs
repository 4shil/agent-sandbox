use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block as TuiBlock, Borders, BarChart, Paragraph},
    Frame,
};
use crossterm::event::{KeyEvent, KeyCode};

use crate::tui::Theme;
use crate::tui::widgets::{ToastManager, Toast};
use agent_sandbox_core::recorder::SessionRecord;

pub struct TimelineScreen;

#[derive(Clone)]
struct TimelineItem {
    name: String,
    duration_ms: u64,
}

impl TimelineScreen {
    pub fn render(frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(2)])
            .split(area);

        let header = Paragraph::new(Line::from(" TIMELINE ").style(theme.title_style()));
        frame.render_widget(header, chunks[0]);

        let data = load_timeline();
        let max = data.iter().map(|d| d.duration_ms).max().unwrap_or(1);

        let bars: Vec<(&str, u64)> = data
            .iter()
            .take(12)
            .map(|d| (d.name.as_str(), (d.duration_ms * 100 / max).max(1)))
            .collect();

        let chart = BarChart::default()
            .block(
                TuiBlock::default()
                    .borders(Borders::ALL)
                    .title(" recent sessions ")
                    .title_style(theme.title_style())
                    .border_style(theme.focused_border_style()),
            )
            .data(&bars)
            .bar_width(6)
            .bar_gap(2)
            .value_style(Style::default().fg(theme.primary))
            .label_style(theme.muted_style())
            .bar_style(Style::default().fg(theme.success).add_modifier(Modifier::BOLD));

        frame.render_widget(chart, chunks[1]);

        let hints = Paragraph::new(Line::from(" Esc Back │ ↑↓ Scroll │ / Search (soon)").style(theme.muted_style()));
        frame.render_widget(hints, chunks[2]);
    }

    pub fn handle_key(key: KeyEvent, toasts: &mut ToastManager) {
        match key.code {
            KeyCode::Char('/') => toasts.push(Toast::info("Search coming soon".to_string())),
            _ => {}
        }
    }
}

fn load_timeline() -> Vec<TimelineItem> {
    let dir = crate::session::get_workspaces_dir();
    let mut sessions = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
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

            sessions.push(TimelineItem { name, duration_ms });
        }
    }

    sessions.sort_by(|a, b| b.duration_ms.cmp(&a.duration_ms));
    sessions
}
