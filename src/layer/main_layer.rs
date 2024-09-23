use std::{io, path::Path};

use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout},
    style::Stylize,
    widgets::Block,
};

use crate::{
    widgets::{MessageBox, PopupState},
    Layer, SSLocal, SSLocalManager, UserData,
};

use super::SSLocalManagerLayer;

#[derive(PartialEq)]
enum Popup {
    Exit,
    Info(String),
    Error(String),
    SSLocalNotFound,
    SSLocalNewVersion(String),
}

pub struct MainLayer {
    exit: bool,
    next_layer: Option<Box<dyn Layer>>,
    userdata: UserData,
    sslocal: Option<SSLocal>,
    popup: Vec<(Popup, PopupState)>,
}

impl MainLayer {
    pub fn new() -> io::Result<Self> {
        let mut popup = Vec::new();
        let userdata = UserData::load().unwrap_or_default();
        let path = Path::new(&userdata.sslocal_exec_path);
        let sslocal = if path.exists() {
            Some(SSLocal::new(path.to_path_buf()))
        } else if let Ok(Some(path)) = SSLocalManager::find_ss_exec_path() {
            Some(SSLocal::new(path))
        } else {
            popup.push((Popup::SSLocalNotFound, PopupState::No));
            None
        };

        Ok(Self {
            exit: false,
            next_layer: None,
            userdata,
            sslocal,
            popup,
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

        if let Some((popup, state)) = self.popup.last_mut() {
            match popup {
                Popup::Exit => {
                    let info = "exit now?";
                    let popup = MessageBox::new("Info", info).green().on_gray();
                    frame.render_stateful_widget(popup, frame.area(), state);
                }
                Popup::Info(ref info) => {
                    let popup = MessageBox::new("Info", info).green().on_gray();
                    frame.render_stateful_widget(popup, frame.area(), state);
                }
                Popup::Error(ref err) => {
                    let popup = MessageBox::new("Error", err).red().on_gray();
                    frame.render_stateful_widget(popup, frame.area(), state);
                }
                Popup::SSLocalNotFound => {
                    let info = "sslocal not found, download it?";
                    let popup = MessageBox::new("Info", info).green().on_gray();
                    frame.render_stateful_widget(popup, frame.area(), state);
                }
                Popup::SSLocalNewVersion(ref info) => {
                    let popup = MessageBox::new("Info", info).green().on_gray();
                    frame.render_stateful_widget(popup, frame.area(), state);
                }
            }
        };
    }

    fn update(&mut self, event: ratatui::crossterm::event::Event) {
        if let Some((popup, state)) = self.popup.last_mut() {
            if let Event::Key(key_event) = event {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc => drop(self.popup.pop()),
                        KeyCode::Left => {
                            if *state == PopupState::No {
                                *state = PopupState::Yes
                            }
                        }
                        KeyCode::Right => {
                            if *state == PopupState::Yes {
                                *state = PopupState::No
                            }
                        }
                        KeyCode::Tab => {
                            if *state == PopupState::Yes {
                                *state = PopupState::No
                            } else if *state == PopupState::No {
                                *state = PopupState::Yes
                            }
                        }
                        KeyCode::Enter => {
                            if *state == PopupState::Yes {
                                match popup {
                                    Popup::Exit => self.exit = true,
                                    Popup::Info(_) => {}
                                    Popup::Error(_) => {}
                                    Popup::SSLocalNotFound => {
                                        self.next_layer = Some(Box::new(SSLocalManagerLayer::new()))
                                    }
                                    Popup::SSLocalNewVersion(_) => todo!(),
                                }
                            }
                            self.popup.pop();
                        }
                        _ => {}
                    }
                }
            }
        } else {
            if let Event::Key(key_event) = event {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char('q') => self.popup.push((Popup::Exit, PopupState::No)),
                        KeyCode::Esc => self.popup.push((Popup::Exit, PopupState::No)),
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
