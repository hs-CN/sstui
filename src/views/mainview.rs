use std::{
    io::{BufRead, BufReader},
    process::Child,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
};

use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout, Margin},
    style::{palette::tailwind::*, Color, Style, Styled, Stylize},
    text::Line,
    widgets::{Block, HighlightSpacing, Paragraph, Row, Table, TableState, Tabs, Wrap},
};

use super::{
    messagebox::{
        CancelableMessageBoxLayer, CancelableMessageBoxResult, MessageBoxLayer,
        YesNoMessageBoxLayer,
    },
    sslocal_update::SSLocalUpdateLayer,
    ssserver_import::SSServerImportLayer,
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
    show_group_index: usize,
    row_styles: [Style; 2],
    table_state: TableState,
    selected_style: Style,
    sslocal: Option<SSLocal>,
    child: Option<Child>,
    child_stop: Arc<AtomicBool>,
    logs: Arc<RwLock<String>>,
}

impl MainLayer {
    pub fn new() -> Self {
        let userdata = UserData::load().unwrap_or_default();
        let row_styles = [
            Style::default().fg(WHITE).bg(GRAY.c950),
            Style::default().fg(WHITE).bg(GRAY.c900),
        ];
        let table_state = TableState::default().with_selected(0);
        let selected_style = Style::default().fg(BLACK).bg(INDIGO.c400);

        Self {
            exit: false,
            state: State::Tab,
            userdata,
            show_group_index: 0,
            row_styles,
            table_state,
            selected_style,
            sslocal: None,
            child: None,
            child_stop: Arc::new(AtomicBool::new(false)),
            logs: Arc::new(RwLock::new(String::new())),
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
    fn before_show(&mut self) -> std::io::Result<()> {
        // sslocal
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

        // update servers
        Ok(())
    }

    fn view(&mut self, frame: &mut ratatui::Frame) {
        let [header_layout, tabs_layout, main_layout, log_layout, footer_layout] =
            Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Percentage(70),
                Constraint::Max(15),
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

        let tabs: Vec<&str> = self
            .userdata
            .server_groups
            .iter()
            .map(|i| i.name.as_str())
            .collect();
        if !tabs.is_empty() {
            let mut current_group = false;
            let mut used_index = 0;
            if let Some((group_index, server_index)) = self.userdata.selected_server {
                if group_index == self.show_group_index {
                    current_group = true;
                    used_index = server_index;
                }
            }
            let selected_server_group = &self.userdata.server_groups[self.show_group_index];
            let server_port_str_vec: Vec<String> = selected_server_group
                .ss_servers
                .iter()
                .map(|i| i.server_port.to_string())
                .collect();
            let (name_len, server_len, port_len, method_len) = selected_server_group
                .ss_servers
                .iter()
                .enumerate()
                .fold((0, 0, 0, 0), |(n, s, p, m), (i, server)| {
                    (
                        n.max(server.remarks.len()),
                        s.max(server.server.len()),
                        p.max(server_port_str_vec[i].len()),
                        m.max(server.method.len()),
                    )
                });
            let tabs = Tabs::new(tabs)
                .select(self.show_group_index)
                .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow));
            frame.render_widget(tabs, tabs_layout);

            let header = Row::new(["Name", "Server", "Port", "Method"]).white();
            let rows = selected_server_group
                .ss_servers
                .iter()
                .enumerate()
                .map(|(i, server)| {
                    if current_group && i == used_index {
                        Row::new([
                            server.remarks.as_str(),
                            server.server.as_str(),
                            server_port_str_vec[i].as_str(),
                            server.method.as_str(),
                        ])
                        .white()
                        .on_green()
                    } else {
                        Row::new([
                            server.remarks.as_str(),
                            server.server.as_str(),
                            server_port_str_vec[i].as_str(),
                            server.method.as_str(),
                        ])
                        .set_style(self.row_styles[i % 2])
                    }
                });
            let table = Table::new(
                rows,
                [
                    Constraint::Length(name_len as u16 + 1),
                    Constraint::Length(server_len as u16 + 1),
                    Constraint::Length(port_len as u16 + 1),
                    Constraint::Length(method_len as u16 + 1),
                ],
            )
            .header(header)
            .highlight_style(self.selected_style)
            .highlight_spacing(HighlightSpacing::Always);

            let mut block = Block::bordered();
            if self.state == State::Tab {
                block = block.green();
            }
            frame.render_widget(block, main_layout);

            let main_layout_inner = main_layout.inner(Margin {
                vertical: 1,
                horizontal: 1,
            });
            frame.render_stateful_widget(table, main_layout_inner, &mut self.table_state);
        } else {
            let mut block = Block::bordered();
            let [_top, center, _bottom] = Layout::vertical([
                Constraint::Min(0),
                Constraint::Length(4),
                Constraint::Min(0),
            ])
            .areas(main_layout);
            let mut main = Paragraph::new(vec![
                "Empty server groups".into(),
                "Press 'a' to add server group".into(),
            ])
            .block(Block::bordered())
            .centered();
            if self.state == State::Tab {
                block = block.green();
                main = main.green();
            }
            frame.render_widget(block, main_layout);
            frame.render_widget(main, center);
        }

        let log = self.logs.read().unwrap();
        let mut log = Paragraph::new(log.as_str())
            .block(Block::bordered().title("Log"))
            .wrap(Wrap { trim: true });
        if self.state == State::Log {
            log = log.green();
        }
        frame.render_widget(log, log_layout);

        let op: &str = if let State::Tab = self.state {
            "Next (Tab) | ↑ | ↓ | ← | → | Update (u) | Add (a) | Del (Del) | Select (Enter) | Exit (Esc)"
        } else {
            "Next (Tab) | ↑ | ↓ | Select (Enter) | Configure (c) | Update SSLocal (u) | Exit (Esc)"
        };
        let footer = Paragraph::new(op).centered();
        frame.render_widget(footer, footer_layout);
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
                            if self.exit {
                                self.userdata.save()?;
                                self.child_stop.store(true, Ordering::Relaxed);
                                if let Some(child) = &mut self.child {
                                    child.kill().unwrap();
                                }
                            }
                        }
                        KeyCode::Tab => {
                            self.state = match self.state {
                                State::Tab => State::Log,
                                State::Log => State::Tab,
                            };
                        }
                        KeyCode::Left => {
                            if let State::Tab = self.state {
                                let before = self.show_group_index;
                                self.show_group_index = self.show_group_index.saturating_sub(1);
                                if before != self.show_group_index {
                                    self.table_state.select(Some(0));
                                }
                            }
                        }
                        KeyCode::Right => {
                            if let State::Tab = self.state {
                                let before = self.show_group_index;
                                self.show_group_index = (self.show_group_index + 1)
                                    .min(self.userdata.server_groups.len() - 1);
                                if before != self.show_group_index {
                                    self.table_state.select(Some(0));
                                }
                            }
                        }
                        KeyCode::Up => {
                            self.table_state.select_previous();
                        }
                        KeyCode::Down => {
                            self.table_state.select_next();
                        }
                        KeyCode::Char('a') => {
                            if let State::Tab = self.state {
                                let import = SSServerImportLayer::new().show()?;
                                if let Some(group) = import.result {
                                    self.userdata.server_groups.push(group);
                                    self.userdata.save()?;
                                }
                            }
                        }
                        KeyCode::Delete => {
                            if let State::Tab = self.state {
                                if self.show_group_index < self.userdata.server_groups.len() {
                                    let yes_no = YesNoMessageBoxLayer::new(
                                        "Info",
                                        format!(
                                            "delete group '{}' ?",
                                            self.userdata.server_groups[self.show_group_index].name
                                        ),
                                    )
                                    .red()
                                    .on_gray()
                                    .show()?;
                                    if yes_no.result.is_yes() {
                                        self.userdata.server_groups.remove(self.show_group_index);
                                        self.show_group_index =
                                            self.show_group_index.saturating_sub(1);
                                        self.userdata.save()?;
                                    }
                                }
                            }
                        }
                        KeyCode::Char('u') => match self.state {
                            State::Tab => {
                                if self.show_group_index < self.userdata.server_groups.len() {
                                    if let Err(err) =
                                        self.userdata.server_groups[self.show_group_index].update()
                                    {
                                        MessageBoxLayer::new("Error", err.to_string())
                                            .red()
                                            .on_gray()
                                            .show()?;
                                    }
                                }
                            }
                            State::Log => self.sslocal_update()?,
                        },
                        KeyCode::Char('c') => {}
                        KeyCode::Enter => match self.state {
                            State::Tab => {
                                if self.show_group_index < self.userdata.server_groups.len() {
                                    if let Some(i) = self.table_state.selected() {
                                        let selected_server = &self.userdata.server_groups
                                            [self.show_group_index]
                                            .ss_servers[i];
                                        if let Some(sslocal) = &self.sslocal {
                                            self.child_stop.store(true, Ordering::Relaxed);
                                            if let Some(child) = &mut self.child {
                                                child.kill().unwrap();
                                            }

                                            match sslocal.run(
                                                selected_server,
                                                self.userdata.local_port,
                                                self.userdata.lan_support,
                                            ) {
                                                Ok(mut child) => {
                                                    if child.stdout.is_some() {
                                                        let mut reader = BufReader::new(
                                                            child.stdout.take().unwrap(),
                                                        );
                                                        self.child_stop
                                                            .store(false, Ordering::Relaxed);
                                                        let child_stop = self.child_stop.clone();
                                                        let logs = self.logs.clone();
                                                        std::thread::spawn(move || loop {
                                                            if child_stop.load(Ordering::Relaxed) {
                                                                break;
                                                            }
                                                            let mut buf = String::new();
                                                            if let Ok(n) =
                                                                reader.read_line(&mut buf)
                                                            {
                                                                if n > 0 {
                                                                    *logs.write().unwrap() = buf;
                                                                }
                                                            }
                                                        });
                                                    }
                                                    self.child = Some(child);
                                                    self.userdata.selected_server =
                                                        Some((self.show_group_index, i))
                                                }
                                                Err(err) => {
                                                    MessageBoxLayer::new("Error", err.to_string())
                                                        .red()
                                                        .on_gray()
                                                        .show()?;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            State::Log => {}
                        },
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
