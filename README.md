# clistrap

Enterprise CLI framework with Microsoft Entra ID SSO. Fork this repo and ship your company's internal CLI.

## What you get

- `auth login` - browser-based SSO via Entra ID (PKCE, no client secret)
- `auth status` / `auth logout` - session management
- `--help` and `--version` on every command and subcommand

Tokens are stored in `~/.clistrap/credentials.json` with `600` permissions.

---

## Entra ID app registration

Run these once against your tenant. You need `az` CLI installed and logged in (`az login`).

```bash
# Create the app registration (single-tenant, no client secret)
az ad app create \
  --display-name "my-cli" \
  --sign-in-audience "AzureADMyOrg" \
  --query "{clientId: appId, objectId: id}" \
  --output json

# Add the local redirect URI for the PKCE callback
# Replace <APP_ID> with the appId returned above
az ad app update \
  --id <APP_ID> \
  --public-client-redirect-uris "http://localhost:8899/callback"

# Verify
az ad app show --id <APP_ID> \
  --query "{appId: appId, publicClient: publicClient}" \
  --output json
```

The redirect URI is registered as a native/installed-client URI, which is required for public PKCE flows. No secret is created or needed.

---

## Configuration

```bash
cp clistrap.example.toml clistrap.toml
```

```toml
[company]
name   = "Acme Corp"
domain = "acme.com"         # only users with this email domain can log in

[auth]
tenant_id = "<TENANT_ID>"   # Azure portal > Entra ID > Overview
client_id = "<APP_ID>"      # from az ad app create output above
```

`clistrap.toml` is gitignored. Commit only `clistrap.example.toml` with placeholder values.

To find your tenant ID:

```bash
az account show --query tenantId --output tsv
```

---

## Rename your CLI

`cargo generate --name <your-name>` sets the binary name automatically via `Cargo.toml`.

For a manual fork, edit `Cargo.toml`:

```toml
[package]
name = "acme"

[[bin]]
name = "acme"
path = "src/main.rs"
```

Then `make install` picks up the name automatically - no flags needed.

---

## Build and install

```bash
make build        # compile release binary
make install      # build and copy to /usr/local/bin/clistrap
make reinstall    # build and replace the existing binary
make uninstall    # remove from /usr/local/bin
make clean        # remove build artifacts
```

Install under a custom name without touching Cargo.toml:

```bash
make install CLI_NAME=acme
```

Change the install directory:

```bash
make install BIN_DIR=~/.local/bin
```

---

## Quick start

### With cargo-generate (recommended)

```bash
cargo install cargo-generate       # one-time setup
cargo generate --git git@github.com:amaanax86/clistrap.git --name my-cli
cd my-cli
cp clistrap.example.toml clistrap.toml
make install
my-cli auth login
```

`clistrap.example.toml` is pre-filled with your answers. `make install` picks up the binary name from `Cargo.toml` automatically.

### Manual fork

```bash
git clone https://github.com/amaanax86/clistrap my-cli
cd my-cli
# edit Cargo.toml: set name = "my-cli" under [package] and [[bin]]
cp clistrap.example.toml clistrap.toml   # fill in tenant_id, client_id, domain
make install
my-cli auth login
```

---

## Adding commands

Create `src/commands/<name>.rs`, implement a `run()` function, then:

1. Add `pub mod <name>;` to `src/commands/mod.rs`
2. Add the variant to `Commands` in `src/cli.rs`
3. Add the match arm in `src/main.rs`

---

## Usage

```text
clistrap <COMMAND>

Commands:
  auth  Authenticate with your company SSO via Entra ID
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```text
clistrap auth <COMMAND>

Commands:
  login   Open browser for SSO login and store credentials
  logout  Clear stored credentials
  status  Show current authentication status
```

---

## How the auth flow works

1. CLI generates a PKCE code verifier and challenge (SHA-256, base64url)
2. Opens browser to `login.microsoftonline.com/.../oauth2/v2.0/authorize`
3. Starts a local HTTP server on `localhost:8899`
4. Entra ID redirects back with an authorization code
5. CLI exchanges the code and verifier for tokens (no secret involved)
6. ID token claims (`email`, `name`) are extracted and stored in `~/.clistrap/credentials.json`

The callback port is configurable via `redirect_port` in `clistrap.toml`.
