use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Styled, Stylize},
    widgets::{Block, Clear, Paragraph},
};

use crate::Layer;

pub struct MessageBoxLayer {
    style: Style,
    title: String,
    message: String,
    exit: bool,
}

impl MessageBoxLayer {
    pub fn new<TS: Into<String>, MS: Into<String>>(title: TS, message: MS) -> Self {
        Self {
            style: Style::default(),
            title: title.into(),
            message: message.into(),
            exit: false,
        }
    }
}

impl Styled for MessageBoxLayer {
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

impl Layer for MessageBoxLayer {
    fn view(&mut self, frame: &mut ratatui::Frame) {
        let bottom = message_box_body(&self.title, &self.message, self.style, frame);
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

pub struct YesNoMessageBoxLayer {
    style: Style,
    title: String,
    message: String,
    exit: bool,
    pub result: YesNoMessageBoxResult,
}

impl YesNoMessageBoxLayer {
    pub fn new<TS: Into<String>, MS: Into<String>>(title: TS, message: MS) -> Self {
        Self {
            style: Style::default(),
            title: title.into(),
            message: message.into(),
            exit: false,
            result: YesNoMessageBoxResult::No,
        }
    }
}

impl Styled for YesNoMessageBoxLayer {
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

impl Layer for YesNoMessageBoxLayer {
    fn view(&mut self, frame: &mut ratatui::Frame) {
        let bottom = message_box_body(&self.title, &self.message, self.style, frame);
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
    Complete(std::thread::Result<T>),
}

pub struct CancelableMessageBoxLayer<T> {
    style: Style,
    title: String,
    message: String,
    exit: bool,
    task: Option<std::thread::JoinHandle<T>>,
    pub result: CancelableMessageBoxResult<T>,
}

impl<T> CancelableMessageBoxLayer<T> {
    pub fn new<TS: Into<String>, MS: Into<String>>(
        title: TS,
        message: MS,
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

impl<T> Styled for CancelableMessageBoxLayer<T> {
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

impl<T> Layer for CancelableMessageBoxLayer<T> {
    fn view(&mut self, frame: &mut ratatui::Frame) {
        let bottom = message_box_body(&self.title, &self.message, self.style, frame);
        let cancel = Paragraph::new("[Cancel]".white().on_blue())
            .set_style(self.style)
            .centered();
        frame.render_widget(cancel, bottom);
    }

    fn before_show(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn update(&mut self, event: Option<Event>) -> std::io::Result<()> {
        if let Some(task) = &self.task {
            if task.is_finished() {
                self.exit = true;
            }
        }

        if self.exit {
            self.result = CancelableMessageBoxResult::Complete(self.task.take().unwrap().join());
        } else {
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

fn message_box_body(title: &str, message: &str, style: Style, frame: &mut ratatui::Frame) -> Rect {
    let lines: Vec<&str> = message.lines().collect();
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
    let message = Paragraph::new(message)
        .set_style(style)
        .centered()
        .block(Block::bordered().title(title));
    frame.render_widget(message, center);

    bottom
}
