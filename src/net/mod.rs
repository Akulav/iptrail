//! Everything that talks to the network.

pub mod feeds;
pub mod intel;
pub mod public_ip;

use std::time::Duration;

/// Shared HTTP agent with sane timeouts and an identifying user agent.
///
/// Not HTTPS-only: ip-api.com's free tier is plain HTTP (its answers are treated as untrusted,
/// see `intel`). Every other service is called with an `https://` URL.
pub fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(12)))
        .timeout_connect(Some(Duration::from_secs(6)))
        .max_redirects(3)
        .user_agent(concat!("IPTrail/", env!("CARGO_PKG_VERSION"), " (+https://github.com/Akulav/iptrail)"))
        .build()
        .into()
}
