
ARG RUST_VERSION=1.74.0
ARG APP_NAME=dpch
FROM rust:${RUST_VERSION}-slim-bullseye AS build

RUN apt-get update -y && \
    apt-get install -y pkg-config make g++ libssl-dev && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rustup target add x86_64-unknown-linux-gnu

ARG APP_NAME
WORKDIR /app

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cargo build --locked --release
cp ./target/release/$APP_NAME /bin/server
EOF

FROM debian:bullseye-slim AS final

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    # --home "/nonexistent" \
    --shell "/sbin/nologin" \
    # --no-create-home \
    --uid "${UID}" \
    appuser && \
    apt update && apt install -y wget && \
    wget https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -O /usr/local/bin/yt-dlp && \
    chmod a+rx /usr/local/bin/yt-dlp

# RUN sudo usermod -aG docker appuser
USER appuser

# Copy the executable from the "build" stage.
COPY --from=build /bin/server /bin/
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# What the container should run when it is started.
CMD ["/bin/server"]
