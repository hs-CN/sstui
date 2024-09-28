use std::sync::{OnceLock, RwLock};

mod main_layer;
mod messagebox_layer;

pub use main_layer::MainLayer;

static TERMINAL: OnceLock<RwLock<ratatui::DefaultTerminal>> = OnceLock::new();

pub trait Layer {
    fn view(&mut self, frame: &mut ratatui::Frame);
    fn update(&mut self, event: ratatui::crossterm::event::Event);
    fn is_exit(&self) -> bool;
    fn close(&mut self);
    fn show(mut self) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        while !self.is_exit() {
            TERMINAL
                .get()
                .unwrap()
                .write()
                .unwrap()
                .draw(|frame| self.view(frame))?;
            self.update(ratatui::crossterm::event::read()?);
        }
        Ok(self)
    }
}

pub fn terminal_init(
    terminal: ratatui::DefaultTerminal,
) -> Result<(), RwLock<ratatui::DefaultTerminal>> {
    TERMINAL.set(RwLock::new(terminal))
}

pub fn terminal_init_default() -> Result<(), RwLock<ratatui::DefaultTerminal>> {
    terminal_init(ratatui::init())
}
