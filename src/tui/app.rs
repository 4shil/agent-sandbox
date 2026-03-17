use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

use crate::tui::{Theme, AppEvent, EventLoop, Toast};
use crate::tui::widgets::ToastManager;
use crate::screens::{Screen, SessionsState, filtered_sessions};

pub struct App {
    pub running: bool,
    pub current_screen: Screen,
    pub theme: Theme,
    pub toasts: ToastManager,
    pub screen_size: Rect,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            current_screen: Screen::Home,
            theme: Theme::dark(),
            toasts: ToastManager::new(),
            screen_size: Rect::default(),
        }
    }

    pub fn run(&mut self, terminal: &mut crate::tui::Terminal) -> Result<()> {
        let event_loop = EventLoop::new(std::time::Duration::from_millis(250));

        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            if let Some(event) = event_loop.next() {
                self.handle_event(event);
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Key(key) => {
                use crossterm::event::KeyCode;
                let action = match key.code {
                    KeyCode::Char('q') if matches!(self.current_screen, Screen::Home) => Some(AppAction::Quit),
                    KeyCode::Char('q') if !matches!(self.current_screen, Screen::Home) => Some(AppAction::GoHome),
                    KeyCode::Char('s') if matches!(self.current_screen, Screen::Home) => Some(AppAction::GoSessions),
                    KeyCode::Char('d') if matches!(self.current_screen, Screen::Home) => Some(AppAction::GoStats),
                    KeyCode::Char('t') if matches!(self.current_screen, Screen::Home) => Some(AppAction::GoTimeline),
                    KeyCode::Enter if matches!(self.current_screen, Screen::Sessions(_)) => Some(AppAction::OpenDetail),
                    KeyCode::Esc => Some(AppAction::GoHome),
                    _ => None,
                };
                
                if let Some(action) = action {
                    match action {
                        AppAction::Quit => self.running = false,
                        AppAction::GoHome => self.current_screen = Screen::Home,
                        AppAction::GoSessions => self.current_screen = Screen::Sessions(SessionsState::default()),
                        AppAction::GoStats => self.current_screen = Screen::Stats,
                        AppAction::GoTimeline => self.current_screen = Screen::Timeline,
                        AppAction::OpenDetail => {
                            if let Screen::Sessions(state) = &self.current_screen {
                                let sessions = filtered_sessions(state);
                                if let Some(item) = sessions.get(state.selected) {
                                    self.current_screen = Screen::Detail(item.name.clone());
                                }
                            }
                        }
                    }
                } else {
                    self.current_screen.handle_key(key, &mut self.toasts);
                }
            }
            AppEvent::Resize(w, h) => {
                self.screen_size = Rect::new(0, 0, w, h);
            }
            AppEvent::Tick => {
                self.current_screen.tick();
            }
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        self.screen_size = frame.area();
        self.toasts.cleanup();
        self.current_screen.render(frame, self.screen_size, &self.theme);
        self.toasts.render(frame, self.screen_size, &self.theme);
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}

enum AppAction {
    Quit,
    GoHome,
    GoSessions,
    GoStats,
    GoTimeline,
    OpenDetail,
}
