FROM rust:1-bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN groupadd --system --gid 10001 lookout \
    && useradd --system --uid 10001 --gid lookout --home-dir /var/lib/look-out-api lookout \
    && mkdir -p /var/lib/look-out-api /etc/look-out-api \
    && chown lookout:lookout /var/lib/look-out-api
COPY --from=builder /build/target/release/look-out-api /usr/local/bin/look-out-api
COPY look-out-api.example.toml /etc/look-out-api/look-out-api.example.toml
ENV DATABASE_URL=sqlite:///var/lib/look-out-api/look-out-api.db \
    LOOK_OUT_API_CONFIG=/etc/look-out-api/look-out-api.toml
WORKDIR /var/lib/look-out-api
USER 10001:10001
VOLUME ["/var/lib/look-out-api"]
ENTRYPOINT ["look-out-api"]
CMD ["run"]
