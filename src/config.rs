use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

include!(concat!(env!("OUT_DIR"), "/baked_config.rs"));

const XOR_KEY: u8 = 0x5A;

fn deobfuscate(bytes: &[u8]) -> String {
    String::from_utf8(bytes.iter().map(|b| b ^ XOR_KEY).collect()).unwrap_or_default()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub company: CompanyConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CompanyConfig {
    pub name: String,
    pub domain: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthConfig {
    pub tenant_id: String,
    pub client_id: String,
    #[serde(default = "default_scopes")]
    pub scopes: Vec<String>,
    #[serde(default = "default_redirect_port")]
    pub redirect_port: u16,
}

pub fn default_scopes() -> Vec<String> {
    vec!["openid".into(), "profile".into(), "email".into()]
}

pub fn default_redirect_port() -> u16 {
    8899
}

impl Config {
    pub fn load() -> Result<Self> {
        // Baked-in config takes priority (compiled via build.rs env vars)
        if BAKED_CONFIG {
            return Ok(Config {
                company: CompanyConfig {
                    name: deobfuscate(BAKED_COMPANY),
                    domain: deobfuscate(BAKED_DOMAIN),
                },
                auth: AuthConfig {
                    tenant_id: deobfuscate(BAKED_TENANT_ID),
                    client_id: deobfuscate(BAKED_CLIENT_ID),
                    scopes: default_scopes(),
                    redirect_port: default_redirect_port(),
                },
            });
        }

        // Fall back to clistrap.toml for local development
        let path = locate_config_file().context(
            "clistrap.toml not found and no config was baked into this binary.\n  Copy clistrap.example.toml to clistrap.toml, or rebuild with CLISTRAP_* env vars set.",
        )?;

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Cannot read {}", path.display()))?;

        toml::from_str(&content)
            .with_context(|| format!("Invalid TOML in {}", path.display()))
    }
}

pub(crate) fn locate_config_file() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("CLISTRAP_CONFIG") {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }

    if let Ok(mut dir) = std::env::current_dir() {
        loop {
            let candidate = dir.join("clistrap.toml");
            if candidate.exists() {
                return Some(candidate);
            }
            if !dir.pop() {
                break;
            }
        }
    }

    if let Some(home) = dirs::home_dir() {
        let candidate = home.join(".clistrap").join("config.toml");
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}
