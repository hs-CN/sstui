use std::{io, path::Path, thread::spawn};

use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout},
    style::Stylize,
    widgets::Block,
};

use super::messagebox_layer::MessageBoxLayer;
use crate::{Layer, SSLocal, SSLocalManager, UserData};

pub struct MainLayer {
    exit: bool,
    userdata: UserData,
    sslocal: Option<SSLocal>,
}

impl MainLayer {
    pub fn new() -> Self {
        let userdata = UserData::load().unwrap_or_default();
        let path = Path::new(&userdata.sslocal_exec_path);
        let sslocal = if path.exists() {
            Some(SSLocal::new(path.to_path_buf()))
        } else if let Ok(Some(path)) = SSLocalManager::find_ss_exec_path() {
            Some(SSLocal::new(path))
        } else {
            None
        };

        Self {
            exit: false,
            userdata,
            sslocal,
        }
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

    fn before_show(&mut self) -> io::Result<()> {
        if self.sslocal.is_none() {
            let msg = MessageBoxLayer::yes_or_no("Info", "sslocal not found, download it?")
                .green()
                .on_gray()
                .show()?;
            if msg.result.is_yes() {
                let msg = MessageBoxLayer::cancel("Info", "get latest version now...")
                    .green()
                    .on_gray()
                    .thread_safe();
                let msg_cloned = msg.clone();
                let task = spawn(move || {
                    SSLocalManager::get_latest().and_then(|latest| {
                        msg_cloned.write().unwrap().close();
                        Ok(latest)
                    })
                });
                if !msg.show()?.read().unwrap().result.is_cancel() {
                    match task.join() {
                        Ok(latest) => match latest {
                            Ok(latest) => {
                                MessageBoxLayer::ok(
                                    "Info",
                                    format!("get latest version:{}", latest.tag_name),
                                )
                                .green()
                                .on_gray()
                                .show()?;
                            }
                            Err(err) => {
                                MessageBoxLayer::ok("Info", format!("err:{}", err))
                                    .red()
                                    .on_gray()
                                    .show()?;
                            }
                        },
                        Err(err) => {
                            MessageBoxLayer::ok("Info", format!("err:{:?}", err))
                                .red()
                                .on_gray()
                                .show()?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, event: Event) -> std::io::Result<()> {
        if let Event::Key(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Esc => {
                        self.exit = MessageBoxLayer::yes_or_no("Info", "exit?")
                            .green()
                            .on_gray()
                            .show()?
                            .result
                            .is_yes();
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
        self.exit = true;
    }
}
