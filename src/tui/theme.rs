use ratatui::style::{Color, Modifier, Style};

#[derive(Clone, Debug)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub border: Color,
    pub border_focused: Color,
    pub highlight_bg: Color,
    pub muted: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            bg: Color::Rgb(18, 18, 28),
            fg: Color::Rgb(220, 220, 230),
            primary: Color::Rgb(100, 180, 255),
            secondary: Color::Rgb(180, 130, 255),
            accent: Color::Rgb(255, 180, 60),
            success: Color::Rgb(80, 220, 140),
            warning: Color::Rgb(255, 200, 60),
            error: Color::Rgb(255, 80, 80),
            border: Color::Rgb(60, 60, 80),
            border_focused: Color::Rgb(100, 180, 255),
            highlight_bg: Color::Rgb(40, 40, 60),
            muted: Color::Rgb(100, 100, 120),
        }
    }

    pub fn ocean() -> Self {
        Self {
            bg: Color::Rgb(10, 20, 35),
            fg: Color::Rgb(200, 220, 240),
            primary: Color::Rgb(0, 180, 220),
            secondary: Color::Rgb(80, 200, 200),
            accent: Color::Rgb(255, 140, 100),
            success: Color::Rgb(60, 200, 120),
            warning: Color::Rgb(255, 200, 60),
            error: Color::Rgb(255, 80, 100),
            border: Color::Rgb(40, 60, 80),
            border_focused: Color::Rgb(0, 180, 220),
            highlight_bg: Color::Rgb(20, 40, 60),
            muted: Color::Rgb(80, 100, 120),
        }
    }

    pub fn neon() -> Self {
        Self {
            bg: Color::Rgb(5, 5, 15),
            fg: Color::Rgb(240, 240, 255),
            primary: Color::Rgb(0, 255, 200),
            secondary: Color::Rgb(255, 0, 200),
            accent: Color::Rgb(255, 255, 0),
            success: Color::Rgb(0, 255, 100),
            warning: Color::Rgb(255, 200, 0),
            error: Color::Rgb(255, 50, 80),
            border: Color::Rgb(30, 30, 60),
            border_focused: Color::Rgb(0, 255, 200),
            highlight_bg: Color::Rgb(20, 20, 50),
            muted: Color::Rgb(80, 80, 120),
        }
    }

    pub fn primary_style(&self) -> Style {
        Style::default().fg(self.primary)
    }

    pub fn title_style(&self) -> Style {
        Style::default().fg(self.primary).add_modifier(Modifier::BOLD)
    }

    pub fn focused_border_style(&self) -> Style {
        Style::default().fg(self.border_focused)
    }

    pub fn unfocused_border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn selected_style(&self) -> Style {
        Style::default().bg(self.highlight_bg).fg(self.primary).add_modifier(Modifier::BOLD)
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted)
    }

    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent).add_modifier(Modifier::BOLD)
    }

    pub fn default_style(&self) -> Style {
        Style::default().fg(self.fg).bg(self.bg)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
