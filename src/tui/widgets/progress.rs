use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block as TuiBlock, Borders, Gauge},
    Frame,
};
use crate::tui::Theme;

pub struct ProgressBar {
    pub percent: f64,
    pub label: String,
    pub indeterminate: bool,
}

impl ProgressBar {
    pub fn new(percent: f64, label: String) -> Self {
        Self { percent, label, indeterminate: false }
    }

    pub fn indeterminate(label: String) -> Self {
        Self { percent: 0.0, label, indeterminate: true }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let gauge = Gauge::default()
            .block(TuiBlock::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border)))
            .gauge_style(
                Style::default()
                    .fg(theme.primary)
                    .bg(theme.border)
                    .add_modifier(Modifier::BOLD),
            )
            .percent((self.percent * 100.0) as u16)
            .label(format!("{} {:.0}%", self.label, self.percent * 100.0));
        frame.render_widget(gauge, area);
    }
}
