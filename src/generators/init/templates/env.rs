pub fn content() -> String {
    r#"# ── Réseau ───────────────────────────────────────────────────────────────────
PORT=
GRPC_PORT=
BIND_ADDRESS=

# ── CORS / front ─────────────────────────────────────────────────────────────
CORS_ALLOWED_ORIGINS=
FRONTEND_BASE_URL=

# ── Metrics (Bearer pour GET /metrics) ───────────────────────────────────────
METRICS_TOKEN=
METRICS_ALLOW_OPEN=

# ── Logging ──────────────────────────────────────────────────────────────────
APP_ENV=
RUST_LOG=
"#
    .to_string()
}
