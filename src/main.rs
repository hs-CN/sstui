use sstui::{terminal_init_default, Layer, MainLayer};

fn main() {
    terminal_init_default().unwrap();
    let _ = MainLayer::new().show();
    ratatui::restore();
}
