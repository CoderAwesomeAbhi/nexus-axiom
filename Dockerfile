# Stage 1: Build
FROM ubuntu:22.04 AS builder

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    clang llvm libbpf-dev libelf-dev libseccomp-dev \
    linux-headers-generic curl build-essential pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.75.0
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY ebpf/ ebpf/
COPY build.rs ./

RUN cargo build --release 2>/dev/null || cargo build --release --target x86_64-unknown-linux-gnu 2>/dev/null || cargo build --release

# Stage 2: Runtime (minimal)
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    libbpf0 libelf1 libseccomp2 \
    && rm -rf /var/lib/apt/lists/*

# eBPF requires these kernel interfaces
VOLUME ["/sys/kernel/debug", "/sys/fs/bpf", "/sys/kernel/btf"]

WORKDIR /nexus-axiom
COPY --from=builder /build/target/release/nexus-axiom ./nexus-axiom
COPY config.toml ./
COPY grafana/ ./grafana/

# Dashboard + Metrics
EXPOSE 8080 9090

# eBPF needs privileged — document it clearly
LABEL io.nexus-axiom.requires-privileged="true"
LABEL io.nexus-axiom.requires-host-pid="true"
LABEL org.opencontainers.image.source="https://github.com/CoderAwesomeAbhi/nexus-axiom"
LABEL org.opencontainers.image.description="eBPF security that blocks exploits using LSM hooks"

CMD ["/nexus-axiom/nexus-axiom", "start"]
