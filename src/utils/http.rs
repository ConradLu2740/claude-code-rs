use once_cell::sync::Lazy;
use reqwest::Client;
use std::time::Duration;

pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(120))
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(60))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .expect("Failed to create HTTP client")
});

pub fn get_http_client() -> &'static Client {
    &HTTP_CLIENT
}
