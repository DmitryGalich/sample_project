## Auth

1. Create:
```
services:
  keycloak_db:
    image: postgres:16-alpine
    container_name: keycloak_db
    restart: unless-stopped

    environment:
      POSTGRES_DB: keycloak_db
      POSTGRES_USER: keycloak_db_user
      POSTGRES_PASSWORD: keycloak_db_password

    volumes:
      - keycloak_db_data:/var/lib/postgresql/data

    networks:
      - internal_network

  keycloak:
    image: quay.io/keycloak/keycloak:26.3
    container_name: keycloak

    command:
      - start-dev

    environment:
      KC_DB: postgres
      KC_DB_URL: jdbc:postgresql://keycloak_db:5432/keycloak_db
      KC_DB_USERNAME: keycloak_db_user
      KC_DB_PASSWORD: keycloak_db_password

      KC_BOOTSTRAP_ADMIN_USERNAME: admin
      KC_BOOTSTRAP_ADMIN_PASSWORD: admin_password

    ports:
      - "8080:8080"

    networks:
      - internal_network

    depends_on:
      - keycloak_db


volumes:
  keycloak_db_data:

networks:
  internal_network:


```

2. Create realm: sample_project_realm
3. Create client: frontend
4. Create user: ivan / ivan_password
5. Set client valid redirect uri: http://localhost:3000/callback
6. In browser: http://localhost:8080/realms/sample_project_realm/protocol/openid-connect/auth?client_id=frontend&redirect_uri=http://localhost:3000/callback&response_type=code&scope=openid 
7. Opens login page
8. Enter ivan login and password
9. Opens additional data page
10. Enter email: ivan@example.com, firstname: ivan, lastname: ivanov
11. Will be redirect. From redirect link get code: http://localhost:3000/callback?session_state=6f0c79b3-5d61-4052-bbab-bf81aef5452b&iss=http%3A%2F%2Flocalhost%3A8080%2Frealms%2Fsample_project_realm&code=3dc8eaaf-89ec-4587-8941-eb0eff0546ff.6f0c79b3-5d61-4052-bbab-bf81aef5452b.1a792e2f-83ba-45f9-a2c0-b45e4787ed4a
```
session_state 6f0c79b3-5d61-4052-bbab-bf81aef5452b
iss http://localhost:8080/realms/sample_project_realm
code 3dc8eaaf-89ec-4587-8941-eb0eff0546ff.6f0c79b3-5d61-4052-bbab-bf81aef5452b.1a792e2f-83ba-45f9-a2c0-b45e4787ed4a
```
12. Make POST REQUEST: http://localhost:8080/realms/sample_project_realm/protocol/openid-connect/token

with Content-Type: application/x-www-form-urlencoded

with params: 
```
grant_type=authorization_code&
client_id=frontend&
code= {HERE CODE}
redirect_uri=http://localhost:3000/callback
```

