mod main;
mod messagebox;

pub use main::MainLayer;

use ratatui::crossterm::event::{poll, read, Event};
use std::{
    sync::{Arc, OnceLock, RwLock},
    time::Duration,
};

static TERMINAL: OnceLock<RwLock<ratatui::DefaultTerminal>> = OnceLock::new();

pub trait Layer {
    fn view(&mut self, frame: &mut ratatui::Frame);
    fn before_show(&mut self) -> std::io::Result<()>;
    fn update(&mut self, event: Option<Event>) -> std::io::Result<()>;
    fn close(&mut self);
    fn is_exit(&self) -> bool;
    fn thread_safe(self) -> Arc<RwLock<Self>>
    where
        Self: Sized,
    {
        Arc::new(RwLock::new(self))
    }
}

pub trait Show {
    fn show(self) -> std::io::Result<Self>
    where
        Self: Sized;
}

impl<L: Layer> Show for L {
    fn show(mut self) -> std::io::Result<Self> {
        self.before_show()?;
        while !self.is_exit() {
            TERMINAL
                .get()
                .unwrap()
                .write()
                .unwrap()
                .draw(|frame| self.view(frame))?;
            if poll(Duration::from_millis(10))? {
                self.update(Some(read()?))?;
            } else {
                self.update(None)?;
            }
        }
        Ok(self)
    }
}

impl<L: Layer> Show for Arc<RwLock<L>> {
    fn show(self) -> std::io::Result<Self> {
        self.write().unwrap().before_show()?;
        while !self.read().unwrap().is_exit() {
            TERMINAL
                .get()
                .unwrap()
                .write()
                .unwrap()
                .draw(|frame| self.write().unwrap().view(frame))?;
            if poll(Duration::from_millis(10))? {
                self.write().unwrap().update(Some(read()?))?;
            } else {
                self.write().unwrap().update(None)?;
            }
        }
        Ok(self)
    }
}

pub fn terminal_init(terminal: ratatui::DefaultTerminal) {
    TERMINAL.set(RwLock::new(terminal)).unwrap();
}

pub fn terminal_init_default() {
    terminal_init(ratatui::init());
}
