use std::io::{self, Write};

use sstui::SSLocalManager;

fn main() {
    let manager = SSLocalManager::get_latest().unwrap();
    for asset in manager.assets {
        if asset.name.contains("windows-msvc") {
            SSLocalManager::download(
                &asset,
                Some("socks5://127.0.0.1:10808"),
                Some(|size| {
                    let percent = size * 100 / asset.size;
                    print!(
                        "\rdownload {} [{}%] {}/{}",
                        asset.name, percent, size, asset.size
                    );
                    io::stdout().flush().unwrap();
                }),
            )
            .unwrap()
        }
    }
}
