use serde::{Deserialize, Serialize};
use std::{env::current_exe, fs, io};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserData {
    pub sslocal_exec_path: String,
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
