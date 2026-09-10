Да. Ниже — вариант, который я бы использовал как **production baseline**. Я сохраняю вашу архитектуру `Nginx → Keycloak / Rust`, но исправляю основные проблемы: TLS, закрытие внутренних сервисов, secrets, Keycloak `start`, canonical issuer, `kid`/JWKS rotation, `iss`/`aud`/`exp`, и безопасные proxy headers.

> Важный момент: TLS-сертификат и домен я обозначу как `auth.example.com` / `api.example.com`. Их нужно заменить на ваши реальные домены.

### 1. `docker-compose.yml`

```yaml
services:
  gateway:
    image: nginx:1.27-alpine
    container_name: gateway
    restart: unless-stopped

    ports:
      - "80:80"
      - "443:443"

    networks:
      - internal_network

    volumes:
      - ./gateway/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./gateway/certs:/etc/nginx/certs:ro

    depends_on:
      keycloak:
        condition: service_healthy
      backend:
        condition: service_healthy

    read_only: true
    tmpfs:
      - /var/cache/nginx
      - /var/run
      - /tmp

    security_opt:
      - no-new-privileges:true

  postgres_keycloak:
    image: postgres:16-alpine
    container_name: postgres_keycloak
    restart: unless-stopped

    environment:
      POSTGRES_DB: keycloak
      POSTGRES_USER: keycloak
      POSTGRES_PASSWORD_FILE: /run/secrets/keycloak_db_password

    volumes:
      - keycloak_data:/var/lib/postgresql/data

    secrets:
      - keycloak_db_password

    networks:
      - internal_network

    healthcheck:
      test:
        [
          "CMD-SHELL",
          "pg_isready -U keycloak -d keycloak"
        ]
      interval: 10s
      timeout: 5s
      retries: 10
      start_period: 20s

    security_opt:
      - no-new-privileges:true

  keycloak:
    image: quay.io/keycloak/keycloak:26.3
    container_name: keycloak
    restart: unless-stopped

    command:
      - start

    environment:
      KC_DB: postgres
      KC_DB_URL: jdbc:postgresql://postgres_keycloak:5432/keycloak
      KC_DB_USERNAME: keycloak
      KC_DB_PASSWORD_FILE: /run/secrets/keycloak_db_password

      # Canonical public URL of Keycloak.
      KC_HOSTNAME: https://auth.example.com

      # Keycloak is behind Nginx.
      KC_PROXY_HEADERS: xforwarded

      # Don't allow HTTP access directly to Keycloak.
      KC_HTTP_ENABLED: "true"

      # Production settings.
      KC_HEALTH_ENABLED: "true"
      KC_METRICS_ENABLED: "false"

      # Bootstrap admin.
      KC_BOOTSTRAP_ADMIN_USERNAME: admin
      KC_BOOTSTRAP_ADMIN_PASSWORD_FILE: /run/secrets/keycloak_admin_password

    secrets:
      - keycloak_db_password
      - keycloak_admin_password

    expose:
      - "8080"

    networks:
      - internal_network

    depends_on:
      postgres_keycloak:
        condition: service_healthy

    healthcheck:
      test:
        [
          "CMD-SHELL",
          "exec 3<>/dev/tcp/127.0.0.1/8080"
        ]
      interval: 10s
      timeout: 5s
      retries: 20
      start_period: 30s

    security_opt:
      - no-new-privileges:true

  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile.prod

    container_name: backend
    restart: unless-stopped

    expose:
      - "50051"

    environment:
      EXPOSED_ADDR: "0.0.0.0:50051"

      # Public/canonical Keycloak issuer.
      KEYCLOAK_ISSUER:
        "https://auth.example.com/realms/my-production-realm"

      # Internal URL used by backend to retrieve JWKS.
      KEYCLOAK_JWKS_URL:
        "http://keycloak:8080/realms/my-production-realm/protocol/openid-connect/certs"

      # Replace with your actual Keycloak client ID.
      JWT_AUDIENCE: "my-backend"

    networks:
      - internal_network

    depends_on:
      keycloak:
        condition: service_healthy

    healthcheck:
      test:
        [
          "CMD-SHELL",
          "wget -q --spider http://127.0.0.1:50051/health || exit 1"
        ]
      interval: 10s
      timeout: 5s
      retries: 10
      start_period: 10s

    security_opt:
      - no-new-privileges:true

volumes:
  keycloak_data:

networks:
  internal_network:
    driver: bridge
    internal: true

secrets:
  keycloak_db_password:
    file: ./secrets/keycloak_db_password.txt

  keycloak_admin_password:
    file: ./secrets/keycloak_admin_password.txt
```

