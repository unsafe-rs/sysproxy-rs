use crate::{Error, Result, Sysproxy};
use std::{net::SocketAddr, str::FromStr};
use winreg::{enums, RegKey};

const SUB_KEY: &str = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Internet Settings";

impl Sysproxy {
    pub fn get_system_proxy() -> Result<Sysproxy> {
        let hkcu = RegKey::predef(enums::HKEY_CURRENT_USER);
        let cur_var = hkcu.open_subkey_with_flags(SUB_KEY, enums::KEY_READ)?;

        let enable = cur_var.get_value::<u32, _>("ProxyEnable")? == 1u32;
        let server = cur_var.get_value::<String, _>("ProxyServer")?;
        let server = server.as_str();

        let socket = SocketAddr::from_str(server).or(Err(Error::ParseStr))?;
        let host = socket.ip().to_string();
        let port = socket.port();

        let bypass = cur_var.get_value("ProxyOverride");

        Ok(Sysproxy {
            enable,
            host,
            socks_port: Some(port),
            bypass,
            ..Default::default()
        })
    }

    pub fn set_system_proxy(&self) -> Result<()> {
        let hkcu = RegKey::predef(enums::HKEY_CURRENT_USER);
        let cur_var = hkcu.open_subkey_with_flags(SUB_KEY, enums::KEY_SET_VALUE)?;

        let enable = if self.enable { 1u32 } else { 0u32 };
        cur_var.set_value("ProxyEnable", &enable)?;

        if let Some(port) = self.socks_port {
            let server = format!("{}:{}", self.host, port);
            cur_var.set_value("ProxyServer", &server)?;
        }

        if let Some(bypass) = self.bypass {
            cur_var.set_value("ProxyOverride", &bypass)?;
        }

        Ok(())
    }
}
