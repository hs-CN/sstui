use sstui::{AppBuilder, MainLayer};
use std::io;

fn main() -> io::Result<()> {
    let main_layer = MainLayer::new()?;
    let result = AppBuilder::default().show(main_layer).run();
    ratatui::restore();
    result
}
