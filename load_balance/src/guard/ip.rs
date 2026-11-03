use ipnet::{Ipv4Net, Ipv6Net};
use pingora::prelude::*;
use std::net::SocketAddr;
use std::sync::LazyLock;
use tracing::debug;

static CLOUDFLARE_IPV6: LazyLock<Vec<Ipv6Net>> = LazyLock::new(|| {
    [
        "2400:cb00::/32",
        "2606:4700::/32",
        "2803:f800::/32",
        "2405:b500::/32",
        "2405:8100::/32",
        "2a06:98c0::/29",
        "2c0f:f248::/32",
    ]
    .into_iter()
    .map(|cidr| cidr.parse().expect("Invalid Cloudflare IPv6 CIDR"))
    .collect()
});

static CLOUDFLARE_IPV4: LazyLock<Vec<Ipv4Net>> = LazyLock::new(|| {
    [
        "103.21.244.0/22",
        "103.22.200.0/22",
        "103.31.4.0/22",
        "104.16.0.0/13",
        "104.24.0.0/14",
        "108.162.192.0/18",
        "131.0.72.0/22",
        "141.101.64.0/18",
        "162.158.0.0/15",
        "172.64.0.0/13",
        "173.245.48.0/20",
        "188.114.96.0/20",
        "190.93.240.0/20",
        "197.234.240.0/22",
        "198.41.128.0/17",
    ]
    .into_iter()
    .map(|cidr| cidr.parse().expect("Invalid Cloudflare IPv4 CIDR"))
    .collect()
});

pub async fn check_client_ip_allowing(session: &mut Session) -> Result<bool> {
    let client_ip_addr = session
        .client_addr()
        .expect("Client address unavailable")
        .as_inet()
        .expect("Client address is not an IP address");

    let allow = match client_ip_addr {
        SocketAddr::V6(addr) => {
            let ipv6 = addr.ip();
            debug!(
                ?ipv6,
                is_loopback = ipv6.is_loopback(),
                is_unique_local = ipv6.is_unique_local(),
                is_unicast_link_local = ipv6.is_unicast_link_local(),
                "IPv6 address check"
            );
            ipv6.is_loopback()
                || ipv6.is_unique_local()
                || ipv6.is_unicast_link_local()
                || CLOUDFLARE_IPV6.iter().any(|network| network.contains(ipv6))
        }
        SocketAddr::V4(addr) => {
            let ipv4 = addr.ip();
            debug!(
                ?ipv4,
                is_private = ipv4.is_private(),
                is_loopback = ipv4.is_loopback(),
                "IPv4 address check"
            );
            ipv4.is_private()
                || ipv4.is_loopback()
                || ipv4.is_link_local()
                || CLOUDFLARE_IPV4.iter().any(|network| network.contains(ipv4))
        }
    };
    if !allow {
        session
            .respond_error_with_body(
                403,
                format!("You're IP address {} not allow", client_ip_addr.ip()).into(),
            )
            .await?;
    }
    Ok(false)
}
