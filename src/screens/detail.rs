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

        let meta_text = format!(
            "  ID:       {}\n  Agent:    {}\n  Duration: {}\n  Actions:  {}",
            &id[..id.len().min(30)],
            "opencode",
            "2m 30s",
            "42"
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

        let actions: Vec<ListItem> = (0..5).map(|i| {
            ListItem::new(Line::from(format!("  {:<10} file_modified: main.rs", format!("#{}", i + 1))))
        }).collect();

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
