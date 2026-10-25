# The site is plain HTML + CSS under src/ — there is no build step. This image
# only wraps a static file server around it: static-web-server (Rust, ~4 MB,
# musl-static, no runtime dependencies) copied into an empty filesystem.
FROM joseluisq/static-web-server:2.44.1-alpine AS sws

FROM scratch

COPY --from=sws /usr/local/bin/static-web-server /static-web-server
COPY src /public

# SERVER_ERROR_PAGE_404 is relative, so it resolves under SERVER_ROOT.
ENV SERVER_HOST=0.0.0.0 \
    SERVER_PORT=8080 \
    SERVER_ROOT=/public \
    SERVER_ERROR_PAGE_404=./noscript/404.html \
    SERVER_LOG_LEVEL=info

# Nothing is ever written: the Deployment's readOnlyRootFilesystem holds.
USER 1000:1000
EXPOSE 8080
ENTRYPOINT ["/static-web-server"]
