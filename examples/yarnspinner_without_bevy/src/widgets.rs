//! This module is here for using ratatui to interact with the terminal and
//! does not contain any code specific to yarnspinner

use ratatui::prelude::{Buffer, Rect};
use ratatui::style::{Modifier, Style, Styled};
use ratatui::widgets::{Block, Borders, List, ListState, Paragraph, StatefulWidget, Widget, Wrap};

use yarnspinner::runtime::{DialogueOption, Line, OptionId};

pub struct LineView<'a> {
    line: &'a Line,
    style: Style,
}

impl<'a> LineView<'a> {
    pub fn new(line: &'a Line) -> LineView<'a> {
        LineView {
            line,
            style: Style::new(),
        }
    }
}

impl Widget for LineView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.line.text_without_character_name().as_str())
            .style(self.style)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .title(self.line.character_name().unwrap_or_default())
                    .borders(Borders::ALL),
            )
            .render(area, buf);
    }
}

impl<'a> Styled for LineView<'a> {
    type Item = LineView<'a>;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = style.into();
        self
    }
}

pub struct OptionsViewState {
    items: Vec<DialogueOption>,
    list_state: ListState,
}

impl OptionsViewState {
    pub fn new(items: Vec<DialogueOption>) -> OptionsViewState {
        OptionsViewState {
            items,
            list_state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn selected(&self) -> Option<OptionId> {
        self.list_state.selected().map(|i| self.items[i].id)
    }

    pub fn move_cursor_up(&mut self) {
        let current_index = self.list_state.selected().unwrap_or(0);

        let new_index = if current_index == 0 {
            self.items.len() - 1
        } else {
            current_index - 1
        };

        self.list_state.select(Some(new_index));
    }

    pub fn move_cursor_down(&mut self) {
        let current_index = self.list_state.selected().unwrap_or(0);

        self.list_state
            .select(Some((current_index + 1) % self.items.len()));
    }
}

#[derive(Default)]
pub struct OptionsView {
    style: Style,
}

impl StatefulWidget for OptionsView {
    type State = OptionsViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let widget = List::new(state.items.iter().map(|o| o.line.text.clone()))
            .style(self.style)
            .block(Block::default().title("Options").borders(Borders::ALL))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));

        StatefulWidget::render(widget, area, buf, &mut state.list_state)
    }
}

impl Styled for OptionsView {
    type Item = OptionsView;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = style.into();
        self
    }
}

#[derive(Default)]
pub struct ContinueView {
    style: Style,
}

impl Widget for ContinueView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let widget = List::new(["Continue"])
            .style(self.style)
            .block(Block::default().title("Options").borders(Borders::ALL))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));

        StatefulWidget::render(
            widget,
            area,
            buf,
            &mut ListState::default().with_selected(Some(0)),
        )
    }
}

impl Styled for ContinueView {
    type Item = ContinueView;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = style.into();
        self
    }
}
