use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout},
    style::Stylize,
    text::Line,
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
    Layer,
};

#[derive(PartialEq)]
enum State {
    Tab,
    Log,
}

pub struct MainLayer {
    exit: bool,
    state: State,
    userdata: UserData,
    sslocal: Option<SSLocal>,
}

impl MainLayer {
    pub fn new() -> Self {
        let userdata = UserData::load().unwrap_or_default();

        Self {
            exit: false,
            state: State::Tab,
            userdata,
            sslocal: None,
        }
    }

    fn sslocal_update(&mut self) -> std::io::Result<()> {
        let cancelable = CancelableMessageBoxLayer::new(
            "Info",
            "get latest version...",
            std::thread::spawn(SSLocalManager::get_latest),
        )
        .green()
        .on_gray()
        .show()?;
        if let CancelableMessageBoxResult::Complete(result) = cancelable.result {
            match result {
                Ok(latest) => {
                    let yes_no = YesNoMessageBoxLayer::new(
                        "Info",
                        Line::from(vec![
                            "latest version:".into(),
                            format!(" {} ", latest.tag_name).white().on_red(),
                            ", download it?".into(),
                        ]),
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
                    MessageBoxLayer::new("Error", err.to_string())
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
        let [header_layout, tab_layout, log_layout, op_layout, footer_layout] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Percentage(70),
            Constraint::Max(15),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .flex(Flex::Legacy)
        .areas(frame.area());
        let header = if let Some(sslocal) = &self.sslocal {
            Paragraph::new(format!("Version: {}", sslocal.version))
        } else {
            Paragraph::new("Version: None")
        };
        frame.render_widget(header, header_layout);

        let mut tab = Block::bordered().title("shadowsocks servers");
        if self.state == State::Tab {
            tab = tab.green();
        }
        frame.render_widget(tab, tab_layout);

        let mut log = Block::bordered().title("Log");
        if self.state == State::Log {
            log = log.green();
        }
        frame.render_widget(log, log_layout);

        let op = Paragraph::new("Op:").centered();
        frame.render_widget(op, op_layout);

        let footer = Paragraph::new("Next (Tab) | Up (↑) | Down (↓) | Select (Enter) | Exit (Esc)")
            .centered();
        frame.render_widget(footer, footer_layout);
    }

    fn before_show(&mut self) -> std::io::Result<()> {
        self.sslocal = SSLocalManager::find_sslocal()?;
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
                        KeyCode::Tab => {
                            self.state = match self.state {
                                State::Tab => State::Log,
                                State::Log => State::Tab,
                            };
                        }
                        KeyCode::Char('u') => {}
                        KeyCode::Char('U') => self.sslocal_update()?,
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