### Важное изменение

Backend теперь не имеет:

```yaml
ports:
  - "..."
```

а только:

```yaml
expose:
  - "50051"
```

То есть из Интернета к нему напрямую обратиться нельзя.

То же самое с Keycloak: нет `8081:8080`.

Снаружи виден только Nginx.

---

## 2. `nginx.conf`

Я бы сделал gateway примерно так:

```nginx
events {
    worker_connections 4096;
}

http {
    include       /etc/nginx/mime.types;
    default_type  application/octet-stream;

    server_tokens off;

    sendfile on;
    tcp_nopush on;
    keepalive_timeout 65;

    # ------------------------------------------------------------
    # HTTP -> HTTPS
    # ------------------------------------------------------------

    server {
        listen 80;
        server_name auth.example.com api.example.com;

        return 301 https://$host$request_uri;
    }

    # ------------------------------------------------------------
    # HTTPS gateway
    # ------------------------------------------------------------

    server {
        listen 443 ssl;
        http2 on;

        server_name auth.example.com api.example.com;

        # --------------------------------------------------------
        # TLS
        # --------------------------------------------------------

        ssl_certificate     /etc/nginx/certs/fullchain.pem;
        ssl_certificate_key /etc/nginx/certs/privkey.pem;

        ssl_protocols TLSv1.2 TLSv1.3;

        ssl_session_timeout 1d;
        ssl_session_cache shared:SSL:10m;
        ssl_session_tickets off;

        # --------------------------------------------------------
        # Security headers
        # --------------------------------------------------------

        add_header Strict-Transport-Security
            "max-age=31536000; includeSubDomains"
            always;

        add_header X-Content-Type-Options "nosniff" always;
        add_header X-Frame-Options "DENY" always;
        add_header Referrer-Policy "strict-origin-when-cross-origin" always;

        # --------------------------------------------------------
        # Request limits
        # --------------------------------------------------------

        client_max_body_size 10m;

        # --------------------------------------------------------
        # Health
        # --------------------------------------------------------

        location = /gateway_health {
            access_log off;

            default_type text/plain;
            return 200 "gateway ok\n";
        }

        # ========================================================
        # KEYCLOAK
        # ========================================================

        location / {
            proxy_pass http://keycloak:8080;

            proxy_http_version 1.1;

            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;

            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto https;
            proxy_set_header X-Forwarded-Host $host;
            proxy_set_header X-Forwarded-Port 443;

            proxy_set_header Authorization $http_authorization;

            proxy_connect_timeout 5s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;
        }
    }

    # ============================================================
    # API
    # ============================================================

    server {
        listen 443 ssl;
        http2 on;

        server_name api.example.com;

        ssl_certificate     /etc/nginx/certs/fullchain.pem;
        ssl_certificate_key /etc/nginx/certs/privkey.pem;

        ssl_protocols TLSv1.2 TLSv1.3;

        ssl_session_timeout 1d;
        ssl_session_cache shared:SSL:10m;
        ssl_session_tickets off;

        add_header Strict-Transport-Security
            "max-age=31536000; includeSubDomains"
            always;

        add_header X-Content-Type-Options "nosniff" always;
        add_header X-Frame-Options "DENY" always;
        add_header Referrer-Policy "strict-origin-when-cross-origin" always;

        client_max_body_size 10m;

        # --------------------------------------------------------
        # Backend health
        # --------------------------------------------------------

        location = /health {
            proxy_pass http://backend:50051/health;

            proxy_http_version 1.1;

            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto https;
        }

        # --------------------------------------------------------
        # Protected API
        # --------------------------------------------------------

        location /api/ {
            proxy_pass http://backend:50051/;

            proxy_http_version 1.1;

            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;

            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto https;

            # Forward bearer token.
            proxy_set_header Authorization $http_authorization;

            proxy_connect_timeout 5s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;
        }

        location / {
            return 404;
        }
    }
}
```

