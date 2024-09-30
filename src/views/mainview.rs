use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout},
    style::Stylize,
    widgets::{Block, Paragraph},
};

use super::{
    messagebox::{
        CancelableMessageBoxLayer, CancelableMessageBoxResult, MessageBoxLayer,
        YesNoMessageBoxLayer,
    },
    sslocal_update::SSLocalUpdateLayer,
};
use crate::{
    sslocal::{SSLocal, SSLocalManager},
    userdata::UserData,
    Layer, Show,
};

pub struct MainLayer {
    exit: bool,
    userdata: UserData,
    sslocal: Option<SSLocal>,
}

impl MainLayer {
    pub fn new() -> Self {
        let userdata = UserData::load().unwrap_or_default();
        let path = std::path::Path::new(&userdata.sslocal_exec_path);
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

    fn sslocal_update(&mut self) -> std::io::Result<()> {
        let cancelable = CancelableMessageBoxLayer::new(
            "Info",
            "get latest version now...",
            std::thread::spawn(SSLocalManager::get_latest),
        )
        .green()
        .on_gray()
        .show()?;
        if let CancelableMessageBoxResult::Complete(result) = cancelable.result {
            match result {
                Ok(latest) => match latest {
                    Ok(latest) => {
                        let yes_no = YesNoMessageBoxLayer::new(
                            "Info",
                            format!("find latest version:{}, download it?", latest.tag_name),
                        )
                        .green()
                        .on_gray()
                        .show()?;
                        if yes_no.result.is_yes() {
                            let update = SSLocalUpdateLayer::new(latest).show()?;
                            if update.result.is_some() {
                                self.sslocal = update.result;
                            }
                        }
                    }
                    Err(err) => {
                        MessageBoxLayer::new("Info", format!("err:{}", err))
                            .red()
                            .on_gray()
                            .show()?;
                    }
                },
                Err(err) => {
                    MessageBoxLayer::new("Info", format!("err:{:?}", err))
                        .red()
                        .on_gray()
                        .show()?;
                }
            }
        }
        Ok(())
    }
}

impl Layer for MainLayer {
    fn view(&mut self, frame: &mut ratatui::Frame) {
        let [main, log, footer_layout] = Layout::vertical([
            Constraint::Percentage(70),
            Constraint::Max(15),
            Constraint::Length(1),
        ])
        .flex(Flex::Legacy)
        .areas(frame.area());
        let main_block = Block::bordered().title("shadowsocks servers");
        let log_block = Block::bordered().title("log");
        let footer = Paragraph::new("Exit (Esc)").white().on_cyan().centered();
        frame.render_widget(main_block, main);
        frame.render_widget(log_block, log);
        frame.render_widget(footer, footer_layout);
    }

    fn before_show(&mut self) -> std::io::Result<()> {
        if self.sslocal.is_none() {
            let yes_no = YesNoMessageBoxLayer::new("Info", "sslocal not found, download it?")
                .green()
                .on_gray()
                .show()?;
            if yes_no.result.is_yes() {
                self.sslocal_update()?;
            }
        }
        Ok(())
    }

    fn update(&mut self, event: Option<Event>) -> std::io::Result<()> {
        if let Some(event) = event {
            if let Event::Key(key_event) = event {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc => {
                            self.exit = YesNoMessageBoxLayer::new("Info", "exit?")
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
