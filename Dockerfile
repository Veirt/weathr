# Build stage
FROM rust:latest AS builder

WORKDIR /app

# Copy the project files
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Runtime stage
FROM debian:latest

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/weathr /usr/local/bin/

# Set the entrypoint
ENTRYPOINT ["weathr"]
