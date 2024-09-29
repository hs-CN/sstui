mod main_layer;
mod messagebox_layer;

pub use main_layer::MainLayer;

use ratatui::crossterm::event::{poll, read, Event};
use std::{
    ops::Deref,
    sync::{Arc, OnceLock, RwLock},
    time::Duration,
};

static TERMINAL: OnceLock<RwLock<ratatui::DefaultTerminal>> = OnceLock::new();

pub trait Layer {
    fn view(&mut self, frame: &mut ratatui::Frame);
    fn before_show(&mut self) -> std::io::Result<()>;
    fn update(&mut self, event: Event) -> std::io::Result<()>;
    fn close(&mut self);
    fn is_exit(&self) -> bool;
    fn thread_safe(self) -> ThreadSafeLayer<Self>
    where
        Self: Sized,
    {
        ThreadSafeLayer(Arc::new(RwLock::new(self)))
    }
    fn show(mut self) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        self.before_show()?;
        while !self.is_exit() {
            TERMINAL
                .get()
                .unwrap()
                .write()
                .unwrap()
                .draw(|frame| self.view(frame))?;
            if poll(Duration::from_millis(10))? {
                self.update(read()?)?;
            }
        }
        Ok(self)
    }
}

pub struct ThreadSafeLayer<L: Layer>(Arc<RwLock<L>>);

impl<L: Layer> ThreadSafeLayer<L> {
    pub fn show(self) -> std::io::Result<Self> {
        self.0.write().unwrap().before_show()?;
        while !self.0.read().unwrap().is_exit() {
            TERMINAL
                .get()
                .unwrap()
                .write()
                .unwrap()
                .draw(|frame| self.0.write().unwrap().view(frame))?;
            if poll(Duration::from_millis(10))? {
                self.0.write().unwrap().update(read()?)?;
            }
        }
        Ok(self)
    }
}

impl<L: Layer> Deref for ThreadSafeLayer<L> {
    type Target = Arc<RwLock<L>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<L: Layer> Clone for ThreadSafeLayer<L> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub fn terminal_init(terminal: ratatui::DefaultTerminal) {
    TERMINAL.set(RwLock::new(terminal)).unwrap();
}

pub fn terminal_init_default() {
    terminal_init(ratatui::init());
}
