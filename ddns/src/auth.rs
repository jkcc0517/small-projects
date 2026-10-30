use base64::{engine::general_purpose::STANDARD, Engine as _};
use sha2::{Digest, Sha256};
use worker::*;

fn get_sha256(s: &str) -> String {
    hex::encode(Sha256::digest(s.as_bytes()))
}

pub async fn basic_auth(req: &Request, ctx: &RouteContext<()>) -> Result<()> {
    let basic_auth_hash = ctx.secret("BASIC_AUTH_HASH")?.to_string();

    let auth_header = req
        .headers()
        .get("authorization")?
        .ok_or_else(|| Error::RustError("Missing Authorization header".into()))?;

    let encoded = auth_header
        .split_once(' ')
        .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("Basic"))
        .map(|(_, credentials)| credentials)
        .ok_or_else(|| Error::RustError("Invalid Authorization scheme".into()))?;

    let decoded = STANDARD
        .decode(encoded)
        .map_err(|e| Error::RustError(e.to_string()))?;

    let credentials = String::from_utf8(decoded).map_err(|e| Error::RustError(e.to_string()))?;

    let (username, password) = credentials
        .split_once(':')
        .ok_or_else(|| Error::RustError("Invalid credentials format".into()))?;

    let hash = get_sha256(&format!("{username}:{password}"));

    if hash != basic_auth_hash {
        return Err(Error::RustError("Authentication failed".into()));
    }

    Ok(())
}
