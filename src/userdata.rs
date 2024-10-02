use base64::prelude::*;
use serde::{Deserialize, Serialize};
use std::{env::current_exe, fs, io};

#[derive(Serialize, Deserialize)]
pub struct UserData {
    pub local_port: u16,
    pub lan_support: bool,
    pub selected_server: Option<(usize, usize)>,
    pub server_groups: Vec<ServerGroup>,
}

impl Default for UserData {
    fn default() -> Self {
        Self {
            local_port: 10808,
            lan_support: false,
            selected_server: None,
            server_groups: Vec::new(),
        }
    }
}

impl UserData {
    pub fn load() -> io::Result<Self> {
        let mut file_path = current_exe()?;
        file_path.set_file_name("userdata");
        let content = fs::read(file_path)?;
        Ok(serde_json::from_slice(&content)?)
    }

    pub fn save(&self) -> io::Result<()> {
        let mut file_path = current_exe()?;
        file_path.set_file_name("userdata");
        let content = serde_json::to_vec(&self)?;
        fs::write(file_path, content)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerGroup {
    pub name: String,
    pub update_url: String,
    pub update_type: Option<ServerUpdateType>,
    pub ss_servers: Vec<SSServer>,
}

impl ServerGroup {
    pub fn new<S: Into<String>>(name: S, update_url: S) -> Self {
        Self {
            name: name.into(),
            update_url: update_url.into(),
            update_type: None,
            ss_servers: Vec::new(),
        }
    }

    fn type_check(&mut self, content: &str) -> anyhow::Result<()> {
        // ssjson
        if content.starts_with(['[', '{']) && content.ends_with([']', '}']) {
            self.update_type = Some(ServerUpdateType::SSJson);
            return Ok(());
        }

        // base64 + ssurl
        if let Ok(bytes) = BASE64_STANDARD.decode(&content) {
            let content = String::from_utf8_lossy(&bytes);
            if content.starts_with("ss://") {
                self.update_type = Some(ServerUpdateType::SSUrl);
                return Ok(());
            }
        }

        anyhow::bail!("unknown content type");
    }

    fn _update(&mut self, agent: ureq::Agent) -> anyhow::Result<()> {
        let content = agent.get(&self.update_url).call()?.into_string()?;

        if self.update_type.is_none() {
            self.type_check(&content)?;
        }

        match self.update_type.as_ref().unwrap() {
            ServerUpdateType::SSJson => self.ss_servers = serde_json::from_str(&content)?,
            ServerUpdateType::SSUrl => {
                let bytes = BASE64_STANDARD.decode(&content)?;
                let content = String::from_utf8_lossy(&bytes);
                let mut ss_servers = Vec::new();
                for line in content.lines() {
                    if let Ok(ss_server) = SSServer::from_ssurl_str(line) {
                        ss_servers.push(ss_server);
                    }
                }
                self.ss_servers = ss_servers;
            }
        }
        Ok(())
    }

    pub fn update(&mut self) -> anyhow::Result<()> {
        let agent = ureq::AgentBuilder::new().build();
        self._update(agent)
    }

    pub fn update_proxy<P: AsRef<str>>(&mut self, proxy: P) -> anyhow::Result<()> {
        let proxy = ureq::Proxy::new(proxy).unwrap();
        let agent = ureq::AgentBuilder::new().proxy(proxy).build();
        self._update(agent)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerUpdateType {
    SSJson,
    SSUrl,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SSServer {
    pub remarks: String,
    pub server: String,
    pub server_port: u16,
    pub method: String,
    pub password: String,
}

impl SSServer {
    fn from_ssurl_str(ssurl: &str) -> anyhow::Result<Self> {
        if ssurl.starts_with("ss://") {
            let url = urlencoding::decode(ssurl)?;
            let i = url.find('#').unwrap_or(url.len());
            let bytes = BASE64_STANDARD_NO_PAD.decode(&url[5..i])?;
            let content = String::from_utf8_lossy(&bytes);
            let parts: Vec<_> = content.split([':', '@']).collect();
            let remarks = if i < url.len() {
                url[i + 1..].to_string()
            } else {
                String::new()
            };
            if parts.len() == 4 {
                let method = parts[0].to_string();
                let password = parts[1].to_string();
                let server = parts[2].to_string();
                let server_port = parts[3].parse()?;
                return Ok(SSServer {
                    remarks,
                    server,
                    server_port,
                    method,
                    password,
                });
            }
        }
        anyhow::bail!("invalid ssurl: {}", ssurl);
    }
}