Result: 
```JSON
{
    "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJtTm9NUVJWd2FUZkFOZ3VtQnZSUE5EdHhfLWxQZ21oR3J6X1o3OTNtYTNZIn0.eyJleHAiOjE3ODg5ODcwMzUsImlhdCI6MTc4ODk4NjczNSwiYXV0aF90aW1lIjoxNzg4OTg2NDMwLCJqdGkiOiJvbnJ0YWM6NGZhMzg2NTQtZTI0OC1jOTliLWUzMWYtMTY1YjQ3NTJlMDYwIiwiaXNzIjoiaHR0cDovL2xvY2FsaG9zdDo4MDgwL3JlYWxtcy9zYW1wbGVfcHJvamVjdF9yZWFsbSIsImF1ZCI6ImFjY291bnQiLCJzdWIiOiJiOGJlYjBmYS04N2NiLTRlMmMtODcxMi1mOTNiMjM5MDYzMTEiLCJ0eXAiOiJCZWFyZXIiLCJhenAiOiJmcm9udGVuZCIsInNpZCI6IjZmMGM3OWIzLTVkNjEtNDA1Mi1iYmFiLWJmODFhZWY1NDUyYiIsImFjciI6IjAiLCJhbGxvd2VkLW9yaWdpbnMiOlsiaHR0cDovL2xvY2FsaG9zdDozMDAwIl0sInJlYWxtX2FjY2VzcyI6eyJyb2xlcyI6WyJkZWZhdWx0LXJvbGVzLXNhbXBsZV9wcm9qZWN0X3JlYWxtIiwib2ZmbGluZV9hY2Nlc3MiLCJ1bWFfYXV0aG9yaXphdGlvbiJdfSwicmVzb3VyY2VfYWNjZXNzIjp7ImFjY291bnQiOnsicm9sZXMiOlsibWFuYWdlLWFjY291bnQiLCJtYW5hZ2UtYWNjb3VudC1saW5rcyIsInZpZXctcHJvZmlsZSJdfX0sInNjb3BlIjoib3BlbmlkIHByb2ZpbGUgZW1haWwiLCJlbWFpbF92ZXJpZmllZCI6ZmFsc2UsIm5hbWUiOiJpdmFuIGl2YW5vdiIsInByZWZlcnJlZF91c2VybmFtZSI6Iml2YW4iLCJnaXZlbl9uYW1lIjoiaXZhbiIsImZhbWlseV9uYW1lIjoiaXZhbm92IiwiZW1haWwiOiJpdmFuQGV4YW1wbGUuY29tIn0.ONVd-7GRxontcjUY-irs6RDS-YYkj38_69G5gme_GMq3QItOV4uxeHrjijfSxyRrhb9rjxVWYDsA3npgJAyPg6UH2t7tBPNJkh2oxh8TjvDiDI-Gjltuu5Kc6gdQklWpiOtVGNL2AUOpMid-lFdeU_jQUJ_cU3QaVgLgLdb_vdq4MGOhlBWSgK3cSrXH38kXEicTkT8K0a1cKOjaa-77vW_F5CgjcUDUM_iVwwlPkwsyolKGuwFdq-BpF-8orgm4txbAi8dwo4JjwyhEWDSYRymXg6rGlUnpoBhi0Q9xQQgPZYJt5tTHD_5LXT3NoVA6yyPZ6b8ZAe0fuJEFtO4vMg",
    "expires_in": 300,
    "refresh_expires_in": 1800,
    "refresh_token": "eyJhbGciOiJIUzUxMiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJlNDg1ZWU5Zi0wOTYzLTRhNDktOTk0ZS05YzNlOWU1MmViNGIifQ.eyJleHAiOjE3ODg5ODg1MzUsImlhdCI6MTc4ODk4NjczNSwianRpIjoiNzYxNTYwODItM2QzMy0yOTczLWIyZTktNzVkNTBmN2M5NjgxIiwiaXNzIjoiaHR0cDovL2xvY2FsaG9zdDo4MDgwL3JlYWxtcy9zYW1wbGVfcHJvamVjdF9yZWFsbSIsImF1ZCI6Imh0dHA6Ly9sb2NhbGhvc3Q6ODA4MC9yZWFsbXMvc2FtcGxlX3Byb2plY3RfcmVhbG0iLCJzdWIiOiJiOGJlYjBmYS04N2NiLTRlMmMtODcxMi1mOTNiMjM5MDYzMTEiLCJ0eXAiOiJSZWZyZXNoIiwiYXpwIjoiZnJvbnRlbmQiLCJzaWQiOiI2ZjBjNzliMy01ZDYxLTQwNTItYmJhYi1iZjgxYWVmNTQ1MmIiLCJzY29wZSI6Im9wZW5pZCByb2xlcyBiYXNpYyB3ZWItb3JpZ2lucyBwcm9maWxlIGVtYWlsIGFjciJ9.JURA5JLn7IF9KbMcl2tAs70it2ahe3PNBB65zbVwrf0YUcPFZZrH16H27RdFfjj4g59IU-WW8R-fltCUveqSTQ",
    "token_type": "Bearer",
    "id_token": "eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJtTm9NUVJWd2FUZkFOZ3VtQnZSUE5EdHhfLWxQZ21oR3J6X1o3OTNtYTNZIn0.eyJleHAiOjE3ODg5ODcwMzUsImlhdCI6MTc4ODk4NjczNSwiYXV0aF90aW1lIjoxNzg4OTg2NDMwLCJqdGkiOiJlMzViYjU5NS03YTBiLWI5ODMtMjJiYS1iZjk0NGZlMWVjYTUiLCJpc3MiOiJodHRwOi8vbG9jYWxob3N0OjgwODAvcmVhbG1zL3NhbXBsZV9wcm9qZWN0X3JlYWxtIiwiYXVkIjoiZnJvbnRlbmQiLCJzdWIiOiJiOGJlYjBmYS04N2NiLTRlMmMtODcxMi1mOTNiMjM5MDYzMTEiLCJ0eXAiOiJJRCIsImF6cCI6ImZyb250ZW5kIiwic2lkIjoiNmYwYzc5YjMtNWQ2MS00MDUyLWJiYWItYmY4MWFlZjU0NTJiIiwiYXRfaGFzaCI6IjFPVUY1bVFnMHpoQ1pxSzZBbzF2SXciLCJhY3IiOiIwIiwiZW1haWxfdmVyaWZpZWQiOmZhbHNlLCJuYW1lIjoiaXZhbiBpdmFub3YiLCJwcmVmZXJyZWRfdXNlcm5hbWUiOiJpdmFuIiwiZ2l2ZW5fbmFtZSI6Iml2YW4iLCJmYW1pbHlfbmFtZSI6Iml2YW5vdiIsImVtYWlsIjoiaXZhbkBleGFtcGxlLmNvbSJ9.P3HRlcUGEaXLOAmJUWSiimFbNEXmZDmuNK1CcOPoayvXjoJbgcRtDn56EPq-CB4qalMUFgkkZSdsfvbgp_VrLv9jWx3vVavoHvSAlaogXfpInsKD8cTQh4ez9PPQW4GEmfTNWhDt1XZ6GSjiRODH9a9Jv3Or_M6C8rcs0-gqxsMbJpjoK1tgLKH0yWIvkC09n-uvy5rrDj17Dc6sE2LfzRyVbv4y3kVz4Ro-A0tmGu0csNhlUInGE34INpWpgQnmqGeICcmWothUUSY4CwYdxN-JtbqFqePB1jdBMKc3OFPHvlpGE7ZRRtcxxOg3w1TghyzZKyRlM_fH_m4bZdcRLA",
    "not-before-policy": 0,
    "session_state": "6f0c79b3-5d61-4052-bbab-bf81aef5452b",
    "scope": "openid profile email"
}
```

