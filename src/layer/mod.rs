mod main_layer;
pub use main_layer::MainLayer;
mod sslocal_manager_layer;
pub use sslocal_manager_layer::SSLocalManagerLayer;

pub trait Layer {
    fn view(&mut self, frame: &mut ratatui::Frame);
    fn update(&mut self, event: ratatui::crossterm::event::Event);
    fn next_layer(&mut self) -> Option<Box<dyn Layer>>;
    fn exit(&self) -> bool;
}

pub struct AppBuilder {
    terminal: ratatui::DefaultTerminal,
    layer_stack: Vec<Box<dyn Layer>>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new(ratatui::init())
    }
}

impl AppBuilder {
    pub fn new(terminal: ratatui::DefaultTerminal) -> Self {
        Self {
            terminal,
            layer_stack: Vec::new(),
        }
    }

    pub fn show(mut self, layer: impl Layer + 'static) -> App {
        self.layer_stack.push(Box::new(layer));
        App {
            terminal: self.terminal,
            layer_stack: self.layer_stack,
        }
    }
}

pub struct App {
    terminal: ratatui::DefaultTerminal,
    layer_stack: Vec<Box<dyn Layer>>,
}

impl App {
    pub fn run(&mut self) -> std::io::Result<()> {
        while let Some(layer) = self.layer_stack.last_mut() {
            self.terminal.draw(|frame| layer.view(frame))?;
            layer.update(ratatui::crossterm::event::read()?);
            let next_layer = layer.next_layer();
            if layer.exit() {
                self.layer_stack.pop();
            }
            if let Some(next_layer) = next_layer {
                self.layer_stack.push(next_layer);
            }
        }
        Ok(())
    }
}
