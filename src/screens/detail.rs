use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block as TuiBlock, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use crossterm::event::{KeyEvent, KeyCode};

use crate::tui::Theme;
use crate::tui::widgets::{ToastManager, Toast};
use crate::recorder::SessionRecord;

pub struct DetailScreen;

impl DetailScreen {
    pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, id: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(8),
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(area);

        let header_text = format!(" SESSION DETAIL {} ", &id[..id.len().min(30)]);
        let header = Paragraph::new(
            Line::from(ratatui::text::Span::styled(header_text, theme.title_style()))
        );
        frame.render_widget(header, chunks[0]);

        let record = load_session_record(id);
        let (agent, duration, actions) = if let Some(rec) = &record {
            (
                rec.agent.clone(),
                format_duration(rec.duration_ms.unwrap_or(0)),
                rec.actions.len(),
            )
        } else {
            ("unknown".to_string(), "-".to_string(), 0)
        };

        let meta_text = format!(
            "  ID:       {}\n  Agent:    {}\n  Duration: {}\n  Actions:  {}",
            &id[..id.len().min(30)],
            agent,
            duration,
            actions
        );
        let meta = Paragraph::new(meta_text)
            .style(theme.default_style())
            .wrap(Wrap { trim: false });
        frame.render_widget(
            TuiBlock::default()
                .borders(Borders::ALL)
                .title(" metadata ")
                .title_style(theme.title_style())
                .border_style(theme.focused_border_style()),
            chunks[1],
        );
        let meta_area = Rect { x: chunks[1].x + 1, y: chunks[1].y + 1, width: chunks[1].width.saturating_sub(2), height: chunks[1].height.saturating_sub(2) };
        frame.render_widget(meta, meta_area);

        let actions: Vec<ListItem> = if let Some(rec) = record {
            rec.actions
                .iter()
                .take(12)
                .enumerate()
                .map(|(i, a)| {
                    let detail = a.data.get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let line = if detail.is_empty() {
                        format!("  {:<10} {}", format!("#{}", i + 1), a.action_type)
                    } else {
                        format!("  {:<10} {}: {}", format!("#{}", i + 1), a.action_type, detail)
                    };
                    ListItem::new(Line::from(line))
                })
                .collect()
        } else {
            vec![ListItem::new(Line::from("  No actions found"))]
        };

        let list = List::new(actions)
            .highlight_style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
            .highlight_symbol("▸ ");

        frame.render_widget(
            TuiBlock::default()
                .borders(Borders::ALL)
                .title(" actions ")
                .title_style(theme.title_style())
                .border_style(theme.focused_border_style()),
            chunks[2],
        );
        let action_area = Rect { x: chunks[2].x + 1, y: chunks[2].y + 1, width: chunks[2].width.saturating_sub(2), height: chunks[2].height.saturating_sub(2) };
        frame.render_widget(list, action_area);

        let hints = Paragraph::new(
            Line::from(" E Edit │ T Tag │ N Note │ D Diff │ Esc Back")
                .style(theme.muted_style())
        );
        frame.render_widget(hints, chunks[3]);
    }

    pub fn handle_key(key: KeyEvent, toasts: &mut ToastManager, _id: &str) {
        match key.code {
            KeyCode::Char('t') => {
                toasts.push(Toast::info("Tag feature coming soon".to_string()));
            }
            KeyCode::Char('n') => {
                toasts.push(Toast::info("Note feature coming soon".to_string()));
            }
            _ => {}
        }
    }
}

fn load_session_record(sandbox_name: &str) -> Option<SessionRecord> {
    let dir = crate::session::get_workspaces_dir();
    let path = dir.join(sandbox_name).join("logs");
    let mut latest_path = None;
    let mut latest_time = None;

    if let Ok(entries) = std::fs::read_dir(&path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let metadata = entry.metadata().and_then(|m| m.modified()).ok();
            let is_newer = match (latest_time, metadata) {
                (None, Some(_)) => true,
                (Some(prev), Some(next)) => next > prev,
                _ => false,
            };
            if latest_path.is_none() || is_newer {
                latest_path = Some(entry.path());
                latest_time = metadata;
            }
        }
    }

    if let Some(p) = latest_path {
        if let Ok(content) = std::fs::read_to_string(p) {
            if let Ok(record) = serde_json::from_str::<SessionRecord>(&content) {
                return Some(record);
            }
        }
    }
    None
}

fn format_duration(ms: u64) -> String {
    let secs = ms / 1000;
    let mins = secs / 60;
    let hrs = mins / 60;
    if hrs > 0 {
        format!("{}h {}m", hrs, mins % 60)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs % 60)
    } else {
        format!("{}s", secs)
    }
}
