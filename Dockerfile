# Multi-stage Docker build for OAuth 2.0 Server

# Stage 1: Build the application
FROM rust:1.75-slim AS builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./
COPY domain/Cargo.toml ./domain/
COPY application/Cargo.toml ./application/
COPY infrastructure/Cargo.toml ./infrastructure/

# Copy source code
COPY domain/src ./domain/src
COPY application/src ./application/src
COPY infrastructure/src ./infrastructure/src
COPY infrastructure/migrations ./infrastructure/migrations

# Build the application in release mode
RUN cargo build --release --bin oauth-server

# Stage 2: Create the runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 oauth && \
    mkdir -p /app/data && \
    chown -R oauth:oauth /app

# Set working directory
WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/oauth-server /app/oauth-server

# Copy migrations
COPY --from=builder /app/infrastructure/migrations /app/migrations

# Copy .env.example as template
COPY .env.example /app/.env.example

# Set ownership
RUN chown -R oauth:oauth /app

# Switch to non-root user
USER oauth

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD [ "/bin/sh", "-c", "test -f /app/oauth-server" ]

# Set entrypoint
ENTRYPOINT ["/app/oauth-server"]
