use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::Paragraph,
    Frame,
};
use crate::tui::Theme;

pub struct StatusBar<'a> {
    pub left: &'a str,
    pub right: &'a str,
    pub mode: Option<&'a str>,
}

impl<'a> StatusBar<'a> {
    pub fn new(left: &'a str, right: &'a str) -> Self {
        Self { left, right, mode: None }
    }

    pub fn mode(mut self, mode: &'a str) -> Self {
        self.mode = Some(mode);
        self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let left = if let Some(mode) = self.mode {
            format!(" {} │ {}", mode.to_uppercase(), self.left)
        } else {
            format!(" {}", self.left)
        };
        let right = format!("{} ", self.right);

        let spacing = " ".repeat(area.width as usize);
        let line = Line::from(vec![
            ratatui::text::Span::styled(left, Style::default().fg(theme.primary)),
            ratatui::text::Span::raw(spacing),
            ratatui::text::Span::styled(right, theme.muted_style()),
        ]);

        let block = Paragraph::new(line)
            .style(Style::default().bg(theme.border).fg(theme.fg));
        frame.render_widget(block, area);
    }
}
