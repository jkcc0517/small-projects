use crate::{config::AUTH_TOKEN, response::Response};
use pingora::prelude::*;
use tracing::info;

fn get_auth_token_from_header(auth_header: &str) -> Option<&str> {
    auth_header
        .get(..6)
        .filter(|prefix| prefix.eq_ignore_ascii_case("Basic "))
        .and_then(|_| auth_header.get(6..))
}

pub async fn basic_auth(session: &mut Session) -> Result<bool> {
    if !crate::config::CONFIG.auth.enabled {
        return Ok(false);
    }
    let header = session.req_header();
    let auth_string = header
        .headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    if get_auth_token_from_header(auth_string).unwrap_or_default()
        != crate::config::AUTH_TOKEN.to_string()
    {
        info!("Basic authentication failed");
        Response::new(401, "")
            .header(
                "WWW-Authenticate",
                r#"Basic realm="Restricted", charset="UTF-8""#,
            )
            .send(session)
            .await?;
        return Ok(true);
    }

    Ok(false)
}
