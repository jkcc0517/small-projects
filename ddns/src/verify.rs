use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use time::{macros::format_description, Duration, OffsetDateTime, PrimitiveDateTime, UtcOffset};

pub async fn verify_signature(pem_text: &str, body: &[u8], signature_text: &str) -> Result<()> {
    let pem = pem::parse(pem_text).context("ED25519 Public Key error")?;
    let der = pem.contents();
    let key_bytes: [u8; 32] = der[der.len() - 32..]
        .try_into()
        .context("Invalid public key length")?;
    let verifying_key = VerifyingKey::from_bytes(&key_bytes).context("VerifyingKey failed")?;

    let signature_bytes = STANDARD
        .decode(signature_text)
        .context("Invalid signature(X-Signature) format")?;
    let signature = Signature::from_slice(&signature_bytes).context("Invalid signature format")?;
    verifying_key
        .verify(&body, &signature)
        .context("Signature verification failed")?;
    Ok(())
}

pub fn verify_content(content: &str) -> worker::Result<()> {
    if content.is_empty() {
        return Err("Content can't be empty".into());
    }
    Ok(())
}

pub fn verify_datetime(datetime: &str, expire_minutes: i64) -> worker::Result<()> {
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    let incoming_time = PrimitiveDateTime::parse(datetime, &format)
        .map_err(|_| worker::Error::from("Invalid time format"))?;
    let taiwan_offset = UtcOffset::from_hms(8, 0, 0).unwrap();
    let now_taiwan_offset = OffsetDateTime::now_utc().to_offset(taiwan_offset);
    let now_taiwan = PrimitiveDateTime::new(now_taiwan_offset.date(), now_taiwan_offset.time());
    if incoming_time > now_taiwan {
        return Err("Datetime error: Future time".into());
    }
    let time_difference: Duration = now_taiwan - incoming_time;
    if time_difference > Duration::minutes(expire_minutes) {
        return Err("Signature expired".into());
    }
    Ok(())
}
