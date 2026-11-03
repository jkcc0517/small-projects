# Pingora

A lightweight reverse proxy built with [Pingora](https://github.com/cloudflare/pingora).

## Build

Build the release binary for Alpine Linux (`x86_64-unknown-linux-musl`):

```bash
cargo build --release --target x86_64-unknown-linux-musl
```

The compiled binary is located at:

```text
target/x86_64-unknown-linux-musl/release/load_balance
```

## Deployment

Copy the binary to your Alpine Linux server.

The application expects `config.toml` to be located in the same directory as the executable.

Example directory structure:

```text
load_balance/
├── load_balance
├── config.toml
├── credentials
├── cert.pem
└── key.pem
```

## Configuration

The application loads `config.toml` from the executable's directory by default.

Example:

```toml
[upstream]
servers = [
    "192.168.1.150:30080"
]

[auth]
enabled = true
credentials = "./credentials"

[ssl]
enabled = true
certificate = "./cert.pem"
private_key = "./key.pem"
```

### Upstream

Configure one or more backend servers under `[upstream]`.

```toml
[upstream]
servers = [
    "192.168.1.150:30080",
    "192.168.1.151:30080"
]
```

### Basic Authentication

The credentials file stores the Base64-encoded `username:password` string.

Generate credentials:

```bash
echo -n "username:password" | base64
```

Example output:

```text
amFjazoxMjM0NTY=
```

Save the encoded value in the credentials file:

```bash
echo -n "amFjazoxMjM0NTY=" > credentials
```

The application reads the credentials file when initializing the authentication configuration.

**Note:** Base64 is an encoding format, not encryption. Always use HTTPS when Basic Authentication is enabled.

### SSL

Configure the certificate and private key paths under `[ssl]` in `config.toml`.

```toml
[ssl]
enabled = true
certificate = "./cert.pem"
private_key = "./key.pem"
```

Ensure the certificate and private key files are accessible to the application.

## Runtime

Before starting the application, ensure that:

- `config.toml` exists in the same directory as the executable.
- At least one upstream server is configured.
- The credentials file exists when Basic Authentication is enabled.
- The SSL certificate and private key exist when SSL is enabled.
- The application has permission to read all required files.
