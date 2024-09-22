use std::{
    env::current_exe,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use serde::Deserialize;

pub struct SSLocal {
    exec_path: PathBuf,
}

impl SSLocal {
    pub fn new<P: AsRef<Path>>(exec_path: P) -> Self {
        Self {
            exec_path: exec_path.as_ref().to_path_buf(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LatestRelease {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
pub struct Asset {
    pub name: String,
    pub size: usize,
    pub browser_download_url: String,
}

const CHECK_URL: &'static str =
    "https://api.github.com/repos/shadowsocks/shadowsocks-rust/releases/latest";

pub struct SSLocalManager;
impl SSLocalManager {
    pub fn get_latest() -> anyhow::Result<LatestRelease> {
        let mut latest_release: LatestRelease = ureq::get(CHECK_URL).call()?.into_json()?;
        latest_release.assets = latest_release
            .assets
            .into_iter()
            .filter(|asset| !asset.name.ends_with(".sha256"))
            .collect();
        Ok(latest_release)
    }

    pub fn download<F: Fn(usize), S: AsRef<str>>(
        latest: &Asset,
        proxy: Option<S>,
        f: Option<F>,
    ) -> anyhow::Result<()> {
        let mut agent_builder = ureq::AgentBuilder::new();
        if let Some(proxy_url) = proxy {
            let proxy = ureq::Proxy::new(proxy_url)?;
            agent_builder = agent_builder.proxy(proxy);
        }
        let agent = agent_builder.build();
        let response = agent.get(&latest.browser_download_url).call()?;
        // download to memory
        let mut bytes_reader = response.into_reader();
        let mut bytes: Vec<u8> = Vec::with_capacity(latest.size);
        let mut buf = [0u8; 4096];
        loop {
            let n = bytes_reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            bytes.extend_from_slice(&buf[..n]);
            if let Some(f) = f.as_ref() {
                f(bytes.len());
            }
        }
        // write to file
        let mut file_path = current_exe()?;
        file_path.set_file_name(latest.name.to_owned());
        let mut file = fs::File::create(file_path)?;
        file.write_all(&bytes)?;
        Ok(())
    }
}
