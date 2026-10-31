#[cfg(feature = "client")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use ed25519_dalek::{Signer, SigningKey};
    let now = chrono::Local::now();
    let datetime_string: String = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let body_string = serde_json::json!({
        "zone": "chuchenchin.com",
        "record": "fitness.chuchenchin.com",
        "record_type": "AAAA",
        "content": "2001:b011:7003:f7eb:be24:11ff:fe9e:d94a",
        "created_at": datetime_string,
    })
    .to_string();

    let pem_string = std::fs::read_to_string("./private.pem")
        .map_err(|e| format!("./private.pem not found, {}", e))?;

    let pem = pem::parse(pem_string.clone())?;
    let der = pem.contents();
    let key_bytes: [u8; 32] = der[der.len() - 32..].try_into()?;
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let signature = signing_key.sign(body_string.as_bytes());
    let signature_b64 = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        signature.to_bytes(),
    );

    let client = reqwest::Client::new();
    println!("Request body: {}", &body_string);
    let res = client
        .post("http://localhost:8787/dns")
        .header("Content-Type", "application/json")
        .header("X-Signature", signature_b64)
        .body(body_string)
        .send()
        .await?;

    println!("📡 Worker 回應狀態碼: {}", res.status());
    println!("💬 Worker 回應內容: {}", res.text().await?);

    Ok(())
}

// 如果沒開 feature 的保底提示
#[cfg(not(feature = "client"))]
fn main() {
    panic!("請使用 cargo run --bin ddns_client --features client 來執行我！");
}
