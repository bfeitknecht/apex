FROM --platform=$BUILDPLATFORM ghcr.io/typst/typst:0.15.1 AS builder
WORKDIR /work
COPY . .
RUN set -eu; \
    if [ -d vendor ]; then \
      mkdir -p /pkg/preview/exemel /pkg/preview/thyme; \
      [ -d vendor/exemel ] && cp -R vendor/exemel /pkg/preview/exemel/0.3.0; \
      [ -d vendor/thyme ] && cp -R vendor/thyme /pkg/preview/thyme/0.1.0; \
      typst compile --features html,bundle --format bundle --package-path /pkg site.typ /public; \
    else \
      typst compile --features html,bundle --format bundle site.typ /public; \
    fi

FROM joseluisq/static-web-server:2.44.1-alpine AS sws

FROM scratch
COPY --from=sws /usr/local/bin/static-web-server /static-web-server
COPY --from=builder /public /public
COPY sws.toml /sws.toml

ENV SERVER_HOST=0.0.0.0 \
    SERVER_PORT=8080 \
    SERVER_ROOT=/public \
    SERVER_ERROR_PAGE_404=./404.html \
    SERVER_CACHE_CONTROL_HEADERS=false \
    SERVER_CONFIG_FILE=/sws.toml \
    SERVER_LOG_LEVEL=info

USER 1000:1000
EXPOSE 8080
ENTRYPOINT ["/static-web-server"]
