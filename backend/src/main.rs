mod server;

#[tokio::main]
async fn main() {
    let listen_address = "0.0.0.0:8000";
    let jwks_url = "http://keycloak:8080/realms/sample_project_realm/protocol/openid-connect/certs"
        .to_string();
    let allowed_issuers = vec!["http://localhost/realms/sample_project_realm".to_string()];
    let allowed_audiences = vec!["frontend".to_string(), "account".to_string()];

    let config = server::ServerConfig {
        jwks_url,
        allowed_issuers,
        allowed_audiences,
    };

    server::run_server(listen_address, config).await;
}
