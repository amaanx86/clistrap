use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSet {
    pub access_token: String,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub email: Option<String>,
    pub name: Option<String>,
}

impl TokenSet {
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }
}

fn credentials_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".clistrap")
        .join("credentials.json")
}

pub fn save(tokens: &TokenSet) -> Result<()> {
    let path = credentials_path();
    std::fs::create_dir_all(path.parent().unwrap())
        .context("Failed to create ~/.clistrap directory")?;

    let json = serde_json::to_string_pretty(tokens)?;
    std::fs::write(&path, json)
        .with_context(|| format!("Failed to write credentials to {}", path.display()))?;

    // Lock down file permissions on Unix (no group/world read)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

pub fn load() -> Result<Option<TokenSet>> {
    let path = credentials_path();
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read credentials from {}", path.display()))?;

    let tokens: TokenSet = serde_json::from_str(&content)
        .context("Credentials file is corrupt - run `auth login` to re-authenticate")?;

    Ok(Some(tokens))
}

pub fn clear() -> Result<()> {
    let path = credentials_path();
    if path.exists() {
        std::fs::remove_file(&path)
            .with_context(|| format!("Failed to remove {}", path.display()))?;
    }
    Ok(())
}
