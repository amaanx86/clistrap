use anyhow::Result;
use crate::config::AuthConfig;

/// Builds the Entra ID authorization URL for the PKCE flow.
pub fn auth_url(
    config: &AuthConfig,
    redirect_uri: &str,
    code_challenge: &str,
    state: &str,
) -> Result<String> {
    let base = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
        config.tenant_id
    );

    let scopes = config.scopes.join(" ");

    let url = format!(
        "{base}?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
        urlencoding::encode(&config.client_id),
        urlencoding::encode(redirect_uri),
        urlencoding::encode(&scopes),
        urlencoding::encode(state),
        urlencoding::encode(code_challenge),
    );

    Ok(url)
}

/// Returns the Entra ID token endpoint for the configured tenant.
pub fn token_url(config: &AuthConfig) -> String {
    format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        config.tenant_id
    )
}
