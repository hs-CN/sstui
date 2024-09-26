use std::{
    io,
    path::Path,
    sync::{Arc, RwLock},
};

use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout},
    style::Stylize,
    widgets::Block,
};

use crate::{Layer, SSLocal, SSLocalManager, UserData};

use super::{messagebox_layer::MessageBoxLayer, show};

pub struct MainLayer {
    exit: Arc<RwLock<bool>>,
    userdata: UserData,
    sslocal: Option<SSLocal>,
}

impl MainLayer {
    pub fn new() -> io::Result<Self> {
        let userdata = UserData::load().unwrap_or_default();
        let path = Path::new(&userdata.sslocal_exec_path);
        let sslocal = if path.exists() {
            Some(SSLocal::new(path.to_path_buf()))
        } else if let Ok(Some(path)) = SSLocalManager::find_ss_exec_path() {
            Some(SSLocal::new(path))
        } else {
            None
        };

        Ok(Self {
            exit: Arc::new(RwLock::new(false)),
            userdata,
            sslocal,
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
    }

    fn update(&mut self, event: ratatui::crossterm::event::Event) {
        if let Event::Key(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Char('q') => {
                        let exit = self.exit.clone();
                        show(
                            MessageBoxLayer::yes_or_no(
                                "Info",
                                "exit?",
                                Some(move || *exit.write().unwrap() = true),
                                None,
                            )
                            .green()
                            .on_gray(),
                        );
                    }
                    KeyCode::Esc => {
                        let exit = self.exit.clone();
                        show(
                            MessageBoxLayer::yes_or_no(
                                "Info",
                                "exit?",
                                Some(move || *exit.write().unwrap() = true),
                                None,
                            )
                            .green()
                            .on_gray(),
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    fn is_transparent(&self) -> bool {
        false
    }

    fn exit(&self) -> bool {
        *self.exit.read().unwrap()
    }
}
