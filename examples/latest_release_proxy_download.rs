use std::io::{self, Write};

use sstui::SSLocalManager;

fn main() {
    let latest = SSLocalManager::get_latest().unwrap();
    println!("get latest release: {:?}", latest.tag_name);
    for ref asset in latest.assets {
        if asset.name.contains("x86_64-pc-windows-msvc") {
            // let file_bytes = SSLocalManager::download(
            let file_bytes = SSLocalManager::download_proxy(
                &asset,
                |size| {
                    let percent = size * 100 / asset.size;
                    print!(
                        "\rdownload {} [{}%] {}/{}",
                        asset.name, percent, size, asset.size
                    );
                    io::stdout().flush().unwrap();
                },
                "socks5://127.0.0.1:10808",
            )
            .unwrap();
            SSLocalManager::extract_zip(file_bytes).unwrap();
            println!("\nextract {} completed", asset.name);
        }

        if asset.name.contains("x86_64-unknown-linux-gnu") {
            // let file_bytes = SSLocalManager::download(
            let file_bytes = SSLocalManager::download_proxy(
                &asset,
                |size| {
                    let percent = size * 100 / asset.size;
                    print!(
                        "\rdownload {} [{}%] {}/{}",
                        asset.name, percent, size, asset.size
                    );
                    io::stdout().flush().unwrap();
                },
                "socks5://127.0.0.1:10808",
            )
            .unwrap();
            SSLocalManager::extract_tar_xz(file_bytes).unwrap();
            println!("\nextract {} completed", asset.name)
        }
    }
}