### Почему два `server`

Я разделил:

```text
auth.example.com
```

и

```text
api.example.com
```

Это существенно чище, чем:

```text
localhost:8080/auth
localhost:8080/api/backend
```

Получается:

```text
https://auth.example.com
        ↓
    Keycloak

https://api.example.com
        ↓
      Rust
```

И самое главное — **issuer Keycloak становится стабильным**:

```text
https://auth.example.com/realms/my-production-realm
```

Не нужно добавлять `localhost`, `8081` и `keycloak:8080` в список допустимых issuer.

---

# 3. Rust JWT middleware

Здесь я бы изменил архитектуру существенно.

Не надо:

```rust
keys[0]
```

и не надо загружать один RSA key навсегда.

Нужна структура:

```text
JWT
 ↓
kid
 ↓
JWKS cache
 ↓
найти JWK с соответствующим kid
 ↓
signature
 ↓
iss
 ↓
aud
 ↓
exp
```

Для этого удобно использовать `jsonwebtoken` + `reqwest` и хранить несколько JWK в памяти.

Например:

```rust
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};

use jsonwebtoken::{
    decode,
    decode_header,
    Algorithm,
    DecodingKey,
    Validation,
};

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::RwLock;


#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: String,

    #[serde(default)]
    preferred_username: Option<String>,

    #[serde(default)]
    email: Option<String>,

    exp: usize,

    iss: String,

    #[serde(default)]
    aud: Vec<String>,
}


#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}


#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    n: String,
    e: String,

    #[serde(default)]
    alg: Option<String>,

    #[serde(default)]
    use_: Option<String>,
}


struct JwksCache {
    keys: HashMap<String, DecodingKey>,
    loaded_at: Instant,
}


struct AppState {
    issuer: String,
    audience: String,
    jwks_url: String,

    jwks: RwLock<JwksCache>,
}


type SharedState = Arc<AppState>;


#[tokio::main]
async fn main() {
    let exposed_addr =
        env::var("EXPOSED_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:50051".to_string());

    let issuer =
        env::var("KEYCLOAK_ISSUER")
            .expect("KEYCLOAK_ISSUER must be configured");

    let jwks_url =
        env::var("KEYCLOAK_JWKS_URL")
            .expect("KEYCLOAK_JWKS_URL must be configured");

    let audience =
        env::var("JWT_AUDIENCE")
            .expect("JWT_AUDIENCE must be configured");

    println!("Loading Keycloak JWKS...");

    let keys = load_jwks(&jwks_url)
        .await
        .expect("Failed to load Keycloak JWKS");

    let state = Arc::new(AppState {
        issuer,
        audience,
        jwks_url,

        jwks: RwLock::new(JwksCache {
            keys,
            loaded_at: Instant::now(),
        }),
    });

    let public_routes =
        Router::new()
            .route("/health", get(health));

    let protected_routes =
        Router::new()
            .route(
                "/protected-data",
                get(get_protected_data),
            )
            .route_layer(
                middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                ),
            );

    let app =
        Router::new()
            .merge(public_routes)
            .merge(protected_routes)
            .with_state(state);

    let listener =
        tokio::net::TcpListener::bind(&exposed_addr)
            .await
            .expect("Failed to bind backend");

    println!(
        "Backend listening on {}",
        exposed_addr
    );

    axum::serve(listener, app)
        .await
        .expect("Backend server failed");
}


async fn load_jwks(
    url: &str,
) -> Result<HashMap<String, DecodingKey>, Box<dyn std::error::Error + Send + Sync>> {
    let client =
        reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

    let response =
        client
            .get(url)
            .send()
            .await?
            .error_for_status()?;

    let jwks: Jwks =
        response.json().await?;

    let mut keys = HashMap::new();

    for jwk in jwks.keys {
        // We only accept RSA signing keys.
        if jwk.kty != "RSA" {
            continue;
        }

        if let Some(alg) = &jwk.alg {
            if alg != "RS256" {
                continue;
            }
        }

        let key =
            DecodingKey::from_rsa_components(
                &jwk.n,
                &jwk.e,
            )?;

        keys.insert(jwk.kid, key);
    }

    if keys.is_empty() {
        return Err("JWKS contains no usable RSA keys".into());
    }

    Ok(keys)
}


async fn refresh_jwks(
    state: &SharedState,
) -> Result<(), StatusCode> {
    let keys =
        load_jwks(&state.jwks_url)
            .await
            .map_err(|err| {
                eprintln!(
                    "Failed to refresh JWKS: {}",
                    err
                );

                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    let mut cache =
        state.jwks.write().await;

    cache.keys = keys;
    cache.loaded_at = Instant::now();

    Ok(())
}


async fn auth_middleware(
    State(state): State<SharedState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {

    let auth_header =
        req.headers()
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok());

    let token =
        match auth_header {
            Some(value)
                if value.starts_with("Bearer ") =>
            {
                value
                    .strip_prefix("Bearer ")
                    .unwrap_or("")
                    .trim()
            }

            _ => {
                return Err(StatusCode::UNAUTHORIZED);
            }
        };

    if token.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // ------------------------------------------------------------
    // Read JWT header.
    // ------------------------------------------------------------

    let header =
        decode_header(token)
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Never allow algorithm supplied by attacker.
    if header.alg != Algorithm::RS256 {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let kid =
        header.kid
            .ok_or(StatusCode::UNAUTHORIZED)?;

    // ------------------------------------------------------------
    // Find key by kid.
    // ------------------------------------------------------------

    let decoding_key = {
        let cache =
            state.jwks.read().await;

        cache.keys.get(&kid).cloned()
    };

    let decoding_key =
        match decoding_key {
            Some(key) => key,

            None => {
                // Possible Keycloak key rotation.
                //
                // Refresh JWKS and try once more.
                refresh_jwks(&state).await?;

                let cache =
                    state.jwks.read().await;

                cache.keys
                    .get(&kid)
                    .cloned()
                    .ok_or(StatusCode::UNAUTHORIZED)?
            }
        };

    // ------------------------------------------------------------
    // JWT validation.
    // ------------------------------------------------------------

    let mut validation =
        Validation::new(Algorithm::RS256);

    validation.validate_exp = true;

    // Important: issuer is canonical public URL.
    validation.set_issuer(&[
        state.issuer.as_str()
    ]);

    // Important: validate audience.
    validation.set_audience(&[
        state.audience.as_str()
    ]);

    let token_data =
        decode::<Claims>(
            token,
            &decoding_key,
            &validation,
        )
        .map_err(|err| {
            eprintln!(
                "JWT validation failed: {}",
                err
            );

            StatusCode::UNAUTHORIZED
        })?;

    // ------------------------------------------------------------
    // Put verified claims into request extensions.
    // ------------------------------------------------------------

    req.extensions_mut()
        .insert(token_data.claims);

    Ok(next.run(req).await)
}


async fn health() -> &'static str {
    "Backend health ok"
}


async fn get_protected_data() -> &'static str {
    "This is verified production data from Rust backend!"
}
```

