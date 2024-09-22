use super::Layer;

pub struct SSLocalManagerLayer {
    exit: bool,
    next_layer: Option<Box<dyn Layer>>,
}

impl SSLocalManagerLayer {
    pub fn new() -> Self {
        Self {
            exit: false,
            next_layer: None,
        }
    }
}

impl Layer for SSLocalManagerLayer {
    fn view(&mut self, frame: &mut ratatui::Frame) {
        todo!()
    }

    fn update(&mut self, event: ratatui::crossterm::event::Event) {
        todo!()
    }

    fn next_layer(&mut self) -> Option<Box<dyn Layer>> {
        self.next_layer.take()
    }

    fn exit(&self) -> bool {
        self.exit
    }
}
