FROM node:20-slim AS frontend-builder
WORKDIR /app/frontend

COPY frontend/package*.json ./
RUN npm ci

COPY frontend/ ./
RUN npm run build

FROM rust:latest AS backend-builder
WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml migration/

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN mkdir migration/src && echo "pub fn main() {}" > migration/src/lib.rs

RUN cargo build --release

COPY src/ src/
COPY migration/src/ migration/src/

RUN touch src/main.rs migration/src/lib.rs

RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates openssl && rm -rf /var/lib/apt/lists/*

COPY --from=backend-builder /app/target/release/telegram-ad-mini-app ./telegram-ad-mini-app
COPY --from=frontend-builder /app/frontend/dist ./frontend/dist
EXPOSE 3000

CMD ["./telegram-ad-mini-app"]
