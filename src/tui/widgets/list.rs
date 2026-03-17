#![allow(dead_code)]

use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block as TuiBlock, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use crate::tui::Theme;

pub struct ScrollList<'a> {
    pub items: Vec<ListItem<'a>>,
    pub state: ListState,
    pub scrollbar_state: ScrollbarState,
    pub title: String,
    pub focused: bool,
}

impl<'a> ScrollList<'a> {
    pub fn new(items: Vec<ListItem<'a>>, title: String) -> Self {
        let len = items.len();
        Self {
            items,
            state: ListState::default(),
            scrollbar_state: ScrollbarState::new(len),
            title,
            focused: true,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => { if i >= self.items.len().saturating_sub(1) { 0 } else { i + 1 } }
            None => 0,
        };
        self.select(Some(i));
    }

    pub fn prev(&mut self) {
        let i = match self.state.selected() {
            Some(i) => { if i == 0 { self.items.len().saturating_sub(1) } else { i - 1 } }
            None => 0,
        };
        self.select(Some(i));
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.state.select(index);
        self.scrollbar_state = self.scrollbar_state.position(index.unwrap_or(0));
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let border_style = if self.focused {
            theme.focused_border_style()
        } else {
            theme.unfocused_border_style()
        };

        let selected_style = if self.focused {
            theme.selected_style()
        } else {
            theme.muted_style()
        };

        let block = TuiBlock::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.title))
            .title_style(if self.focused { theme.title_style() } else { theme.muted_style() })
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let list = List::new(self.items.clone())
            .highlight_style(selected_style)
            .highlight_symbol("▸ ");

        frame.render_stateful_widget(list, inner, &mut self.state);

        // Scrollbar
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(theme.muted));

        let scrollbar_area = Rect {
            x: area.right() - 1,
            y: area.y + 1,
            width: 1,
            height: area.height.saturating_sub(2),
        };
        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut self.scrollbar_state);
    }
}