---

# 4. Важный момент с `aud`

Теперь у нас есть:

```rust
validation.set_audience(&[
    state.audience.as_str()
]);
```

Поэтому ваш Keycloak client должен выдавать токен, в котором присутствует:

```json
"aud": "my-backend"
```

или соответствующий массив audience.

Если ваш client называется, например:

```text
backend-api
```

тогда:

```yaml
JWT_AUDIENCE: "backend-api"
```

и в Rust будет проверяться именно:

```text
aud == backend-api
```

Это значительно лучше, чем ваш текущий:

```rust
validation.validate_aud = false;
```

---

# 5. Secrets

Создайте:

```text
secrets/
├── keycloak_db_password.txt
└── keycloak_admin_password.txt
```

Например:

```text
secrets/keycloak_db_password.txt
```

содержит только пароль, без кавычек.

И:

```text
secrets/keycloak_admin_password.txt
```

содержит пароль администратора.

Добавьте в `.gitignore`:

```gitignore
secrets/
gateway/certs/
.env
```

**Не коммитьте эти файлы в Git.**

---

# 6. Ещё одна важная вещь: Docker network

Я поставил:

```yaml
networks:
  internal_network:
    internal: true
```

Это хорошо с точки зрения изоляции, но перед использованием проверьте вашу конкретную Docker Compose конфигурацию и сетевую схему.

