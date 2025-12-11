# Build stage
FROM rust:1.75-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libdbus-1-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy src to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > src/lib.rs

# Build dependencies only (this layer is cached)
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src ./src

# Build the actual application
RUN touch src/main.rs src/lib.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies for notifications (optional)
RUN apt-get update && apt-get install -y \
    libdbus-1-3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -m -u 1000 logwatcher

# Copy binary from builder
COPY --from=builder /app/target/release/logwatcher /usr/local/bin/logwatcher

# Set ownership
RUN chown logwatcher:logwatcher /usr/local/bin/logwatcher

USER logwatcher

ENTRYPOINT ["logwatcher"]
CMD ["--help"]
