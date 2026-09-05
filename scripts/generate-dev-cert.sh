mkdir -p nginx/certs

openssl req \
  -x509 \
  -nodes \
  -newkey rsa:4096 \
  -keyout nginx/certs/server.key \
  -out nginx/certs/server.crt \
  -days 365 \
  -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost"