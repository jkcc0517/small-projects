use serde::Deserialize;
use std::{fs, sync::LazyLock};

#[derive(Deserialize)]
pub struct Config {
    pub auth: AuthConfig,
    pub ssl: SslConfig,
    pub upstream: UpstreamConfig,
}

#[derive(Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub credentials: String,
}

#[derive(Deserialize)]
pub struct SslConfig {
    pub enabled: bool,
    pub cert: String,
    pub key: String,
}

#[derive(Deserialize)]
pub struct UpstreamConfig {
    pub servers: Vec<String>,
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let content = fs::read_to_string("./config.toml").expect("Failed to read ./config.toml");

    toml::from_str(&content).expect("Failed to parse config.toml")
});

pub static AUTH_TOKEN: LazyLock<String> = LazyLock::new(|| {
    fs::read_to_string(&CONFIG.auth.credentials)
        .expect("Failed to read credentials")
        .trim()
        .to_string()
});
