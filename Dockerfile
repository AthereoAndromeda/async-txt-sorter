FROM rust:latest AS builder

WORKDIR /app

COPY . .

RUN cargo build -r

# Production stage
FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/async-txt-sorter .

ENV RUST_LOG=info

ENTRYPOINT ["/app/async-txt-sorter"]
