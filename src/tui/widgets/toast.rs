use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::tui::Theme;
use std::time::Instant;

#[derive(Clone)]
pub enum ToastType {
    Success,
    Error,
    Info,
    Warning,
}

pub struct Toast {
    pub message: String,
    pub toast_type: ToastType,
    pub created: Instant,
    pub duration_ms: u64,
}

impl Toast {
    pub fn success(msg: String) -> Self {
        Self { message: msg, toast_type: ToastType::Success, created: Instant::now(), duration_ms: 3000 }
    }
    pub fn error(msg: String) -> Self {
        Self { message: msg, toast_type: ToastType::Error, created: Instant::now(), duration_ms: 5000 }
    }
    pub fn info(msg: String) -> Self {
        Self { message: msg, toast_type: ToastType::Info, created: Instant::now(), duration_ms: 3000 }
    }
    pub fn warning(msg: String) -> Self {
        Self { message: msg, toast_type: ToastType::Warning, created: Instant::now(), duration_ms: 4000 }
    }

    pub fn is_expired(&self) -> bool {
        self.created.elapsed().as_millis() > self.duration_ms as u128
    }
}

pub struct ToastManager {
    toasts: Vec<Toast>,
}

impl ToastManager {
    pub fn new() -> Self {
        Self { toasts: Vec::new() }
    }

    pub fn push(&mut self, toast: Toast) {
        self.toasts.push(toast);
    }

    pub fn cleanup(&mut self) {
        self.toasts.retain(|t| !t.is_expired());
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        self.cleanup();
        let max_visible = 5;
        let visible: Vec<&Toast> = self.toasts.iter().rev().take(max_visible).collect();

        for (i, toast) in visible.iter().enumerate() {
            let y = area.y + i as u16 + 1;
            let width = (toast.message.len() + 4) as u16;
            let x = area.right() - width - 1;
            let toast_area = Rect::new(x, y, width, 3);

            let (icon, style) = match toast.toast_type {
                ToastType::Success => ("✓", Style::default().fg(theme.success).bg(theme.bg)),
                ToastType::Error => ("✗", Style::default().fg(theme.error).bg(theme.bg)),
                ToastType::Info => ("ℹ", Style::default().fg(theme.primary).bg(theme.bg)),
                ToastType::Warning => ("⚠", Style::default().fg(theme.warning).bg(theme.bg)),
            };

            frame.render_widget(Clear, toast_area);
            let msg = format!("{} {}", icon, toast.message);
            let line = Paragraph::new(msg).style(style);
            frame.render_widget(line, toast_area);
        }
    }
}
