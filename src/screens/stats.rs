use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block as TuiBlock, Borders, BarChart, Paragraph},
    Frame,
};
use crossterm::event::{KeyEvent, KeyCode};

use crate::tui::Theme;
use crate::tui::widgets::ToastManager;

pub struct StatsScreen;

impl StatsScreen {
    pub fn render(frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(4), Constraint::Min(6), Constraint::Length(2)])
            .split(area);

        let header = Paragraph::new(Line::from(" STATS ").style(theme.title_style()));
        frame.render_widget(header, chunks[0]);

        let summary = summary_line();
        let summary_box = Paragraph::new(summary)
            .style(theme.default_style())
            .block(
                TuiBlock::default()
                    .borders(Borders::ALL)
                    .title(" summary ")
                    .title_style(theme.title_style())
                    .border_style(theme.focused_border_style()),
            );
        frame.render_widget(summary_box, chunks[1]);

        let agent_counts = agent_counts();
        let data: Vec<(&str, u64)> = agent_counts
            .iter()
            .map(|(k, v)| (k.as_str(), *v as u64))
            .collect();

        let chart = BarChart::default()
            .block(
                TuiBlock::default()
                    .borders(Borders::ALL)
                    .title(" sessions per agent ")
                    .title_style(theme.title_style())
                    .border_style(theme.unfocused_border_style()),
            )
            .data(&data)
            .bar_width(6)
            .bar_gap(2)
            .value_style(Style::default().fg(theme.primary))
            .label_style(theme.muted_style())
            .bar_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));

        frame.render_widget(chart, chunks[2]);

        let hints = Paragraph::new(Line::from(" Esc Back │ T Timeline │ / Search (soon)").style(theme.muted_style()));
        frame.render_widget(hints, chunks[3]);
    }

    pub fn handle_key(_key: KeyEvent, _toasts: &mut ToastManager) {}
}

fn summary_line() -> String {
    let total = count_sessions();
    let agents = agent_counts();
    let top_agent = agents.iter().max_by_key(|(_, v)| *v).map(|(k, _)| k.as_str()).unwrap_or("-");
    format!("  Total sessions: {} │ Top agent: {}", total, top_agent)
}

fn agent_counts() -> Vec<(String, usize)> {
    let dir = crate::session::get_workspaces_dir();
    let mut counts = std::collections::BTreeMap::<String, usize>::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            let agent = name.split('-').next().unwrap_or("unknown").to_string();
            *counts.entry(agent).or_insert(0) += 1;
        }
    }

    counts.into_iter().collect()
}

fn count_sessions() -> usize {
    let dir = crate::session::get_workspaces_dir();
    std::fs::read_dir(&dir).map(|d| d.count()).unwrap_or(0)
}
