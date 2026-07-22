pub fn content() -> String {
    r#"[package]
name = "api"
version = "0.1.0"
edition = "2024"

[dependencies]
axum = "0.8.9"
tokio = { version = "1.53.1", features = ["full"]}
tower-http = { version = "0.7.0", features = ["cors", "trace"] }
tracing = "0.1.44"
tracing-subscriber = { version = "0.3.23", features = ["env-filter", "json"] }
tracing-appender = "0.2"
anyhow = "1.0.104"
dotenvy = "0.15.7"
serde_json = "1.0.151"
thiserror = "2.0.19"
ipnet = "2.12.0"
prometheus = { version = "0.14.0", features = ["process"] }
tikv-jemallocator = "0.7.0"
subtle = "2.6"
sha2 = "0.11.0"
"#
    .to_string()
}
