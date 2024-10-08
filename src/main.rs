use sstui::{terminal_init_default, Layer, MainLayer};

fn main() {
    terminal_init_default();
    let result = MainLayer::new().show();
    ratatui::restore();
    result.unwrap();
}
