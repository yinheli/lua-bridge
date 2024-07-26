FROM --platform=$BUILDPLATFORM rust:1.75-bookworm as builder
WORKDIR /app
RUN apt-get update && \
    apt-get install -y libssl-dev lua5.4 liblua5.4-dev && \
    apt-get clean autoclean && apt-get autoremove --yes && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release

FROM --platform=$BUILDPLATFORM debian:bookworm-slim
LABEL org.opencontainers.image.authors="yinheli"
RUN apt-get update && \
    apt-get install -y libssl-dev lua5.4 liblua5.4-dev && \
    apt-get clean autoclean && apt-get autoremove --yes && rm -rf /var/lib/apt/lists/*
RUN mkdir /app
WORKDIR /app
COPY --from=builder /app/target/release/lua-bridge .
COPY --from=builder /app/README.md .
COPY --from=builder /app/.env-example .
CMD ./lua-bridge serve
