use ratatui::crossterm::event::{poll, read, Event};
use std::{
    sync::{OnceLock, RwLock},
    time::Duration,
};

static TERMINAL: OnceLock<RwLock<ratatui::DefaultTerminal>> = OnceLock::new();

pub trait Layer {
    fn before_show(&mut self) -> std::io::Result<()>;
    fn view(&mut self, frame: &mut ratatui::Frame);
    fn update(&mut self, event: Option<Event>) -> std::io::Result<()>;
    fn close(&mut self);
    fn is_exit(&self) -> bool;
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
                self.update(Some(read()?))?;
            } else {
                self.update(None)?;
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
