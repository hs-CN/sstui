use sstui::{terminal_init_default, MainLayer, Show};

fn main() {
    terminal_init_default();
    let result = MainLayer::new().show();
    ratatui::restore();
    result.unwrap();
}
