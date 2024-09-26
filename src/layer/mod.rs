mod main_layer;
use std::sync::{Arc, RwLock};

pub use main_layer::MainLayer;

mod messagebox_layer;

static LAYER_STACK: RwLock<Vec<Arc<RwLock<dyn Layer>>>> = RwLock::new(Vec::new());

pub trait Layer: Send + Sync {
    fn view(&mut self, frame: &mut ratatui::Frame);
    fn update(&mut self, event: ratatui::crossterm::event::Event);
    fn is_transparent(&self) -> bool;
    fn exit(&self) -> bool;
}

pub struct AppBuilder(ratatui::DefaultTerminal);

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new(ratatui::init())
    }
}

impl AppBuilder {
    pub fn new(terminal: ratatui::DefaultTerminal) -> Self {
        Self(terminal)
    }

    pub fn show(self, layer: impl Layer + 'static) -> Self {
        show(layer);
        self
    }

    pub fn run(self) -> std::io::Result<()> {
        let mut terminal = self.0;
        loop {
            let top_layer = {
                let stack = LAYER_STACK.read().unwrap();
                if stack.is_empty() {
                    break;
                }
                let end_index = stack.len() - 1;
                let mut start_index = end_index;
                while start_index > 0 && stack[start_index].read().unwrap().is_transparent() {
                    start_index -= 1;
                }
                terminal.draw(|frame| {
                    for i in start_index..=end_index {
                        stack[i].write().unwrap().view(frame);
                    }
                })?;
                LAYER_STACK.read().unwrap()[end_index].clone()
            };

            top_layer
                .write()
                .unwrap()
                .update(ratatui::crossterm::event::read()?);

            if top_layer.read().unwrap().exit() {
                LAYER_STACK.write().unwrap().pop();
            }
        }
        Ok(())
    }
}

pub fn show(layer: impl Layer + 'static) -> Arc<RwLock<dyn Layer>> {
    let layer = Arc::new(RwLock::new(layer));
    LAYER_STACK.write().unwrap().push(layer.clone());
    layer
}
