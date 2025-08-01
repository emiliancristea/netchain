# Multi-stage build for Netchain node
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && \
    apt-get install -y \
    build-essential \
    git \
    clang \
    curl \
    libssl-dev \
    llvm \
    libudev-dev \
    protobuf-compiler \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /netchain

# Copy source code
COPY . .

# Build the project
RUN cargo build --release

# Production image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create netchain user
RUN useradd -m -u 1000 -U -s /bin/sh -d /netchain netchain

# Create directories
RUN mkdir -p /data /chain-specs && \
    chown -R netchain:netchain /data /chain-specs

# Copy binary
COPY --from=builder /netchain/target/release/netchain-node /usr/local/bin/
COPY --from=builder /netchain/target/release/netchain-benchmarks /usr/local/bin/

# Copy chain specifications
COPY node/chain-specs/ /chain-specs/

# Set permissions
RUN chmod +x /usr/local/bin/netchain-node /usr/local/bin/netchain-benchmarks

# Switch to netchain user
USER netchain

# Create data directory
WORKDIR /data

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9933/health || exit 1

# Default command
CMD ["netchain-node", "--help"]