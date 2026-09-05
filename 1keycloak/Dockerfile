FROM quay.io/keycloak/keycloak:26.7.3 AS builder

ENV KC_DB=postgres
ENV KC_HEALTH_ENABLED=true
ENV KC_METRICS_ENABLED=false

RUN /opt/keycloak/bin/kc.sh build


FROM quay.io/keycloak/keycloak:26.7.3

COPY --from=builder /opt/keycloak/ /opt/keycloak/

COPY realm-export.json \
    /opt/keycloak/data/import/app-realm.json

USER 1000

ENTRYPOINT ["/opt/keycloak/bin/kc.sh"]