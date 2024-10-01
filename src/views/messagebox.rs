use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Styled, Stylize},
    text::Text,
    widgets::{block::Title, Block, Clear, Paragraph},
};

use crate::Layer;

pub struct MessageBoxLayer<'a> {
    style: Style,
    title: Title<'a>,
    message: Text<'a>,
    exit: bool,
}

impl<'a> MessageBoxLayer<'a> {
    pub fn new<T: Into<Title<'a>>, M: Into<Text<'a>>>(title: T, message: M) -> Self {
        Self {
            style: Style::default(),
            title: title.into(),
            message: message.into(),
            exit: false,
        }
    }
}

impl<'a> Styled for MessageBoxLayer<'a> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        Self {
            style: style.into(),
            ..self
        }
    }
}

impl<'a> Layer for MessageBoxLayer<'a> {
    fn view(&mut self, frame: &mut ratatui::Frame) {
        let bottom = message_box_body(self.title.clone(), self.message.clone(), self.style, frame);
        let ok = Paragraph::new("[Ok]".white().on_blue())
            .set_style(self.style)
            .centered();
        frame.render_widget(ok, bottom);
    }

    fn before_show(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn update(&mut self, event: Option<Event>) -> std::io::Result<()> {
        if let Some(event) = event {
            if let Event::Key(key_event) = event {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Enter => self.exit = true,
                        KeyCode::Esc => self.exit = true,
                        _ => {}
                    }
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

#[derive(PartialEq)]
pub enum YesNoMessageBoxResult {
    Yes,
    No,
}

impl YesNoMessageBoxResult {
    pub fn is_yes(&self) -> bool {
        *self == YesNoMessageBoxResult::Yes
    }

    pub fn is_no(&self) -> bool {
        *self == YesNoMessageBoxResult::No
    }
}

pub struct YesNoMessageBoxLayer<'a> {
    style: Style,
    title: Title<'a>,
    message: Text<'a>,
    exit: bool,
    pub result: YesNoMessageBoxResult,
}

impl<'a> YesNoMessageBoxLayer<'a> {
    pub fn new<T: Into<Title<'a>>, M: Into<Text<'a>>>(title: T, message: M) -> Self {
        Self {
            style: Style::default(),
            title: title.into(),
            message: message.into(),
            exit: false,
            result: YesNoMessageBoxResult::No,
        }
    }
}

impl<'a> Styled for YesNoMessageBoxLayer<'a> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        Self {
            style: style.into(),
            ..self
        }
    }
}

impl<'a> Layer for YesNoMessageBoxLayer<'a> {
    fn view(&mut self, frame: &mut ratatui::Frame) {
        let bottom = message_box_body(self.title.clone(), self.message.clone(), self.style, frame);
        let [bottom_left, bottom_right] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .flex(Flex::Legacy)
                .areas(bottom);
        if self.result == YesNoMessageBoxResult::Yes {
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

    fn before_show(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn update(&mut self, event: Option<Event>) -> std::io::Result<()> {
        if let Some(event) = event {
            if let Event::Key(key_event) = event {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Left => self.result = YesNoMessageBoxResult::Yes,
                        KeyCode::Right => self.result = YesNoMessageBoxResult::No,
                        KeyCode::Tab => {
                            self.result = if self.result == YesNoMessageBoxResult::Yes {
                                YesNoMessageBoxResult::No
                            } else {
                                YesNoMessageBoxResult::Yes
                            }
                        }
                        KeyCode::Enter => self.exit = true,
                        KeyCode::Esc => {
                            self.result = YesNoMessageBoxResult::No;
                            self.exit = true;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn close(&mut self) {
        self.exit = true
    }

    fn is_exit(&self) -> bool {
        self.exit
    }
}

pub enum CancelableMessageBoxResult<T> {
    Cancel,
    Complete(T),
}

pub struct CancelableMessageBoxLayer<'a, T> {
    style: Style,
    title: Title<'a>,
    message: Text<'a>,
    exit: bool,
    task: Option<std::thread::JoinHandle<T>>,
    pub result: CancelableMessageBoxResult<T>,
}

impl<'a, T> CancelableMessageBoxLayer<'a, T> {
    pub fn new<TI: Into<Title<'a>>, M: Into<Text<'a>>>(
        title: TI,
        message: M,
        task: std::thread::JoinHandle<T>,
    ) -> Self {
        Self {
            style: Style::default(),
            title: title.into(),
            message: message.into(),
            exit: false,
            task: Some(task),
            result: CancelableMessageBoxResult::Cancel,
        }
    }
}

impl<'a, T> Styled for CancelableMessageBoxLayer<'a, T> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        Self {
            style: style.into(),
            ..self
        }
    }
}

impl<'a, T> Layer for CancelableMessageBoxLayer<'a, T> {
    fn view(&mut self, frame: &mut ratatui::Frame) {
        let bottom = message_box_body(self.title.clone(), self.message.clone(), self.style, frame);
        let cancel = Paragraph::new("[Cancel]".white().on_blue())
            .set_style(self.style)
            .centered();
        frame.render_widget(cancel, bottom);
    }

    fn before_show(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn update(&mut self, event: Option<Event>) -> std::io::Result<()> {
        if self.task.as_ref().unwrap().is_finished() {
            match self.task.take().unwrap().join() {
                Ok(result) => self.result = CancelableMessageBoxResult::Complete(result),
                Err(err) => {
                    MessageBoxLayer::new("Error", format!("{:?}", err))
                        .red()
                        .on_gray()
                        .show()?;
                }
            }
            self.exit = true;
        }

        if !self.exit {
            if let Some(event) = event {
                if let Event::Key(key_event) = event {
                    if key_event.kind == KeyEventKind::Press {
                        match key_event.code {
                            KeyCode::Enter => self.exit = true,
                            KeyCode::Esc => self.exit = true,
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn close(&mut self) {
        self.exit = true
    }

    fn is_exit(&self) -> bool {
        self.exit
    }
}

fn message_box_body<'a>(
    title: Title<'a>,
    message: Text<'a>,
    style: Style,
    frame: &mut ratatui::Frame,
) -> Rect {
    let [center] = Layout::vertical([Constraint::Length(message.lines.len() as u16 + 2)])
        .flex(Flex::Center)
        .areas(frame.area());
    let [center] = Layout::horizontal([Constraint::Length(message.width().max(10) as u16 + 4)])
        .flex(Flex::Center)
        .areas(center);
    let bottom = Rect {
        x: center.x,
        y: center.bottom() - 1,
        width: center.width,
        height: 1,
    };

    frame.render_widget(Clear, center);
    let message = Paragraph::new(message)
        .set_style(style)
        .centered()
        .block(Block::bordered().title(title));
    frame.render_widget(message, center);

    bottom
}
