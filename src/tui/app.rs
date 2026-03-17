use anyhow::Result;
use ratatui::{layout::Rect, Frame};
use std::process::Command;
use crossterm::{execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, event::{DisableMouseCapture, EnableMouseCapture}};

use crate::tui::{Theme, AppEvent, EventLoop};
use crate::tui::widgets::ToastManager;
use crate::screens::{Screen, SessionsState, DetailState, HomeState, selected_agent, filtered_sessions};

pub struct App {
    pub running: bool,
    pub current_screen: Screen,
    pub theme: Theme,
    pub toasts: ToastManager,
    pub screen_size: Rect,
    pub show_help: bool,
    pub goto_prefix: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            current_screen: Screen::Home(HomeState::default()),
            theme: Theme::dark(),
            toasts: ToastManager::new(),
            screen_size: Rect::default(),
            show_help: false,
            goto_prefix: false,
        }
    }

    pub fn run(&mut self, terminal: &mut crate::tui::Terminal) -> Result<()> {
        let event_loop = EventLoop::new(std::time::Duration::from_millis(50));

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
                use crossterm::event::{KeyCode, KeyModifiers};

                // Ctrl+C / Ctrl+Q always quit
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('c') | KeyCode::Char('q') => {
                            self.running = false;
                            return;
                        }
                        _ => {}
                    }
                }

                if self.show_help {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('?') => self.show_help = false,
                        _ => {}
                    }
                    return;
                }

                if self.goto_prefix {
                    self.goto_prefix = false;
                    match key.code {
                        KeyCode::Char('h') => self.current_screen = Screen::Home(HomeState::default()),
                        KeyCode::Char('s') => self.current_screen = Screen::Sessions(SessionsState::default()),
                        KeyCode::Char('t') => self.current_screen = Screen::Timeline,
                        KeyCode::Char('d') => self.current_screen = Screen::Stats,
                        _ => {}
                    }
                    return;
                }

                let action = match key.code {
                    KeyCode::Char('?') => Some(AppAction::ToggleHelp),
                    KeyCode::Char('g') => Some(AppAction::StartGoto),
                    KeyCode::Char('/') => Some(AppAction::FocusSearch),
                    KeyCode::Char('q') if matches!(self.current_screen, Screen::Home(_)) => Some(AppAction::Quit),
                    KeyCode::Char('q') if !matches!(self.current_screen, Screen::Home(_)) => Some(AppAction::GoHome),
                    KeyCode::Char('s') if matches!(self.current_screen, Screen::Home(_)) => Some(AppAction::GoSessions),
                    KeyCode::Char('d') if matches!(self.current_screen, Screen::Home(_)) => Some(AppAction::GoStats),
                    KeyCode::Char('t') if matches!(self.current_screen, Screen::Home(_)) => Some(AppAction::GoTimeline),
                    KeyCode::Enter if matches!(self.current_screen, Screen::Home(_)) => Some(AppAction::LaunchSelectedAgent),
                    KeyCode::Enter if matches!(self.current_screen, Screen::Sessions(_)) => Some(AppAction::OpenDetail),
                    KeyCode::Char('c') if matches!(self.current_screen, Screen::Sessions(_)) => Some(AppAction::ContinueSession),
                    KeyCode::Esc => Some(AppAction::GoBack),
                    _ => None,
                };

                if let Some(action) = action {
                    match action {
                        AppAction::Quit => self.running = false,
                        AppAction::GoHome => self.current_screen = Screen::Home(HomeState::default()),
                        AppAction::GoSessions => self.current_screen = Screen::Sessions(SessionsState::default()),
                        AppAction::GoStats => self.current_screen = Screen::Stats,
                        AppAction::GoTimeline => self.current_screen = Screen::Timeline,
                        AppAction::OpenDetail => {
                            if let Screen::Sessions(state) = &self.current_screen {
                                let sessions = filtered_sessions(state);
                                if let Some(item) = sessions.get(state.selected) {
                                    self.current_screen = Screen::Detail(item.name.clone(), DetailState::default());
                                }
                            }
                        }
                        AppAction::GoBack => {
                            self.current_screen = match &self.current_screen {
                                Screen::Detail(_, _) => Screen::Sessions(SessionsState::default()),
                                Screen::Sessions(_) => Screen::Home(HomeState::default()),
                                Screen::Timeline => Screen::Home(HomeState::default()),
                                Screen::Stats => Screen::Home(HomeState::default()),
                                Screen::Home(_) => Screen::Home(HomeState::default()),
                            };
                        }
                        AppAction::ToggleHelp => self.show_help = !self.show_help,
                        AppAction::StartGoto => self.goto_prefix = true,
                        AppAction::FocusSearch => {
                            if !matches!(self.current_screen, Screen::Sessions(_)) {
                                self.current_screen = Screen::Sessions(SessionsState::default());
                            }
                            if let Screen::Sessions(state) = &mut self.current_screen {
                                state.searching = true;
                            }
                        }
                        AppAction::LaunchSelectedAgent => {
                            if let Screen::Home(state) = &self.current_screen {
                                if let Some(agent) = selected_agent(state) {
                                    let _ = self.run_external_agent(agent, None);
                                } else {
                                    self.toasts.push(crate::tui::widgets::Toast::warning("No installed agent found".to_string()));
                                }
                            }
                        }
                        AppAction::ContinueSession => {
                            if let Screen::Sessions(state) = &self.current_screen {
                                let sessions = filtered_sessions(state);
                                if let Some(item) = sessions.get(state.selected) {
                                    let _ = self.run_external_agent(&item.agent, Some(&item.name));
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
        if self.show_help {
            self.render_help_overlay(frame);
        }
        self.toasts.render(frame, self.screen_size, &self.theme);
    }

    fn render_help_overlay(&self, frame: &mut Frame) {
        use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

        let area = Rect {
            x: self.screen_size.width / 8,
            y: self.screen_size.height / 8,
            width: self.screen_size.width.saturating_mul(3) / 4,
            height: self.screen_size.height.saturating_mul(3) / 4,
        };

        let text = "Global\n  ?           Toggle help\n  g + h/s/t/d Jump screens\n  /           Focus sessions search\n  Esc         Back\n\nHome\n  s           Sessions\n  t           Timeline\n  d           Stats\n  q           Quit\n\nSessions\n  /           Search mode\n  ↑/↓         Navigate\n  Enter       Open detail\n\nDetail\n  ↑/↓         Select action\n  Enter/Space Expand action JSON\n  Esc         Back\n";

        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new(text)
                .wrap(Wrap { trim: false })
                .style(self.theme.default_style())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Keybindings ")
                        .title_style(self.theme.title_style())
                        .border_style(self.theme.focused_border_style()),
                ),
            area,
        );
    }

    fn run_external_agent(&mut self, agent: &str, workspace: Option<&str>) -> Result<()> {
        let mut stdout = std::io::stdout();
        disable_raw_mode()?;
        execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;

        let status = if let Some(name) = workspace {
            let dir = crate::session::get_workspaces_dir().join(name);
            Command::new(agent).current_dir(dir).status()
        } else {
            // Run agent directly (not through abox)
            Command::new(agent).status()
        };

        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        enable_raw_mode()?;

        match &status {
            Ok(s) if s.success() => {}
            Ok(s) => {
                self.toasts.push(crate::tui::widgets::Toast::warning(
                    format!("{} exited with {:?}", agent, s.code())
                ));
            }
            Err(e) => {
                self.toasts.push(crate::tui::widgets::Toast::error(
                    format!("failed to launch {}: {}", agent, e)
                ));
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
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
    GoBack,
    ToggleHelp,
    StartGoto,
    FocusSearch,
    LaunchSelectedAgent,
    ContinueSession,
}
