use std::io;

use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Styled, Stylize},
    widgets::{Block, Clear, Paragraph},
};

use super::Layer;

#[derive(PartialEq)]
pub enum MessageBoxResult {
    Yes,
    No,
    None,
    Ok,
    Cancel,
}

impl MessageBoxResult {
    #[inline]
    pub fn is_yes(&self) -> bool {
        *self == MessageBoxResult::Yes
    }

    #[inline]
    pub fn is_no(&self) -> bool {
        *self == MessageBoxResult::No
    }

    #[inline]
    pub fn is_ok(&self) -> bool {
        *self == MessageBoxResult::Ok
    }

    #[inline]
    pub fn is_cancel(&self) -> bool {
        *self == MessageBoxResult::Cancel
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Type {
    YesOrNo,
    Ok,
    Cancel,
}

pub struct MessageBoxLayer {
    style: Style,
    type_: Type,
    title: String,
    message: String,
    exit: bool,
    pub result: MessageBoxResult,
}

impl MessageBoxLayer {
    pub fn yes_or_no<T: Into<String>, M: Into<String>>(title: T, message: M) -> Self {
        Self {
            style: Style::default(),
            type_: Type::YesOrNo,
            title: title.into(),
            message: message.into(),
            exit: false,
            result: MessageBoxResult::No,
        }
    }

    pub fn ok<T: Into<String>, M: Into<String>>(title: T, message: M) -> Self {
        Self {
            style: Style::default(),
            type_: Type::Ok,
            title: title.into(),
            message: message.into(),
            exit: false,
            result: MessageBoxResult::Ok,
        }
    }

    pub fn cancel<T: Into<String>, M: Into<String>>(title: T, message: M) -> Self {
        Self {
            style: Style::default(),
            type_: Type::Cancel,
            title: title.into(),
            message: message.into(),
            exit: false,
            result: MessageBoxResult::None,
        }
    }
}

impl Layer for MessageBoxLayer {
    fn view(&mut self, frame: &mut ratatui::Frame) {
        let lines: Vec<&str> = self.message.lines().collect();
        let line_count = lines.len() as u16;
        let line_width = lines.iter().map(|l| l.len()).max().unwrap().max(10) as u16;
        let [center] = Layout::vertical([Constraint::Length(line_count + 2)])
            .flex(Flex::Center)
            .areas(frame.area());
        let [center] = Layout::horizontal([Constraint::Length(line_width + 4)])
            .flex(Flex::Center)
            .areas(center);
        let bottom = Rect {
            x: center.x,
            y: center.bottom() - 1,
            width: center.width,
            height: 1,
        };

        frame.render_widget(Clear, center);
        let message = Paragraph::new(self.message.as_str())
            .set_style(self.style)
            .centered()
            .block(Block::bordered().title(self.title.as_str()));
        frame.render_widget(message, center);

        match self.type_ {
            Type::YesOrNo => {
                let [bottom_left, bottom_right] =
                    Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .flex(Flex::Legacy)
                        .areas(bottom);
                if self.result.is_yes() {
                    let yes = Paragraph::new("[Y]".white().on_blue())
                        .set_style(self.style)
                        .centered();
                    frame.render_widget(yes, bottom_left);
                    let no = Paragraph::new("[N]".set_style(self.style)).centered();
                    frame.render_widget(no, bottom_right);
                } else {
                    let yes = Paragraph::new("[Y]".set_style(self.style)).centered();
                    frame.render_widget(yes, bottom_left);
                    let no = Paragraph::new("[N]".white().on_blue())
                        .set_style(self.style)
                        .centered();
                    frame.render_widget(no, bottom_right);
                }
            }
            Type::Ok => {
                let ok = Paragraph::new("[Ok]".white().on_blue())
                    .set_style(self.style)
                    .centered();
                frame.render_widget(ok, bottom);
            }
            Type::Cancel => {
                let cancel = Paragraph::new("[Cancel]".white().on_blue())
                    .set_style(self.style)
                    .centered();
                frame.render_widget(cancel, bottom);
            }
        }
    }

    fn before_show(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn update(&mut self, event: Event) -> std::io::Result<()> {
        if let Event::Key(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                match (self.type_, key_event.code) {
                    (Type::YesOrNo, KeyCode::Left) => self.result = MessageBoxResult::Yes,
                    (Type::YesOrNo, KeyCode::Right) => self.result = MessageBoxResult::No,
                    (Type::YesOrNo, KeyCode::Tab) => {
                        self.result = if self.result.is_yes() {
                            MessageBoxResult::No
                        } else {
                            MessageBoxResult::Yes
                        }
                    }
                    (Type::YesOrNo, KeyCode::Enter) => self.exit = true,
                    (Type::Ok, KeyCode::Enter) => {
                        self.result = MessageBoxResult::Ok;
                        self.exit = true;
                    }
                    (Type::Cancel, KeyCode::Enter) => {
                        self.result = MessageBoxResult::Cancel;
                        self.exit = true;
                    }
                    (_, KeyCode::Esc) => {
                        self.result = MessageBoxResult::None;
                        self.exit = true;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn is_exit(&self) -> bool {
        self.exit
    }

    fn close(&mut self) {
        self.exit = true
    }
}

impl Styled for MessageBoxLayer {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = style.into();
        self
    }
}
