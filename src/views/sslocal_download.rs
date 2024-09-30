use std::{
    sync::{atomic::AtomicUsize, mpsc::Receiver, Arc},
    thread::{sleep_ms, JoinHandle},
};

use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Gauge, Paragraph},
};

use crate::{
    sslocal::{Asset, SSLocal, SSLocalManager},
    Layer, Show,
};

use super::messagebox::MessageBoxLayer;

pub struct SSLocalDownloadLayer {
    exit: bool,
    latest: Arc<Asset>,
    downloaded_size: Arc<AtomicUsize>,
    task: Option<JoinHandle<anyhow::Result<()>>>,
    pub result: Option<SSLocal>,
}

impl SSLocalDownloadLayer {
    pub fn new(latest: Asset) -> Self {
        let latest = Arc::new(latest);
        let latest_cloned = latest.clone();
        let downloaded_size = Arc::new(AtomicUsize::new(0));
        let downloaded_size_cloned = downloaded_size.clone();
        let task = std::thread::spawn(move || {
            // let bytes = SSLocalManager::download_proxy(
            //     latest_cloned.as_ref(),
            //     |size| downloaded_size_cloned.store(size, std::sync::atomic::Ordering::Relaxed),
            //     "socks5://127.0.0.1:10808",
            // )?;
            let bytes = SSLocalManager::download(latest_cloned.as_ref(), |size| {
                downloaded_size_cloned.store(size, std::sync::atomic::Ordering::Relaxed)
            })?;
            if latest_cloned.name.ends_with(".zip") {
                SSLocalManager::extract_zip(&bytes)?
            } else if latest_cloned.name.ends_with(".tar.xz") {
                SSLocalManager::extract_tar_xz(&bytes)?
            }
            Ok(())
        });
        Self {
            exit: false,
            latest,
            downloaded_size,
            task: Some(task),
            result: None,
        }
    }
}

impl Layer for SSLocalDownloadLayer {
    fn view(&mut self, frame: &mut ratatui::Frame) {
        let [center] = Layout::vertical([Constraint::Length(5)])
            .flex(Flex::Center)
            .areas(frame.area());
        let [center] = Layout::horizontal([Constraint::Percentage(80)])
            .flex(Flex::Center)
            .areas(center);
        let bottom = Rect {
            x: center.x,
            y: center.bottom() - 1,
            width: center.width,
            height: 1,
        };

        let downloaded_size = self
            .downloaded_size
            .load(std::sync::atomic::Ordering::Relaxed) as f32
            / 1024.0
            / 1024.0;
        let total_size = self.latest.size as f32 / 1024.0 / 1024.0;
        let ratio = downloaded_size / total_size;

        let progress = Gauge::default()
            .block(Block::bordered().title(format!("Downloading {}", self.latest.name)))
            .label(format!("{:.2} MB/{:.2} MB", downloaded_size, total_size))
            .ratio(ratio as f64)
            .gauge_style(Style::default().fg(Color::Green))
            .use_unicode(true)
            .green()
            .on_gray();
        frame.render_widget(progress, center);

        let cancel = Paragraph::new("[Cancel]".white().on_blue()).centered();
        frame.render_widget(cancel, bottom);
    }

    fn before_show(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn update(&mut self, event: Option<Event>) -> std::io::Result<()> {
        if let Some(task) = &self.task {
            if task.is_finished() {
                if let Some(path) = SSLocalManager::find_ss_exec_path()? {
                    self.result = Some(SSLocal::new(path));
                    self.exit = true;
                } else {
                    match self.task.take().unwrap().join() {
                        Ok(ok) => {
                            if let Err(err) = ok {
                                MessageBoxLayer::new("Error", format!("{:?}", err))
                                    .red()
                                    .on_gray()
                                    .show()?;
                            }
                        }
                        Err(err) => {
                            MessageBoxLayer::new("Error", format!("{:?}", err))
                                .red()
                                .on_gray()
                                .show()?;
                        }
                    }
                    self.exit = true;
                }
            }
        }
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

    fn close(&mut self) {
        self.exit = true;
    }

    fn is_exit(&self) -> bool {
        self.exit
    }
}
