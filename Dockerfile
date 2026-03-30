# Build stage
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM alpine:3.20

RUN apk add --no-cache ca-certificates && \
    addgroup -g 1001 -S appgroup && \
    adduser -S appuser -u 1001 -G appgroup

WORKDIR /app
COPY --from=builder /app/target/release/ferris-search /app/ferris-search
RUN chown appuser:appgroup /app/ferris-search

USER appuser

ENV DEFAULT_SEARCH_ENGINE=bing
ENV PORT=3000

EXPOSE 3000

CMD ["/app/ferris-search"]
