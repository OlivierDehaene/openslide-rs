FROM debian:stable
RUN apt update
RUN apt install -y curl
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > install_rustup.sh && sh install_rustup.sh -y
RUN apt install -y libcairo2-dev libgdk-pixbuf2.0-dev libglib2.0-dev libjpeg62-turbo-dev libopenjp2-7-dev libpng-dev libsqlite3-dev libtiff5-dev libxml2-dev libwebp-dev libzstd-dev
RUN apt install -y build-essential
COPY . openslide/
RUN ~/.cargo/bin/cargo build --manifest-path openslide/Cargo.toml
RUN ~/.cargo/bin/cargo test --manifest-path openslide/Cargo.toml