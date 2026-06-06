use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use sha2::{Digest, Sha256};

pub struct PkceChallenge {
    pub verifier: String,
    pub challenge: String,
}

impl PkceChallenge {
    pub fn new() -> Self {
        // RFC 7636 recommends 32 bytes minimum for the verifier
        let mut verifier_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut verifier_bytes);
        let verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);

        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

        Self { verifier, challenge }
    }
}

pub fn generate_state() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub struct CallbackResult {
    pub code: String,
    pub state: String,
}

/// Blocks the calling thread waiting for a single OAuth2 callback request.
/// Run this inside `tokio::task::spawn_blocking`.
pub fn wait_for_callback(port: u16) -> Result<CallbackResult> {
    let server = tiny_http::Server::http(format!("127.0.0.1:{port}"))
        .map_err(|e| anyhow::anyhow!("Cannot bind callback server on port {port}: {e}"))?;

    let request = server
        .recv()
        .map_err(|e| anyhow::anyhow!("Callback server error: {e}"))?;

    let url = request.url().to_string();
    let result = parse_callback_url(&url)?;

    let html = r#"<!DOCTYPE html>
<html><head><title>Authenticated</title>
<style>
  body{font-family:system-ui,sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;margin:0;background:#f8fafc;}
  .card{text-align:center;padding:48px 40px;background:#fff;border-radius:16px;box-shadow:0 4px 24px rgba(0,0,0,.08);}
  h1{color:#16a34a;font-size:1.5rem;margin:0 0 8px;}
  p{color:#64748b;margin:0;}
  .check{font-size:3rem;display:block;margin-bottom:16px;}
</style></head>
<body><div class="card">
  <span class="check">✓</span>
  <h1>Authentication Successful</h1>
  <p>You can close this tab and return to your terminal.</p>
</div></body></html>"#;

    let response = tiny_http::Response::from_string(html).with_header(
        tiny_http::Header::from_bytes(b"Content-Type", b"text/html; charset=utf-8").unwrap(),
    );

    request.respond(response).ok();
    Ok(result)
}

fn parse_callback_url(url: &str) -> Result<CallbackResult> {
    let query = url
        .split_once('?')
        .map(|(_, q)| q)
        .context("No query string in callback URL")?;

    let mut code = None;
    let mut state = None;
    let mut error: Option<String> = None;
    let mut error_description: Option<String> = None;

    for pair in query.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let decoded = urlencoding::decode(value)
                .unwrap_or_default()
                .into_owned();
            match key {
                "code" => code = Some(decoded),
                "state" => state = Some(decoded),
                "error" => error = Some(decoded),
                "error_description" => error_description = Some(decoded),
                _ => {}
            }
        }
    }

    if let Some(err) = error {
        let desc = error_description.unwrap_or_default();
        anyhow::bail!("Auth provider returned error: {err} - {desc}");
    }

    Ok(CallbackResult {
        code: code.context("Missing 'code' in callback URL")?,
        state: state.context("Missing 'state' in callback URL")?,
    })
}
