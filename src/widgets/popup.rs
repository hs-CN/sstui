use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout, Rect},
    style::{Styled, Stylize},
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
};

#[derive(PartialEq)]
pub enum PopupState {
    Yes,
    No,
    Ok,
    Cancel,
}

impl PopupState {
    pub fn update(&mut self, event: &Event) -> bool {
        if let Event::Key(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Left => {
                        if *self == PopupState::No {
                            *self = PopupState::Yes
                        }
                        return true;
                    }
                    KeyCode::Right => {
                        if *self == PopupState::Yes {
                            *self = PopupState::No
                        }
                        return true;
                    }
                    KeyCode::Tab => {
                        if *self == PopupState::Yes {
                            *self = PopupState::No
                        } else if *self == PopupState::No {
                            *self = PopupState::Yes
                        }
                        return true;
                    }
                    _ => {}
                }
            }
        }
        false
    }
}

pub struct MessageBox<'a> {
    title: &'a str,
    content: &'a str,
    style: ratatui::style::Style,
}

impl<'a> MessageBox<'a> {
    pub fn new(title: &'a str, content: &'a str) -> Self {
        Self {
            title,
            content,
            style: ratatui::style::Style::default(),
        }
    }
}

impl<'a> StatefulWidget for MessageBox<'a> {
    type State = PopupState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        // layout
        let lines: Vec<&str> = self.content.lines().collect();
        let line_count = lines.len() as u16;
        let line_width = lines.iter().map(|l| l.len()).max().unwrap().max(10) as u16;
        let [center] = Layout::vertical([Constraint::Length(line_count + 2)])
            .flex(Flex::Center)
            .areas(area);
        let [center] = Layout::horizontal([Constraint::Length(line_width + 4)])
            .flex(Flex::Center)
            .areas(center);
        let bottom = Rect {
            x: center.x,
            y: center.bottom() - 1,
            width: center.width,
            height: 1,
        };
        let [bottom_left, bottom_right] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .flex(Flex::Legacy)
                .areas(bottom);

        // widgets
        let message = Paragraph::new(self.content)
            .set_style(self.style)
            .centered()
            .block(Block::bordered().title(self.title));
        Clear.render(center, buf);
        message.render(center, buf);

        match state {
            PopupState::Yes => {
                // [Y]
                Paragraph::new("[Y]".white().on_blue())
                    .set_style(self.style)
                    .centered()
                    .render(bottom_left, buf);
                // [N]
                Paragraph::new("[N]".set_style(self.style))
                    .centered()
                    .render(bottom_right, buf);
            }
            PopupState::No => {
                // [Y]
                Paragraph::new("[Y]".set_style(self.style))
                    .centered()
                    .render(bottom_left, buf);
                // [N]
                Paragraph::new("[N]".white().on_blue())
                    .set_style(self.style)
                    .centered()
                    .render(bottom_right, buf);
            }
            PopupState::Ok => {
                // [Ok]
                Paragraph::new("[Ok]".white().on_blue())
                    .set_style(self.style)
                    .centered()
                    .render(bottom, buf);
            }
            PopupState::Cancel => {
                // [Cancel]
                Paragraph::new("[Cancel]".white().on_blue())
                    .set_style(self.style)
                    .centered()
                    .render(bottom, buf);
            }
        }
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
