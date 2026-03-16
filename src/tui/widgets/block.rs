use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block as TuiBlock, Borders, Paragraph},
    Frame,
};

use crate::tui::Theme;

pub struct StyledBlock<'a> {
    pub title: &'a str,
    pub focused: bool,
}

impl<'a> StyledBlock<'a> {
    pub fn new(title: &'a str, focused: bool) -> Self {
        Self { title, focused }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme, inner: Paragraph<'static>) {
        let border_style = if self.focused {
            theme.focused_border_style()
        } else {
            theme.unfocused_border_style()
        };

        let block = TuiBlock::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.title))
            .title_style(if self.focused {
                theme.title_style()
            } else {
                theme.muted_style()
            })
            .border_style(border_style);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(inner, inner_area);
    }
}
