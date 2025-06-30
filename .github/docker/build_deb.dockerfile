FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV LANG=C.UTF-8
ENV LC_ALL=C.UTF-8
ENV PATH=/root/.cargo/bin:$PATH

# Installer dependencies til WebKitGTK og Rust build
RUN apt-get update && apt-get install -y --no-install-recommends \
    software-properties-common \
    curl \
    build-essential \
    ninja-build \
    meson \
    pkg-config \
    git \
    ca-certificates \
    libglib2.0-dev \
    libgtk-4-dev \
    libgdk-pixbuf-2.0-dev \
    libpango1.0-dev \
    libjpeg-dev \
    libpng-dev \
    libharfbuzz-dev \
    libxslt1-dev \
    libicu-dev \
    libwebp-dev \
    libsecret-1-dev \
    libepoxy-dev \
    libenchant-2-dev \
    libcurl4-openssl-dev \
    libxml2-dev \
    libsqlite3-dev \
    libxt-dev \
    libnotify-dev \
    libxcomposite-dev \
    libxdamage-dev \
    libxrandr-dev \
    libatk1.0-dev \
    libatk-bridge2.0-dev \
    libegl1-mesa-dev \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    libxtst-dev \
    libwayland-dev \
    libxml2-utils \
    python3 \
    python3-pip \
    python3-setuptools \
    python3-cffi \
 && rm -rf /var/lib/apt/lists/*

# Installer Rust (stable)
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Clone WebKitGTK 6.x (seneste stable tag, fx 2.60 eller 2.66, opdater efter behov)
RUN git clone --depth 1 https://github.com/WebKit/WebKit.git /webkitgtk
WORKDIR /webkitgtk

# Check ud ønsket version - fx den seneste stabile branch/tags:
RUN git checkout 2.66.4

# Build WebKitGTK med meson og ninja
RUN python3 Tools/Scripts/update-webkitgtk-libs.py  # installer webkitgtk dependencies
RUN meson build --prefix=/usr -Dgtk4=true -Dport=gtk -Dmini-gtk=true
RUN ninja -C build
RUN ninja -C build install

# Sæt library path
ENV PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig
ENV LD_LIBRARY_PATH=/usr/lib

# Byg din Rust-app (kopiér kode ind i containeren)
WORKDIR /app
COPY . /app

RUN source $HOME/.cargo/env && cargo build --release

CMD ["./target/release/marco"]
