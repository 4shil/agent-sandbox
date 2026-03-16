use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block as TuiBlock, Borders, List, ListItem, Paragraph},
    Frame,
};
use crossterm::event::{KeyEvent, KeyCode};

use crate::tui::Theme;
use crate::tui::widgets::{StyledBlock, StatusBar, ToastManager, Toast};
use crate::screens::Screen;

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

impl HomeScreen {
    pub fn render(frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),
                Constraint::Length(1),
            ])
            .split(area);

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),
                Constraint::Length(1),
                Constraint::Min(8),
                Constraint::Length(3),
            ])
            .split(chunks[0]);

        let logo = Paragraph::new(LOGO)
            .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD));
        frame.render_widget(logo, main_chunks[0]);

        let items: Vec<ListItem> = AGENTS.iter().map(|(name, desc, installed)| {
            let mark = if *installed { "✓" } else { " " };
            let style = if *installed {
                Style::default().fg(theme.success)
            } else {
                Style::default().fg(theme.muted)
            };
            ListItem::new(Line::from(format!("  [{}] {:<14} {}", mark, name, desc)))
                .style(style)
        }).collect();

        let list = List::new(items);

        frame.render_widget(TuiBlock::default()
            .borders(Borders::ALL)
            .title(" detected agents ")
            .title_style(theme.title_style())
            .border_style(theme.focused_border_style()), main_chunks[2]);

        let list_area = Rect {
            x: main_chunks[2].x + 1,
            y: main_chunks[2].y + 1,
            width: main_chunks[2].width.saturating_sub(2),
            height: main_chunks[2].height.saturating_sub(2),
        };
        frame.render_widget(list, list_area);

        let hints = Paragraph::new(
            Line::from(" ↑↓ Navigate │ Enter Launch │ S Sessions │ D Stats │ Q Quit")
                .style(theme.muted_style())
        );
        frame.render_widget(hints, main_chunks[3]);

        let session_count = count_sessions();
        let agent_count = AGENTS.iter().filter(|(_, _, i)| *i).count();
        let status_text = format!("Sessions: {} │ Agents: {}", session_count, agent_count);
        let status = StatusBar::new(
            &status_text,
            "v0.9.0 │ Space: 2.1GB",
        );
        status.render(frame, chunks[1], theme);
    }

    pub fn handle_key(key: KeyEvent, toasts: &mut ToastManager) {
        match key.code {
            KeyCode::Char('s') => {
                // Screen transition handled in app
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                toasts.push(Toast::info("Select an agent to launch".to_string()));
            }
            _ => {}
        }
    }

    pub fn tick() {}
}

fn count_sessions() -> usize {
    let home = std::env::var("HOME").unwrap_or_default();
    let dir = format!("{}/.agent-sandbox/workspaces", home);
    std::fs::read_dir(&dir).map(|d| d.count()).unwrap_or(0)
}
