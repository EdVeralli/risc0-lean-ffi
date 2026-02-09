FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV HOME=/root
ENV PATH="${HOME}/.cargo/bin:${HOME}/.elan/bin:${HOME}/.risc0/bin:${PATH}"
ENV LD_LIBRARY_PATH="${HOME}/.elan/toolchains/leanprover--lean4---v4.26.0/lib/lean"

# Dependencias del sistema
RUN apt-get update && apt-get install -y \
    curl git build-essential pkg-config libssl-dev \
    cmake clang lld \
    libgmp-dev libuv1-dev libc++-dev libc++abi-dev \
    && rm -rf /var/lib/apt/lists/*

# Instalar Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Instalar elan (Lean 4)
RUN curl -sSf https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh | sh -s -- -y --default-toolchain leanprover/lean4:v4.26.0

# Instalar RISC Zero toolchain
RUN curl -L https://risczero.com/install | bash \
    && rzup install cargo-risczero 1.2.6 \
    && rzup install r0vm 1.2.6 \
    && rzup install rust \
    && rzup install cpp

# Copiar proyecto
WORKDIR /app
COPY . .

# 1. Compilar Lean
RUN cd lean_verifier && lake build

# 2. Compilar Rust + RISC Zero
RUN cargo build --release

# Ejecutar
CMD ["cargo", "run", "--release"]