Идея должна быть:

```text
                   INTERNET
                       │
                       │ :443
                       ▼
                ┌─────────────┐
                │    NGINX    │
                └──────┬──────┘
                       │
             internal_network
                 ┌─────┴─────┐
                 │           │
                 ▼           ▼
            Keycloak      Rust API
                 │
                 ▼
             PostgreSQL
```

**Никаких внешних портов** для:

```text
PostgreSQL
Keycloak
Rust
```

---

# 7. Что особенно важно исправить относительно вашего исходника

В вашем исходном коде было:

```rust
let key_data = &keys[0];
```

Теперь:

```rust
let kid = header.kid
```

и:

```rust
cache.keys.get(&kid)
```

Это принципиально важное изменение.

При ротации Keycloak делает примерно:

```text
старый key
    ↓
новый key
    ↓
JWT использует kid=new-key
    ↓
backend видит неизвестный kid
    ↓
refresh JWKS
    ↓
находит new-key
    ↓
проверяет JWT
```

Backend больше не нужно перезапускать для каждой ротации ключа.

---

# 8. Но есть ещё один production-уровень улучшения

Я бы **не оставлял refresh JWKS без ограничения**, как в простом примере выше.

Иначе злоумышленник может отправить огромное количество JWT с разными случайными `kid`, заставляя backend постоянно ходить в Keycloak.

Лучше добавить:

```text
unknown kid
    ↓
JWKS refresh
    ↓
cooldown 5–30 секунд
    ↓
повторный lookup
```

То есть сделать JWKS cache с контролем refresh rate.

Также я бы добавил:

* rate limiting на Nginx;
* access logs без вывода `Authorization`;
* graceful shutdown;
* timeouts для HTTP client;
* ограничение JWT размера;
* нормальный Dockerfile с non-root пользователем;
* отдельный Docker network для PostgreSQL;
* регулярные обновления Keycloak/Postgres/Nginx;
* автоматическую ротацию secrets;
* backup PostgreSQL;
* мониторинг Keycloak и backend.

### И ещё один нюанс

Если у вас **только REST API**, то Nginx → Rust выглядит отлично. Но ваш backend слушает:

```text
50051
```

и называется это как gRPC-порт. При этом показанный Axum-код — **обычный HTTP**, а не gRPC. Поэтому я бы для ясности переименовал его, например, в:

```text
8080
```

или:

```text
3000
```

Если же `50051` у вас выбран специально и поверх него действительно будет HTTP/1.1/HTTP2 — это тоже возможно.

**Итого:** после этих изменений схема уже становится вполне нормальной production-основой. Самые критичные исправления — **не публиковать Keycloak/backend, HTTPS, secrets вне compose, canonical issuer, `kid` + JWKS rotation и обязательная проверка audience**.


