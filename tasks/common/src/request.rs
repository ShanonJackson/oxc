use std::time::Duration;

use ureq::{Agent, Proxy, tls::RootCerts, tls::TlsConfig, tls::TlsProvider};

/// detect proxy from environment variable in following order:
/// HTTPS_PROXY | https_proxy | HTTP_PROXY | http_proxy | ALL_PROXY | all_proxy
fn detect_proxy() -> Option<Proxy> {
    for env in ["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy", "ALL_PROXY", "all_proxy"]
    {
        if let Ok(env) = std::env::var(env)
            && let Ok(proxy) = Proxy::new(&env)
        {
            return Some(proxy);
        }
    }
    None
}

fn build_agent(provider: TlsProvider) -> Agent {
    let tls_config =
        TlsConfig::builder().provider(provider).root_certs(RootCerts::PlatformVerifier).build();
    let config = Agent::config_builder()
        .proxy(detect_proxy())
        .timeout_global(Some(Duration::from_secs(5)))
        .tls_config(tls_config)
        .build();
    Agent::new_with_config(config)
}

/// build an agent with proxy automatically detected
pub fn agent() -> Agent {
    build_agent(TlsProvider::Rustls)
}

pub(crate) fn agent_with(provider: TlsProvider) -> Agent {
    build_agent(provider)
}
