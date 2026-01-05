# Build stage
FROM rust:latest as builder

WORKDIR /usr/src/app
COPY . .

# Install dependencies needed for git2 (cmake, pkg-config, etc.)
RUN apt-get update && apt-get install -y cmake pkg-config libssl-dev zlib1g-dev

# Build the application
RUN cargo install --path .

# Runtime stage
FROM debian:trixie-slim

# Install runtime dependencies (openssl)
RUN apt-get update && apt-get install -y libssl3 zlib1g ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/local/cargo/bin/git-checkout /usr/local/bin/git-checkout

WORKDIR /app

# Set the startup command
CMD ["git-checkout"]
