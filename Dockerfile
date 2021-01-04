FROM ubuntu

ENV TZ=Europe/Paris
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

# Install Rust
RUN apt update
RUN apt install curl -y
RUN apt install build-essential -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > install_rustup.sh && sh install_rustup.sh -y

# Install dependencies
COPY install_dependencies.sh .
RUN ./install_dependencies.sh

# Test
COPY . openslide-rs/
RUN ~/.cargo/bin/cargo build --manifest-path openslide-rs/Cargo.toml
RUN ~/.cargo/bin/cargo test --manifest-path openslide-rs/Cargo.toml