If

```
{
    "error": "invalid_grant",
    "error_description": "Code not valid"
}
```

go again from point 6.

13. Access_token may be decoded by https://www.jwt.io/


```JSON
Header
{
  "alg": "RS256",
  "typ": "JWT",
  "kid": "mNoMQRVwaTfANgumBvRPNDtx_-lPgmhGrz_Z793ma3Y"
}
Payload
{
  "exp": 1788987035,
  "iat": 1788986735,
  "auth_time": 1788986430,
  "jti": "onrtac:4fa38654-e248-c99b-e31f-165b4752e060",
  "iss": "http://localhost:8080/realms/sample_project_realm",
  "aud": "account",
  "sub": "b8beb0fa-87cb-4e2c-8712-f93b23906311",
  "typ": "Bearer",
  "azp": "frontend",
  "sid": "6f0c79b3-5d61-4052-bbab-bf81aef5452b",
  "acr": "0",
  "allowed-origins": [
    "http://localhost:3000"
  ],
  "realm_access": {
    "roles": [
      "default-roles-sample_project_realm",
      "offline_access",
      "uma_authorization"
    ]
  },
  "resource_access": {
    "account": {
      "roles": [
        "manage-account",
        "manage-account-links",
        "view-profile"
      ]
    }
  },
  "scope": "openid profile email",
  "email_verified": false,
  "name": "ivan ivanov",
  "preferred_username": "ivan",
  "given_name": "ivan",
  "family_name": "ivanov",
  "email": "ivan@example.com"
}
Signature
...
```

14. JWKS (JSON Web Key Set): http://localhost:8080/realms/sample_project_realm/protocol/openid-connect/certs

