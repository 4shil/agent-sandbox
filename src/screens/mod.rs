mod home;
mod sessions;
mod detail;
mod timeline;
mod stats;

pub use home::HomeScreen;
pub use sessions::{SessionsScreen, SessionsState};
pub use detail::DetailScreen;
pub use timeline::TimelineScreen;
pub use stats::StatsScreen;

use anyhow::Result;
use ratatui::{layout::Rect, Frame};
use crossterm::event::KeyEvent;

use crate::tui::Theme;
use crate::tui::widgets::ToastManager;

pub enum Screen {
    Home,
    Sessions(SessionsState),
    Detail(String), // session id
    Timeline,
    Stats,
}

impl Screen {
    pub fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        match self {
            Screen::Home => HomeScreen::render(frame, area, theme),
            Screen::Sessions(state) => SessionsScreen::render(frame, area, theme, state),
            Screen::Detail(id) => DetailScreen::render(frame, area, theme, id),
            Screen::Timeline => TimelineScreen::render(frame, area, theme),
            Screen::Stats => StatsScreen::render(frame, area, theme),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, toasts: &mut ToastManager) {
        match self {
            Screen::Home => HomeScreen::handle_key(key, toasts),
            Screen::Sessions(state) => SessionsScreen::handle_key(key, state, toasts),
            Screen::Detail(id) => DetailScreen::handle_key(key, toasts, id),
            Screen::Timeline => TimelineScreen::handle_key(key, toasts),
            Screen::Stats => StatsScreen::handle_key(key, toasts),
        }
    }

    pub fn tick(&mut self) {
        match self {
            Screen::Home => HomeScreen::tick(),
            Screen::Sessions(_) => SessionsScreen::tick(),
            _ => {}
        }
    }
}
