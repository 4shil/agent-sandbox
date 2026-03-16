mod home;
mod sessions;
mod detail;

pub use home::HomeScreen;
pub use sessions::{SessionsScreen, SessionsState};
pub use detail::DetailScreen;

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
            _ => {}
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, toasts: &mut ToastManager) {
        match self {
            Screen::Home => HomeScreen::handle_key(key, toasts),
                        Screen::Sessions(state) => SessionsScreen::handle_key(key, state, toasts),
            Screen::Detail(id) => DetailScreen::handle_key(key, toasts, id),
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        match self {
            Screen::Home => HomeScreen::tick(),
            Screen::Sessions => SessionsScreen::tick(),
            _ => {}
        }
    }
}
