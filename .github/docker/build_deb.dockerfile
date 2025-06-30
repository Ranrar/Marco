FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

# Installer byggeafhængigheder
RUN apt-get update && apt-get install -y \
  build-essential \
  cmake \
  ninja-build \
  git \
  python3 \
  python3-pip \
  perl \
  pkg-config \
  libgtk-4-dev \
  libglib2.0-dev \
  libgdk-pixbuf-2.0-dev \
  libpango1.0-dev \
  libsourceview5-dev \
  ca-certificates \
  curl \
  xz-utils \
  libegl1-mesa-dev \
  libgles2-mesa-dev \
  libjpeg-dev \
  libxslt1-dev \
  libwoff-dev \
  libwebp-dev \
  libharfbuzz-dev \
  libwoff2-dev \
  libopus-dev \
  libsecret-1-dev \
  libicu-dev \
  libsqlite3-dev \
  libxml2-utils \
  librest-0.7-dev \
  libjavascriptcoregtk-6.0-dev \
  libxslt1-dev \
  wget \
  && rm -rf /var/lib/apt/lists/*

# Installer meson og ninja via pip (nyeste versioner)
RUN pip3 install meson ninja

# Download WebKitGTK 6.0 kildekode (justér version efter behov)
RUN git clone --branch 6.0.7 https://gitlab.gnome.org/GNOME/webkit.git /webkit

WORKDIR /webkit

# Konfigurer build (tilpas om nødvendigt)
RUN meson _build --prefix=/usr/local --buildtype=release

# Byg og installer
RUN ninja -C _build
RUN ninja -C _build install

# Ryd op i build filer for at spare plads (valgfrit)
RUN rm -rf _build /webkit

# Sæt environment variable hvis nødvendigt
ENV PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH

# Herefter kan du bygge din Rust app baseret på dette image
