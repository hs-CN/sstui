use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout},
    style::Stylize,
    widgets::{Block, Paragraph, Wrap},
};

use super::messagebox::MessageBoxLayer;
use crate::{userdata::ServerGroup, Layer};

enum State {
    Name,
    Url,
}

pub struct SSServerImportLayer {
    exit: bool,
    name: String,
    url: String,
    state: State,
    pub result: Option<ServerGroup>,
}

impl SSServerImportLayer {
    pub fn new() -> Self {
        Self {
            exit: false,
            name: String::new(),
            url: String::new(),
            state: State::Name,
            result: None,
        }
    }
}

impl Layer for SSServerImportLayer {
    fn before_show(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn view(&mut self, frame: &mut ratatui::Frame) {
        let [title_layout, name_layout, url_layout, footer_layout] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .flex(Flex::Legacy)
        .areas(frame.area());

        let title = " Add Server Group ";
        let [title_layout] = Layout::horizontal([Constraint::Length(title.len() as u16 + 2)])
            .flex(Flex::Center)
            .areas(title_layout);
        let title = Paragraph::new(title).centered().block(Block::bordered());
        frame.render_widget(title, title_layout);

        let footer =
            Paragraph::new("Next (Tab) | Clear (Del) | Confirm (Enter) | Exit (Esc)").centered();
        frame.render_widget(footer, footer_layout);

        let mut name =
            Paragraph::new(self.name.as_str()).block(Block::bordered().title("Group Name"));
        if let State::Name = self.state {
            name = name.green();
        }
        frame.render_widget(name, name_layout);

        let mut url = Paragraph::new(self.url.as_str())
            .block(Block::bordered().title("Update URL"))
            .wrap(Wrap { trim: true });
        if let State::Url = self.state {
            url = url.green();
        }
        frame.render_widget(url, url_layout);
    }

    fn update(&mut self, event: Option<ratatui::crossterm::event::Event>) -> std::io::Result<()> {
        if let Some(event) = event {
            if let Event::Key(key_event) = event {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc => self.exit = true,
                        KeyCode::Delete => {
                            match self.state {
                                State::Name => self.name.clear(),
                                State::Url => self.url.clear(),
                            };
                        }
                        KeyCode::Backspace => {
                            match self.state {
                                State::Name => self.name.pop(),
                                State::Url => self.url.pop(),
                            };
                        }
                        KeyCode::Tab => {
                            self.state = match self.state {
                                State::Name => State::Url,
                                State::Url => State::Name,
                            };
                        }
                        KeyCode::Char(c) => match self.state {
                            State::Name => self.name.push(c),
                            State::Url => self.url.push(c),
                        },
                        KeyCode::Enter => {
                            let mut result = ServerGroup::new(&self.name, &self.url);
                            match result.update() {
                                Ok(_) => {
                                    self.result = Some(result);
                                    self.exit = true;
                                }
                                Err(e) => {
                                    MessageBoxLayer::new("Error", e.to_string())
                                        .red()
                                        .on_gray()
                                        .show()?;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn close(&mut self) {
        self.exit = true;
    }

    fn is_exit(&self) -> bool {
        self.exit
    }
}
