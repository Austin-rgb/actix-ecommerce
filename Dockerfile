# Stage 1: Build
FROM rust:latest as builder

WORKDIR /app
COPY . .

# 1. Define the argument at the top of your build stage
ARG GITHUB_TOKEN

# 2. Use the argument to configure git
RUN git config --global url."https://${GITHUB_TOKEN}:x-oauth-basic@github.com/".insteadOf "https://github.com/"

RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/actix-ecommerce .

EXPOSE 8080

CMD ["./actix-ecommerce"]
