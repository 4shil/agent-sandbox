use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block as TuiBlock, Borders, Clear, Paragraph},
    Frame,
};
use crate::tui::Theme;

pub enum ModalType {
    Confirm,
    Input,
    Info,
}

pub struct Modal<'a> {
    pub title: &'a str,
    pub message: &'a str,
    pub modal_type: ModalType,
    pub input_value: Option<String>,
    pub confirm_result: Option<bool>,
}

impl<'a> Modal<'a> {
    pub fn confirm(title: &'a str, message: &'a str) -> Self {
        Self { title, message, modal_type: ModalType::Confirm, input_value: None, confirm_result: None }
    }

    pub fn input(title: &'a str, message: &'a str) -> Self {
        Self { title, message, modal_type: ModalType::Input, input_value: Some(String::new()), confirm_result: None }
    }

    pub fn info(title: &'a str, message: &'a str) -> Self {
        Self { title, message, modal_type: ModalType::Info, input_value: None, confirm_result: None }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let popup_area = centered_rect(60, 20, area);
        frame.render_widget(Clear, popup_area);

        let block = TuiBlock::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.title))
            .title_style(theme.title_style())
            .border_style(Style::default().fg(theme.accent))
            .style(Style::default().bg(theme.bg));

        frame.render_widget(block, popup_area);

        let inner = centered_rect(80, 60, popup_area);
        let message = Paragraph::new(self.message)
            .style(Style::default().fg(theme.fg));
        frame.render_widget(message, inner);

        let hint = match self.modal_type {
            ModalType::Confirm => "[Y]es  [N]o",
            ModalType::Input => "Type and press Enter",
            ModalType::Info => "Press any key",
        };

        let hint_area = Rect {
            x: popup_area.x + 2,
            y: popup_area.bottom() - 3,
            width: popup_area.width.saturating_sub(4),
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(hint).style(theme.muted_style()),
            hint_area,
        );
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
