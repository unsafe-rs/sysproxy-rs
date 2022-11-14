use crate::{Error, Result, Sysproxy};
use std::{process::Command, str::from_utf8};

const CMD_KEY: &str = "org.gnome.system.proxy";

impl Sysproxy {
    pub fn get_system_proxy() -> Result<Sysproxy> {
        let enable = Sysproxy::get_enable()?;

        let mut socks = get_proxy("socks")?;
        let https = get_proxy("https")?;
        let http = get_proxy("http")?;

        socks.http_port = http.http_port;
        socks.https_port = https.https_port;

        socks.enable = enable;
        socks.bypass = match Sysproxy::get_bypass() {
            Ok(v) => Some(v),
            Err(_) => None,
        };

        Ok(socks)
    }

    pub fn set_system_proxy(&self) -> Result<()> {
        self.set_enable()?;

        if self.enable {
            self.set_socks()?;
            self.set_https()?;
            self.set_http()?;
            self.set_bypass()?;
        }

        Ok(())
    }

    pub fn get_enable() -> Result<bool> {
        let mode = gsettings().args(["get", CMD_KEY, "mode"]).output()?;
        let mode = from_utf8(&mode.stdout).or(Err(Error::ParseStr))?.trim();
        Ok(mode == "'manual'")
    }

    pub fn get_bypass() -> Result<String> {
        let bypass = gsettings()
            .args(["get", CMD_KEY, "ignore-hosts"])
            .output()?;
        let bypass = from_utf8(&bypass.stdout).or(Err(Error::ParseStr))?.trim();

        let bypass = bypass.strip_prefix('[').unwrap_or(bypass);
        let bypass = bypass.strip_suffix(']').unwrap_or(bypass);

        let bypass = bypass
            .split(',')
            .map(|h| strip_str(h.trim()))
            .collect::<Vec<&str>>()
            .join(",");

        Ok(bypass)
    }

    pub fn get_http() -> Result<Sysproxy> {
        get_proxy("http")
    }

    pub fn get_https() -> Result<Sysproxy> {
        get_proxy("https")
    }

    pub fn get_socks() -> Result<Sysproxy> {
        get_proxy("socks")
    }

    pub fn set_enable(&self) -> Result<()> {
        let mode = if self.enable { "'manual'" } else { "'none'" };
        gsettings().args(["set", CMD_KEY, "mode", mode]).status()?;
        Ok(())
    }

    pub fn set_bypass(&self) -> Result<()> {
        if let Some(bypass) = self.bypass.clone() {
            let bypass = bypass
                .split(',')
                .map(|h| {
                    let mut host = String::from(h.trim());
                    if !host.starts_with('\'') && !host.starts_with('"') {
                        host = String::from("'") + &host;
                    }
                    if !host.ends_with('\'') && !host.ends_with('"') {
                        host = host + "'";
                    }
                    host
                })
                .collect::<Vec<String>>()
                .join(", ");

            let bypass = format!("[{bypass}]");

            gsettings()
                .args(["set", CMD_KEY, "ignore-hosts", bypass.as_str()])
                .status()?;
        }
        Ok(())
    }

    pub fn set_http(&self) -> Result<()> {
        if let Some(port) = self.http_port {
            set_proxy("http", &self.host, port)?;
        }
        Ok(())
    }

    pub fn set_https(&self) -> Result<()> {
        if let Some(port) = self.https_port {
            set_proxy("https", &self.host, port)?;
        }
        Ok(())
    }

    pub fn set_socks(&self) -> Result<()> {
        if let Some(port) = self.socks_port {
            set_proxy("socks", &self.host, port)?;
        }
        Ok(())
    }
}

fn gsettings() -> Command {
    Command::new("gsettings")
}

pub fn set_proxy(service: &str, host: &str, port: u16) -> Result<()> {
    let schema = format!("{CMD_KEY}.{service}");
    let schema = schema.as_str();

    let host = format!("'{}'", host);
    let host = host.as_str();
    let port = format!("{}", port);
    let port = port.as_str();

    gsettings().args(["set", schema, "host", host]).status()?;
    gsettings().args(["set", schema, "port", port]).status()?;

    Ok(())
}

pub fn get_proxy(service: &str) -> Result<Sysproxy> {
    let schema = format!("{CMD_KEY}.{service}");
    let schema = schema.as_str();

    let host = gsettings().args(["get", schema, "host"]).output()?;
    let host = from_utf8(&host.stdout).or(Err(Error::ParseStr))?.trim();
    let host = strip_str(host);

    let port = gsettings().args(["get", schema, "port"]).output()?;
    let port = from_utf8(&port.stdout).or(Err(Error::ParseStr))?.trim();
    let port = port.parse().unwrap_or(80u16);

    Ok(match service {
        "http" => Sysproxy {
            enable: false,
            host: String::from(host),
            http_port: Some(port),
            ..Default::default()
        },
        "https" => Sysproxy {
            enable: false,
            host: String::from(host),
            https_port: Some(port),
            ..Default::default()
        },
        "socks" => Sysproxy {
            enable: false,
            host: String::from(host),
            socks_port: Some(port),
            ..Default::default()
        },
        _ => return Err(Error::ParseStr),
    })
}

fn strip_str(text: &str) -> &str {
    text.strip_prefix('\'')
        .unwrap_or(text)
        .strip_suffix('\'')
        .unwrap_or(text)
}
