use std::{
    env::current_exe,
    io::Cursor,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::mpsc::Sender,
};

use serde::Deserialize;
use xz2::read::XzDecoder;
use zip::ZipArchive;

use crate::userdata::SSServer;

pub struct SSLocal {
    exec_path: PathBuf,
    pub version: String,
}

impl SSLocal {
    pub fn new(exec_path: PathBuf) -> std::io::Result<Self> {
        let output = Command::new(&exec_path).arg("--version").output()?;
        let version = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(Self { exec_path, version })
    }

    pub fn run(
        &self,
        server: &SSServer,
        local_port: u16,
        lan_support: bool,
    ) -> std::io::Result<Child> {
        let local_addr = if lan_support {
            format!("0.0.0.0:{}", local_port)
        } else {
            format!("127.0.0.1:{}", local_port)
        };
        Command::new(&self.exec_path)
            .stdout(Stdio::piped())
            .arg("-b")
            .arg(local_addr)
            .arg("-s")
            .arg(format!("{}:{}", &server.server, &server.server_port))
            .arg("-m")
            .arg(&server.method)
            .arg("-k")
            .arg(&server.password)
            .arg("-U")
            .arg("-v")
            .spawn()
    }
}

#[derive(Debug, Deserialize)]
pub struct LatestRelease {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Asset {
    pub name: String,
    pub size: usize,
    pub browser_download_url: String,
}

pub struct SSLocalManager;

impl SSLocalManager {
    const CHECK_URL: &'static str =
        "https://api.github.com/repos/shadowsocks/shadowsocks-rust/releases/latest";

    pub fn find_sslocal() -> std::io::Result<Option<SSLocal>> {
        let mut dir = current_exe()?;
        dir.set_file_name("ss");
        for entry in dir.read_dir()? {
            let path = entry?;
            if path.file_type()?.is_file() && path.file_name().to_string_lossy().contains("sslocal")
            {
                let sslocal = SSLocal::new(path.path())?;
                return Ok(Some(sslocal));
            }
        }
        Ok(None)
    }

    fn _get_latest(agent: ureq::Agent) -> anyhow::Result<LatestRelease> {
        let mut latest_release: LatestRelease = agent.get(Self::CHECK_URL).call()?.into_json()?;
        latest_release
            .assets
            .retain(|asset| !asset.name.ends_with(".sha256"));
        Ok(latest_release)
    }

    pub fn get_latest() -> anyhow::Result<LatestRelease> {
        let agent = ureq::AgentBuilder::new().build();
        Self::_get_latest(agent)
    }

    pub fn get_latest_proxy<P: AsRef<str>>(proxy: P) -> anyhow::Result<LatestRelease> {
        let proxy = ureq::Proxy::new(proxy)?;
        let agent = ureq::AgentBuilder::new().proxy(proxy).build();
        Self::_get_latest(agent)
    }

    fn _download(agent: ureq::Agent, url: &str, tx: Sender<Vec<u8>>) -> anyhow::Result<()> {
        let response = agent.get(url).call()?;
        let mut bytes_reader = response.into_reader();
        let mut buf = [0u8; 4096];
        loop {
            let n = bytes_reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            if tx.send(buf[..n].to_vec()).is_err() {
                break;
            }
        }
        Ok(())
    }

    pub fn download(url: &str, tx: Sender<Vec<u8>>) -> anyhow::Result<()> {
        let agent = ureq::AgentBuilder::new().build();
        Self::_download(agent, url, tx)
    }

    pub fn download_proxy<P: AsRef<str>>(
        url: &str,
        tx: Sender<Vec<u8>>,
        proxy: P,
    ) -> anyhow::Result<()> {
        let proxy = ureq::Proxy::new(proxy)?;
        let agent = ureq::AgentBuilder::new().proxy(proxy).build();
        Self::_download(agent, url, tx)
    }

    pub fn extract_zip(bytes: &[u8]) -> zip::result::ZipResult<()> {
        let mut zip = ZipArchive::new(Cursor::new(bytes))?;
        let mut dir = current_exe()?;
        dir.set_file_name("ss");
        zip.extract(dir)?;
        Ok(())
    }

    pub fn extract_tar_xz(bytes: &[u8]) -> zip::result::ZipResult<()> {
        let xz = XzDecoder::new(Cursor::new(bytes));
        let mut tar = tar::Archive::new(xz);
        let mut dir = current_exe()?;
        dir.set_file_name("ss");
        tar.unpack(dir)?;
        Ok(())
    }
}
