FROM ubuntu:22.04

LABEL maintainer="Nexus Axiom Security"
LABEL description="eBPF security that actually kills exploits"

# Install dependencies
RUN apt-get update && apt-get install -y \
    clang \
    llvm \
    libbpf-dev \
    linux-headers-generic \
    curl \
    git \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy source
WORKDIR /nexus-axiom
COPY . .

# Build
RUN cargo build --release

# Build examples
WORKDIR /nexus-axiom/examples
RUN make

# Expose dashboard port
EXPOSE 8000

WORKDIR /nexus-axiom

# Default command: show help
CMD ["./target/release/nexus-axiom", "--help"]