```JSON
{
    "keys": [
        {
            "kid": "mNoMQRVwaTfANgumBvRPNDtx_-lPgmhGrz_Z793ma3Y",
            "kty": "RSA",
            "alg": "RS256",
            "use": "sig",
            "x5c": [
                "MIICtzCCAZ8CBgGgh9jbFjANBgkqhkiG9w0BAQsFADAfMR0wGwYDVQQDDBRzYW1wbGVfcHJvamVjdF9yZWFsbTAeFw0yNjA5MDkyMDIzNTFaFw0zNjA5MDkyMDI1MzFaMB8xHTAbBgNVBAMMFHNhbXBsZV9wcm9qZWN0X3JlYWxtMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAn8T3MXvjJgh0h0b9Eiy8dE5QHOz9duTKdDgFQ191ePTDQAIqEOSO6MJa01sfi8dNHIf80SfB0H1vOt/RvctHt8NHA8T41injb+YLuqPbWOPW5J6+Q3BXCYx3W6nosPV+/2AQ1bQBKfhRX7Es+yz+KDOimiL/8jjOgh/HW3lUTMq5Bvg4yJSlKpafi3G4rjcD9oixBjJs/kFdTa1XGM5nBISvgtw+VUf0joUZuS9gpCjU/xSQA3CJl1uuHwRJKdLceYnBrBznkp3/SyCZs9NRZB/C3tgpf7MjiKGUBLLDz8Myi2xCdd1rIr6nMaAq2yPFtmKXRCvN30FLnTq6OCHKtwIDAQABMA0GCSqGSIb3DQEBCwUAA4IBAQBR4BhaEby3oA6+jqVoD0NuJpyBDh6cW9AxytP4a48DxAFJTr5i2tVg5gZAiiaPgRPGsHc2mV8YxH0DFN/2soYFKW1woTytf5q4O5p7AA7oOGBs55KuDqfJPXXDoBe4B+4pjp+VLgkPJJGk+N4Qte8E80Ws0ffMouSKlwKZo+Feo6qcrzgs0UPJWpcQRZfo2rzdOVnLcINWI8vYkEXLOoVVgVk/yHL8px6/OXkoQTqFV8AfkAPY+tMy3T21QeTLwr4WwRNuaYjKB+rMDlszRHQUu/7A8lSxS1nxH8etqW2vM4u0OXKqCNt/p3738z/SYMewEPIGHuXZvggOJMlqZIZQ"
            ],
            "x5t": "5Mq_jwFDAIfvrHaxqywsTWDctxQ",
            "x5t#S256": "tI5dt2QhiDqmxOsXTH27SPM8LVCktPS0sRx-jGr9Pwg",
            "n": "n8T3MXvjJgh0h0b9Eiy8dE5QHOz9duTKdDgFQ191ePTDQAIqEOSO6MJa01sfi8dNHIf80SfB0H1vOt_RvctHt8NHA8T41injb-YLuqPbWOPW5J6-Q3BXCYx3W6nosPV-_2AQ1bQBKfhRX7Es-yz-KDOimiL_8jjOgh_HW3lUTMq5Bvg4yJSlKpafi3G4rjcD9oixBjJs_kFdTa1XGM5nBISvgtw-VUf0joUZuS9gpCjU_xSQA3CJl1uuHwRJKdLceYnBrBznkp3_SyCZs9NRZB_C3tgpf7MjiKGUBLLDz8Myi2xCdd1rIr6nMaAq2yPFtmKXRCvN30FLnTq6OCHKtw",
            "e": "AQAB"
        },
        {
            "kid": "fuW1D5BjPoenyg9_k77vuuaWrUP-3tNjDo6PUwwd5ZI",
            "kty": "RSA",
            "alg": "RSA-OAEP",
            "use": "enc",
            "x5c": [
                "MIICtzCCAZ8CBgGgh9jcajANBgkqhkiG9w0BAQsFADAfMR0wGwYDVQQDDBRzYW1wbGVfcHJvamVjdF9yZWFsbTAeFw0yNjA5MDkyMDIzNTFaFw0zNjA5MDkyMDI1MzFaMB8xHTAbBgNVBAMMFHNhbXBsZV9wcm9qZWN0X3JlYWxtMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAwNxF9tdxcQqqf8c+rJ0wa8R2wcfgdILzANxKUQtlwck+d/M6K59BzBDELS6H/dEpECSpoqIScuZAqkZ4ocWJQW+O3Ja864Fp69uluninLiYlTiCVkb7P9kffMbW7SMseghIjFJbzv3QXD5hiqiWMSR6wVXCzl7FVzLQBSvava2guezzdYXnTrXXD+mPNYVATi3yU5tugtHePQeG736ssnhokC+xPsTXVWn98rxvdKjQlg4Wml94dwOUdSBOwWMaPZax9OIxSE3mGGXLMEhJ092WIQtc5SMpPzidgBwFnGmUzRrVXMgZXY6Vxj27DPLBjq+6t3odEjYxFsdgZ1V3fuwIDAQABMA0GCSqGSIb3DQEBCwUAA4IBAQAXVtZCPzgZvJnrwtmFHJExXWyFltoa+wsKScxxAvM5xzqIPzL1d+MVQntWl/Faz3aL6Z8t6Fmo6IrjNjpbvIfCbeDFkFP8Dy0bF3tf/uRLNs534givU7ouwPQPoDX4OQ3dlUjF73ZxOZZU6TW8NChlB6BqibxVBQZoOtS5/77W6QpiuLP51SnrQ7oBlqSJV3XerCBZrV/eiz2Croqfb2XUgm5ki7cuE67BHnAWhBXN3gqVI0K7zjIfrCLNQQaN/fDK5vwthn3c6S9qB3USF2zcEpu9jLuVSO/s6ccBPgb5BRVUY9ZCOrTb2piq3N57cIGN9M5Y9EGZUSQdR0JaWr+B"
            ],
            "x5t": "xCMj2jPBk4Cd_lFjZnAaZzbjxsw",
            "x5t#S256": "29l0N0z1BJeRYNeVWxda1wTEzf-jlhTN_5RNelUK_KY",
            "n": "wNxF9tdxcQqqf8c-rJ0wa8R2wcfgdILzANxKUQtlwck-d_M6K59BzBDELS6H_dEpECSpoqIScuZAqkZ4ocWJQW-O3Ja864Fp69uluninLiYlTiCVkb7P9kffMbW7SMseghIjFJbzv3QXD5hiqiWMSR6wVXCzl7FVzLQBSvava2guezzdYXnTrXXD-mPNYVATi3yU5tugtHePQeG736ssnhokC-xPsTXVWn98rxvdKjQlg4Wml94dwOUdSBOwWMaPZax9OIxSE3mGGXLMEhJ092WIQtc5SMpPzidgBwFnGmUzRrVXMgZXY6Vxj27DPLBjq-6t3odEjYxFsdgZ1V3fuw",
            "e": "AQAB"
        }
    ]
}
```

