#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod config;
mod guard;
mod response;

use async_trait::async_trait;
use bytes::Bytes;
use http::Method;
use pingora::prelude::*;
use pingora::server::configuration::ServerConf;
use response::Response;
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::guard::basic::basic_auth;

fn main() {
    tracing_subscriber::fmt()
        // .with_env_filter("info,load_balance=info")
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let _ = &*config::CONFIG;

    if config::CONFIG.auth.enabled {
        let _ = &*config::AUTH_TOKEN;
    }

    debug!("Start this program!");
    let mut my_server = Server::new(None).unwrap();
    let mut conf = ServerConf::default();

    conf.threads = 4; // 設定為你的 CPU 核心數
    // conf.upstream_keepalive_pool_size = 1000;
    my_server.configuration = Arc::new(conf);
    my_server.bootstrap();

    let upstreams =
        LoadBalancer::try_from_iter(config::CONFIG.upstream.servers.iter().map(String::as_str))
            .unwrap();

    let mut lb = http_proxy_service(&my_server.configuration, LB(Arc::new(upstreams)));
    let ipv4_address = "[::]:80";
    info!("Pingora listing on {}", ipv4_address);
    lb.add_tcp("[::]:80");

    if config::CONFIG.ssl.enabled {
        let cert_path = &config::CONFIG.ssl.cert;
        let key_path = &config::CONFIG.ssl.key;

        let mut tls_settings =
            pingora::listeners::tls::TlsSettings::intermediate(cert_path, key_path).unwrap();
        tls_settings.enable_h2();

        let ipv6_address = "[::]:443";
        info!("Pingora listing on {}", ipv6_address);
        lb.add_tls_with_settings(ipv6_address, None, tls_settings);
    }

    my_server.add_service(lb);
    my_server.run_forever();
}

pub struct LB(Arc<LoadBalancer<RoundRobin>>);
async fn get_test_page(session: &mut Session) -> Result<bool> {
    let body = Bytes::from_static(b"Hello, world!\n");

    let len_string: String = body.len().to_string();
    Response::new(200, body)
        .header("Content-Type", "text/plain; charset=utf-8")
        .header("Content-Length", len_string)
        .header("Server", format!("Pingora"))
        .send(session)
        .await?;

    Ok(true)
}

#[async_trait]
impl ProxyHttp for LB {
    /// For this small example, we don't need context storage
    type CTX = ();
    fn new_ctx(&self) -> () {
        ()
    }
    async fn request_filter(&self, session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool> {
        if let Ok(true) = guard::ip::check_client_ip_allowing(session).await {
            return Ok(true);
        }
        if let Ok(true) = basic_auth(session).await {
            return Ok(true);
        }
        let client_addr = session.client_addr();

        debug!(
            client_addr = ?client_addr,
            "Incoming connection"
        );

        let req = session.req_header();

        if req.method == Method::GET && req.uri.path() == "/test" {
            return get_test_page(session).await;
        }

        Ok(false)
    }
    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let upstream = self
            .0
            .select(b"", 256) // hash doesn't matter for round robin
            .unwrap();

        // `HttpPeer::new(address, tls, sni)`:
        // - `true` enables TLS, because the 1.1.1.1 demo backends are served over HTTPS.
        //   Use `false` for a plain-HTTP upstream (e.g. a local server).
        // - the SNI `one.one.one.one` must match the upstream's TLS certificate.
        //   When TLS is disabled the SNI is unused, so pass an empty string instead.

        let mut peer = Box::new(HttpPeer::new(upstream.addr.clone(), false, "".to_string()));

        // peer.options.idle_timeout = Some(std::time::Duration::from_secs(60));
        Ok(peer)
    }
    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        // upstream_request.insert_header("Host", "one.one.one.one").unwrap();

        Ok(())
    }
    async fn logging(&self, session: &mut Session, e: Option<&Error>, _ctx: &mut Self::CTX)
    where
        Self::CTX: Send + Sync,
    {
        let client_ip = session
            .client_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "-".to_string());

        let req = session.req_header();
        let method = req.method.as_str();
        let uri = req.uri.to_string();
        let version = format!("{:?}", req.version);

        let status = session
            .response_written()
            .map_or(0, |resp| resp.status.as_u16());

        let bytes_sent = session.body_bytes_sent();

        tracing::info!(
            "{} - - \"{} {} {}\" {} {} error={:?}",
            client_ip,
            method,
            uri,
            version,
            status,
            bytes_sent,
            e,
        );
    }
}
