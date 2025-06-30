FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive LANG=C.UTF-8 LC_ALL=C.UTF-8 PATH=/root/.cargo/bin:$PATH

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential ninja-build meson pkg-config git curl ca-certificates \
    libglib2.0-dev libgtk-4-dev libgdk-pixbuf-2.0-dev libpango1.0-dev \
    libjpeg-dev libpng-dev libharfbuzz-dev libxslt1-dev libicu-dev libwebp-dev \
    libsecret-1-dev libepoxy-dev libcurl4-openssl-dev libxml2-dev libsqlite3-dev \
    libxt-dev libnotify-dev libxcomposite-dev libxdamage-dev libxrandr-dev \
    libatk1.0-dev libatk-bridge2.0-dev libegl1-mesa-dev libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev libxtst-dev libwayland-dev libxml2-utils \
    python3 python3-pip python3-setuptools python3-cffi \
 && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Byg WebKitGTK 6.x
RUN git clone --depth 1 https://github.com/WebKit/WebKit.git /webkitgtk
WORKDIR /webkitgtk/Tools/gtk
RUN ./update-webkitgtk-libs
WORKDIR /webkitgtk
RUN meson build --prefix=/usr -Dgtk4=true -Dport=gtk -Dmini-gtk=true
RUN ninja -C build && ninja -C build install

# Byg din Rust-app
WORKDIR /app
COPY . .
RUN source /root/.cargo/env && \
    cargo install cargo-deb && \
    cargo build --release && \
    cargo deb --no-build

# Standard CMD kan være unødvendig i CI
