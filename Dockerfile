# Stage 1: Build
FROM rust:latest as builder

WORKDIR /app
COPY . .

RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/actix-ecommerce .

EXPOSE 8080

CMD ["./actix-ecommerce"]