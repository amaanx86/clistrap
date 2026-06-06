use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use serde::Deserialize;

use crate::{
    auth::{
        pkce::{generate_state, wait_for_callback, PkceChallenge},
        provider,
        store::{self, TokenSet},
    },
    cli::AuthCommands,
    config::Config,
    output,
};

pub async fn run(action: AuthCommands) -> Result<()> {
    match action {
        AuthCommands::Login { email } => login(email).await,
        AuthCommands::Logout => logout(),
        AuthCommands::Status => status(),
    }
}

async fn login(email: Option<String>) -> Result<()> {
    let config = Config::load()?;

    if let Some(ref addr) = email {
        let domain = addr
            .split('@')
            .nth(1)
            .context("Invalid email address - expected user@domain.com")?;

        if domain != config.company.domain {
            anyhow::bail!(
                "Email domain @{domain} does not match this CLI's configured domain @{}.\n  Use your @{} address.",
                config.company.domain,
                config.company.domain,
            );
        }
    }

    let port = config.auth.redirect_port;
    let redirect_uri = format!("http://localhost:{port}/callback");
    let pkce = PkceChallenge::new();
    let state = generate_state();

    let url = provider::auth_url(&config.auth, &redirect_uri, &pkce.challenge, &state)?;

    output::header(&format!("Authenticating with {} via Entra ID", config.company.name));
    output::info("Opening browser for SSO login...");
    output::info(&format!(
        "If the browser does not open, paste this URL:\n  {url}"
    ));

    open::that(&url).ok();

    output::info(&format!("Waiting for Entra ID callback on port {port}..."));

    let callback = tokio::task::spawn_blocking(move || wait_for_callback(port))
        .await
        .context("Callback thread panicked")??;

    if callback.state != state {
        anyhow::bail!("State mismatch - possible CSRF. Please try again.");
    }

    output::info("Exchanging authorization code for tokens...");

    let token_set = exchange_code(
        &config.auth,
        &callback.code,
        &redirect_uri,
        &pkce.verifier,
    )
    .await?;

    store::save(&token_set)?;

    output::success(&format!(
        "Logged in as {}",
        token_set.email.as_deref().unwrap_or("unknown")
    ));
    if let Some(ref name) = token_set.name {
        output::kv("Name", name);
    }
    output::kv("Expires", &token_set.expires_at.to_rfc3339());

    Ok(())
}

fn logout() -> Result<()> {
    store::clear()?;
    output::success("Logged out - credentials cleared.");
    Ok(())
}

fn status() -> Result<()> {
    match store::load()? {
        None => {
            output::warn("Not logged in. Run `auth login` to authenticate.");
        }
        Some(ref tokens) if tokens.is_expired() => {
            output::warn("Session expired. Run `auth login` to re-authenticate.");
            if let Some(ref email) = tokens.email {
                output::kv("Last user", email);
            }
        }
        Some(ref tokens) => {
            output::success("Authenticated");
            if let Some(ref email) = tokens.email {
                output::kv("Email", email);
            }
            if let Some(ref name) = tokens.name {
                output::kv("Name", name);
            }
            output::kv("Expires", &tokens.expires_at.to_rfc3339());
        }
    }
    Ok(())
}

// --- token exchange ---------------------------------------------------------

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    id_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

#[derive(Deserialize, Default)]
struct IdTokenClaims {
    email: Option<String>,
    /// Entra ID uses `upn` (user principal name) as the email field
    upn: Option<String>,
    name: Option<String>,
}

async fn exchange_code(
    config: &crate::config::AuthConfig,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
) -> Result<TokenSet> {
    let token_url = provider::token_url(config);

    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", config.client_id.as_str()),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("code_verifier", code_verifier),
    ];

    let client = reqwest::Client::new();
    let resp = client
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .context("Failed to reach Entra ID token endpoint")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Token exchange failed ({status}): {body}");
    }

    let token_resp: TokenResponse = resp
        .json()
        .await
        .context("Failed to parse token response from Entra ID")?;

    let expires_in = token_resp.expires_in.unwrap_or(3600);
    let expires_at = Utc::now() + chrono::Duration::seconds(expires_in as i64);

    let claims = token_resp
        .id_token
        .as_deref()
        .and_then(|t| decode_jwt_claims::<IdTokenClaims>(t).ok())
        .unwrap_or_default();

    Ok(TokenSet {
        access_token: token_resp.access_token,
        id_token: token_resp.id_token,
        refresh_token: token_resp.refresh_token,
        expires_at,
        // Entra ID may return email in `email` or `upn`
        email: claims.email.or(claims.upn),
        name: claims.name,
    })
}

fn decode_jwt_claims<T: serde::de::DeserializeOwned>(token: &str) -> Result<T> {
    let payload = token
        .split('.')
        .nth(1)
        .context("Invalid JWT: missing payload segment")?;

    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .context("Invalid JWT: payload is not valid base64url")?;

    serde_json::from_slice(&bytes).context("Invalid JWT: cannot deserialize claims")
}
