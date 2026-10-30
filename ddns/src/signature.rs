use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

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
