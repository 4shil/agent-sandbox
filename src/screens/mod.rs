mod home;
mod sessions;
mod detail;
mod timeline;
mod stats;

pub use home::{HomeScreen, HomeState, selected_agent};
pub use sessions::{SessionsScreen, SessionsState, filtered_sessions};
pub use detail::DetailScreen;
pub use timeline::TimelineScreen;
pub use stats::StatsScreen;

use ratatui::{layout::Rect, Frame};
use crossterm::event::KeyEvent;
use std::collections::BTreeSet;

use crate::tui::Theme;
use crate::tui::widgets::ToastManager;

#[derive(Default, Clone)]
pub struct DetailState {
    pub selected: usize,
    pub expanded: BTreeSet<usize>,
}

pub enum Screen {
    Home(HomeState),
    Sessions(SessionsState),
    Detail(String, DetailState), // session id + ui state
    Timeline,
    Stats,
}

impl Screen {
    pub fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        match self {
            Screen::Home(state) => HomeScreen::render(frame, area, theme, state),
            Screen::Sessions(state) => SessionsScreen::render(frame, area, theme, state),
            Screen::Detail(id, state) => DetailScreen::render(frame, area, theme, id, state),
            Screen::Timeline => TimelineScreen::render(frame, area, theme),
            Screen::Stats => StatsScreen::render(frame, area, theme),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, toasts: &mut ToastManager) {
        match self {
            Screen::Home(state) => HomeScreen::handle_key(key, toasts, state),
            Screen::Sessions(state) => SessionsScreen::handle_key(key, state, toasts),
            Screen::Detail(id, state) => DetailScreen::handle_key(key, toasts, id, state),
            Screen::Timeline => TimelineScreen::handle_key(key, toasts),
            Screen::Stats => StatsScreen::handle_key(key, toasts),
        }
    }

    pub fn tick(&mut self) {
        match self {
            Screen::Home(_) => HomeScreen::tick(),
            Screen::Sessions(_) => SessionsScreen::tick(),
            _ => {}
        }
    }
}
