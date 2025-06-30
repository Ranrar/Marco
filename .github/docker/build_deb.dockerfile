FROM ubuntu:24.04

RUN apt-get update && apt-get install -y \
    software-properties-common \
    curl \
    build-essential \
    pkg-config \
    libgtk-4-dev \
    libglib2.0-dev \
    libgdk-pixbuf-2.0-dev \
    libpango1.0-dev \
    libsourceview5-dev \
    libwebkit2gtk-6.0-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Rust (rustup)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

COPY . .

RUN cargo build --release

CMD ["./target/release/marco"]
