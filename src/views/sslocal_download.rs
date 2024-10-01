use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread::JoinHandle,
    time::Duration,
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
    asset: Arc<Asset>,
    cancel_token: Arc<AtomicBool>,
    downloaded_size: Arc<AtomicUsize>,
    download_task: Option<JoinHandle<anyhow::Result<()>>>,
    extract_task: Option<JoinHandle<anyhow::Result<()>>>,
    pub result: Option<SSLocal>,
}

impl SSLocalDownloadLayer {
    pub fn new(asset: Asset) -> Self {
        let asset = Arc::new(asset);
        let cancel_token = Arc::new(AtomicBool::new(false));
        let downloaded_size = Arc::new(AtomicUsize::new(0));

        let (tx, rx) = std::sync::mpsc::channel();

        let asset_cloned = asset.clone();
        let download_task = std::thread::spawn(move || {
            SSLocalManager::download(&asset_cloned.browser_download_url, tx)
        });

        let asset_cloned = asset.clone();
        let cancel_token_cloned = cancel_token.clone();
        let downloaded_size_cloned = downloaded_size.clone();
        let extract_task = std::thread::spawn(move || {
            let mut bytes = Vec::with_capacity(asset_cloned.size);
            loop {
                if cancel_token_cloned.load(Ordering::Relaxed) {
                    break;
                }
                if bytes.len() == asset_cloned.size {
                    break;
                }
                if let Ok(vec_u8) = rx.recv_timeout(Duration::from_millis(100)) {
                    downloaded_size_cloned.fetch_add(vec_u8.len(), Ordering::Relaxed);
                    bytes.extend_from_slice(&vec_u8);
                }
            }
            if bytes.len() == asset_cloned.size {
                if asset_cloned.name.ends_with(".zip") {
                    SSLocalManager::extract_zip(&bytes)?;
                } else if asset_cloned.name.ends_with(".tar.xz") {
                    SSLocalManager::extract_tar_xz(&bytes)?;
                }
            }
            Ok(())
        });

        Self {
            exit: false,
            asset,
            cancel_token,
            downloaded_size,
            download_task: Some(download_task),
            extract_task: Some(extract_task),
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
            .load(std::sync::atomic::Ordering::Relaxed);
        let percent = downloaded_size / self.asset.size;
        let downloaded_size = downloaded_size as f32 / 1024.0 / 1024.0;
        let total_size = self.asset.size as f32 / 1024.0 / 1024.0;

        let progress = Gauge::default()
            .block(Block::bordered().title(format!("Downloading '{}' ", self.asset.name)))
            .label(format!("{:.2} MB / {:.2} MB", downloaded_size, total_size))
            .percent(percent as u16)
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
        if self.download_task.as_ref().unwrap().is_finished()
            && self.extract_task.as_ref().unwrap().is_finished()
        {
            if let Some(path) = SSLocalManager::find_ss_exec_path()? {
                self.result = Some(SSLocal::new(path));
            } else {
                match self.download_task.take().unwrap().join() {
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
                match self.extract_task.take().unwrap().join() {
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
            }
            self.exit = true;
        }

        if !self.exit {
            if let Some(event) = event {
                if let Event::Key(key_event) = event {
                    if key_event.kind == KeyEventKind::Press {
                        match key_event.code {
                            KeyCode::Enter => {
                                self.cancel_token.store(true, Ordering::Relaxed);
                                self.exit = true;
                            }
                            KeyCode::Esc => {
                                self.cancel_token.store(true, Ordering::Relaxed);
                                self.exit = true;
                            }
                            _ => {}
                        }
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
