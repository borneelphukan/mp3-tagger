use reqwest::Client;
use std::sync::OnceLock;
use std::time::Duration;

/// One connection pool shared by every request in the process.
///
/// Building a `reqwest::Client` configures TLS roots and connection pools, so
/// doing it per artwork lookup was both slow and wasteful.
static CLIENT: OnceLock<Client> = OnceLock::new();

pub fn client() -> &'static Client {
    CLIENT.get_or_init(|| {
        Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(60))
            .pool_idle_timeout(Duration::from_secs(90))
            .user_agent(concat!("xur-porisoy/", env!("CARGO_PKG_VERSION")))
            .build()
            .unwrap_or_default()
    })
}
