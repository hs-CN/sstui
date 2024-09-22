use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Styled, Stylize},
    text::Line,
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
};

#[derive(PartialEq)]
pub enum MessageBoxState {
    Yes,
    No,
}

pub struct MessageBox<'a> {
    title: &'a str,
    content: &'a str,
    style: ratatui::style::Style,
}

impl<'a> MessageBox<'a> {
    pub fn new(tittle: &'a str, content: &'a str) -> Self {
        Self {
            title: tittle,
            content,
            style: ratatui::style::Style::default(),
        }
    }
}

impl<'a> StatefulWidget for MessageBox<'a> {
    type State = MessageBoxState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        // layout
        let lines: Vec<&str> = self.content.lines().collect();
        let line_count = lines.len() as u16;
        let line_width = lines.iter().map(|l| l.len()).max().unwrap() as u16;
        let [center] = Layout::vertical([Constraint::Length(line_count + 2)])
            .flex(Flex::Center)
            .areas(area);
        let [center] = Layout::horizontal([Constraint::Length(line_width + 4)])
            .flex(Flex::Center)
            .areas(center);
        let [bottom_left, bottom_right] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .flex(Flex::Legacy)
                .areas(Rect {
                    x: center.x,
                    y: center.bottom() - 1,
                    width: center.width,
                    height: 1,
                });

        // widgets
        let message = Paragraph::new(self.content)
            .set_style(self.style)
            .centered()
            .block(Block::bordered().title(self.title));
        let (yes, no) = if *state == MessageBoxState::Yes {
            (
                Paragraph::new(Line::from(vec![
                    " ".into(),
                    "<Y>".white().on_blue(),
                    " ".into(),
                ]))
                .set_style(self.style)
                .centered(),
                Paragraph::new(Line::from(" <N> ").set_style(self.style)).centered(),
            )
        } else {
            (
                Paragraph::new(Line::from(" <Y> ").set_style(self.style)).centered(),
                Paragraph::new(Line::from(vec![
                    " ".into(),
                    "<N>".white().on_blue(),
                    " ".into(),
                ]))
                .set_style(self.style)
                .centered(),
            )
        };

        // render
        Clear.render(center, buf);
        message.render(center, buf);
        yes.render(bottom_left, buf);
        no.render(bottom_right, buf);
    }
}

impl<'a> Styled for MessageBox<'a> {
    type Item = Self;

    fn style(&self) -> ratatui::prelude::Style {
        self.style
    }

    fn set_style<S: Into<ratatui::prelude::Style>>(self, style: S) -> Self::Item {
        Self {
            title: self.title,
            content: self.content,
            style: style.into(),
        }
    }
}
