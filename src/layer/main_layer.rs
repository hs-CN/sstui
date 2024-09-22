use std::{fs, io};

use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout},
    style::Stylize,
    widgets::Block,
};

use crate::{
    widgets::{MessageBox, MessageBoxState},
    Layer, SSLocal, UserData,
};

use super::SSLocalManagerLayer;

pub struct MainLayer {
    exit: bool,
    next_layer: Option<Box<dyn Layer>>,
    userdata: UserData,
    sslocal: Option<SSLocal>,
    message: Option<String>,
    messagebox_state: MessageBoxState,
}

impl MainLayer {
    pub fn new() -> io::Result<Self> {
        let userdata = UserData::load().unwrap_or_default();
        let sslocal = if fs::exists(&userdata.sslocal_exec_path)? {
            Some(SSLocal::new(&userdata.sslocal_exec_path))
        } else {
            None
        };
        let message = if sslocal.is_none() {
            Some("'sslocal' not found, download it?".to_string())
        } else {
            None
        };

        Ok(Self {
            exit: false,
            next_layer: None,
            userdata,
            sslocal,
            message,
            messagebox_state: MessageBoxState::Yes,
        })
    }
}

impl Layer for MainLayer {
    fn view(&mut self, frame: &mut ratatui::Frame) {
        let [main, log] = Layout::vertical([Constraint::Percentage(70), Constraint::Max(15)])
            .flex(Flex::Legacy)
            .areas(frame.area());
        let main_block = Block::bordered().title("shadowsocks servers");
        let log_block = Block::bordered().title("log");
        frame.render_widget(main_block, main);
        frame.render_widget(log_block, log);

        if let Some(msg) = self.message.as_ref() {
            frame.render_stateful_widget(
                MessageBox::new("Info", msg).green().on_gray(),
                frame.area(),
                &mut self.messagebox_state,
            );
        }
    }

    fn update(&mut self, event: ratatui::crossterm::event::Event) {
        if self.message.is_some() {
            if let Event::Key(key_event) = event {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Left => self.messagebox_state = MessageBoxState::Yes,
                        KeyCode::Right => self.messagebox_state = MessageBoxState::No,
                        KeyCode::Tab => {
                            if self.messagebox_state == MessageBoxState::Yes {
                                self.messagebox_state = MessageBoxState::No
                            } else {
                                self.messagebox_state = MessageBoxState::Yes
                            }
                        }
                        KeyCode::Esc => self.message = None,
                        KeyCode::Enter => {
                            self.message = None;
                            if self.messagebox_state == MessageBoxState::Yes {
                                self.next_layer = Some(Box::new(SSLocalManagerLayer::new()));
                            }
                        }
                        _ => {}
                    }
                }
            }
        } else {
            if let Event::Key(key_event) = event {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char('q') => self.exit = true,
                        KeyCode::Esc => self.exit = true,
                        _ => {}
                    }
                }
            }
        }
    }

    fn next_layer(&mut self) -> Option<Box<dyn Layer>> {
        self.next_layer.take()
    }

    fn exit(&self) -> bool {
        self.exit
    }
}
