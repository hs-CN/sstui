use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Styled, Stylize},
    widgets::{Block, Clear, Paragraph},
};

use super::Layer;

#[derive(PartialEq)]
enum MessageBoxType {
    YesOrNo,
    Ok,
    Cancel,
}

pub struct MessageBoxLayer<F: FnOnce() + Send + Sync + 'static> {
    type_: MessageBoxType,
    title: String,
    message: String,
    exit: bool,
    style: Style,
    on_yes: Option<F>,
    on_no: Option<F>,
    on_ok: Option<F>,
    on_cancel: Option<F>,
    yes: bool,
}

impl<F: FnOnce() + Send + Sync + 'static> MessageBoxLayer<F> {
    pub fn yes_or_no<S: Into<String>>(
        title: S,
        message: S,
        on_yes: Option<F>,
        on_no: Option<F>,
    ) -> Self {
        Self {
            type_: MessageBoxType::YesOrNo,
            title: title.into(),
            message: message.into(),
            exit: false,
            style: Style::default(),
            on_yes,
            on_no,
            on_ok: None,
            on_cancel: None,
            yes: false,
        }
    }

    pub fn ok<S: Into<String>>(title: S, message: S, on_ok: Option<F>) -> Self {
        Self {
            type_: MessageBoxType::Ok,
            title: title.into(),
            message: message.into(),
            exit: false,
            style: Style::default(),
            on_yes: None,
            on_no: None,
            on_ok,
            on_cancel: None,
            yes: false,
        }
    }

    pub fn cancel<S: Into<String>>(title: S, message: S, on_cancel: Option<F>) -> Self {
        Self {
            type_: MessageBoxType::Cancel,
            title: title.into(),
            message: message.into(),
            exit: false,
            style: Style::default(),
            on_yes: None,
            on_no: None,
            on_ok: None,
            on_cancel,
            yes: false,
        }
    }
}

impl<F: FnOnce() + Send + Sync + 'static> Layer for MessageBoxLayer<F> {
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
            MessageBoxType::YesOrNo => {
                let [bottom_left, bottom_right] =
                    Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .flex(Flex::Legacy)
                        .areas(bottom);
                if self.yes {
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
            MessageBoxType::Ok => {
                let ok = Paragraph::new("[Ok]".white().on_blue())
                    .set_style(self.style)
                    .centered();
                frame.render_widget(ok, bottom);
            }
            MessageBoxType::Cancel => {
                let cancel = Paragraph::new("[Cancel]".white().on_blue())
                    .set_style(self.style)
                    .centered();
                frame.render_widget(cancel, bottom);
            }
        }
    }

    fn update(&mut self, event: Event) {
        if let Event::Key(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                if self.type_ == MessageBoxType::YesOrNo {
                    match key_event.code {
                        KeyCode::Left => self.yes = true,
                        KeyCode::Right => self.yes = false,
                        KeyCode::Tab => self.yes = !self.yes,
                        KeyCode::Esc => self.exit = true,
                        KeyCode::Enter => {
                            if self.yes {
                                if let Some(f) = self.on_yes.take() {
                                    f();
                                }
                            } else {
                                if let Some(f) = self.on_no.take() {
                                    f();
                                }
                            }
                            self.exit = true;
                        }
                        _ => {}
                    }
                } else {
                    match key_event.code {
                        KeyCode::Esc => self.exit = true,
                        KeyCode::Enter => {
                            if let Some(f) = self.on_ok.take() {
                                f();
                            } else if let Some(f) = self.on_cancel.take() {
                                f();
                            }
                            self.exit = true;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn is_transparent(&self) -> bool {
        true
    }

    fn exit(&self) -> bool {
        self.exit
    }
}

impl<F: FnOnce() + Send + Sync + 'static> Styled for MessageBoxLayer<F> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = style.into();
        self
    }
}
