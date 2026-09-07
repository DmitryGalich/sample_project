# Sample project

## Keycloak authentication

Start the stack:

```sh
docker compose up --build
```

Keycloak is available at `http://localhost:8081` (`admin` / `admin`). The imported realm is `sample`; the development user is `demo` / `demo` and the API client is `sample-api`.

Get a token:

```sh
TOKEN=$(curl -s -X POST 'http://localhost:8081/realms/sample/protocol/openid-connect/token' \
  -H 'content-type: application/x-www-form-urlencoded' \
  --data-urlencode 'client_id=sample-api' \
  --data-urlencode 'grant_type=password' \
  --data-urlencode 'username=demo' \
  --data-urlencode 'password=demo' | jq -r .access_token)
```

The public health endpoint is `GET http://localhost:8080/api/backend/health`. Protected endpoints require `Authorization: Bearer <token>`; for example `GET http://localhost:8080/api/backend/me`.