15. Backend:

``` yaml
services:
  keycloak_db:
    image: postgres:16-alpine
    container_name: keycloak_db
    restart: unless-stopped

    environment:
      POSTGRES_DB: keycloak_db
      POSTGRES_USER: keycloak_db_user
      POSTGRES_PASSWORD: keycloak_db_password

    volumes:
      - keycloak_db_data:/var/lib/postgresql/data

    networks:
      - internal_network

  keycloak:
    image: quay.io/keycloak/keycloak:26.3
    container_name: keycloak

    command:
      - start-dev

    environment:
      KC_DB: postgres
      KC_DB_URL: jdbc:postgresql://keycloak_db:5432/keycloak_db
      KC_DB_USERNAME: keycloak_db_user
      KC_DB_PASSWORD: keycloak_db_password

      KC_BOOTSTRAP_ADMIN_USERNAME: admin
      KC_BOOTSTRAP_ADMIN_PASSWORD: admin_password

    ports:
      - "8080:8080"

    networks:
      - internal_network

    depends_on:
      - keycloak_db

  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile.dev
    container_name: backend
    restart: unless-stopped
    ports:
      - "8000:8000"
    volumes:
      - .:/workspace
      - cargo_cache:/usr/local/cargo/registry
    networks:
      - internal_network
    command: sleep infinity
    depends_on:
      - keycloak

volumes:
  keycloak_db_data:
  cargo_cache:

networks:
  internal_network:

```

``` dockerfile
FROM rust:1.88

RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    curl \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt

RUN cargo install sqlx-cli --version 0.7.4 --no-default-features --features native-tls,postgres 

WORKDIR /workspace
```

``` rust
use axum::{
    http::{header::AUTHORIZATION, StatusCode},
    routing::get,
    Router,
};


#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/protected", get(protected_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!("Started on 8000");
    axum::serve(listener, app).await.unwrap();
}

async fn protected_handler(
    headers: axum::http::HeaderMap,
) -> Result<String, StatusCode> {
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header["Bearer ".len()..];
    
    println!("Token: {}", token);

    Ok(format!("Token isn't tested yet"))
}
```

GET: http://localhost:8000/api/protected with Header
| Key | Value |
| -------- | ------- |
| Authorization | Bearer BIBA |


Backend console:
```
Started on 8000
Token: biba
```

Client console:
```
Token received but untested
```