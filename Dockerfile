# Build stage: resolve dependencies, type-check, compile to a self-contained
# binary. Text assets (CSS, favicon, robots, age key) are part of the module
# graph and end up embedded in the binary.
FROM denoland/deno:2.9.7 AS build
WORKDIR /app
COPY deno.json deno.lock ./
RUN deno install --frozen
COPY main.ts ./
COPY src ./src
COPY static ./static
RUN deno check main.ts
RUN deno compile --frozen --allow-env --allow-net --allow-read=/app -o server main.ts

# Collect the shared libraries the binary links against (glibc + loader), so
# the runtime stage can be an empty filesystem. Works on any target arch.
RUN mkdir /out && ldd /app/server | grep -oE '/[^ ]+' | sort -u | xargs -I{} install -D {} /out{}

FROM scratch

# tini for correct signal handling / reaping as PID 1.
COPY --from=build /tini /tini
COPY --from=build /out/ /
COPY --from=build /app/server /server
COPY --from=build /app/static /app/static

WORKDIR /app
USER 1000:1000
EXPOSE 8080
ENTRYPOINT ["/tini", "--", "/server"]
