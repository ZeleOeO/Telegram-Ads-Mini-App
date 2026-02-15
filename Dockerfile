# Stage 1: Build Frontend
FROM node:20-slim AS frontend-builder
WORKDIR /app/frontend

# Copy frontend dependency files
COPY frontend/package*.json ./
RUN npm ci

# Copy frontend source and build
COPY frontend/ ./
RUN npm run build

# Stage 2: Build Backend
FROM rust:latest AS backend-builder
WORKDIR /app

# Install build dependencies (needed for some crates like link-cplusplus or openssl-sys)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml migration/

# Create dummy src to cache dependencies
# We need to create a dummy main.rs for the app and the migration crate to build deps
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN mkdir migration/src && echo "fn main() {}" > migration/src/main.rs

# Build dependencies (this layer will be cached if Cargo.toml/lock don't change)
# We need to build enabling the features we use
RUN cargo build --release

# Now copy the actual source code
COPY src/ src/
COPY migration/src/ migration/src/

# Touch main files to force rebuild of the app code (not just deps)
RUN touch src/main.rs migration/src/main.rs

# Build the actual application
RUN cargo build --release

# Stage 3: Runtime
FROM debian:bullseye-slim AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates libssl1.1 openssl && rm -rf /var/lib/apt/lists/*

# Copy backend binary
COPY --from=backend-builder /app/target/release/telegram-ad-mini-app ./telegram-ad-mini-app

# Copy frontend build artifacts
# The Rust app expects to serve "frontend/dist" relative to its execution directory
COPY --from=frontend-builder /app/frontend/dist ./frontend/dist

# Expose the application port
EXPOSE 3000

# Set the entrypoint
CMD ["./telegram-ad-mini-app"]
