FROM rust:1.85-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/maxt-relay /usr/local/bin/maxt-relay
USER 65532:65532
EXPOSE 8080
ENTRYPOINT ["maxt-relay"]
