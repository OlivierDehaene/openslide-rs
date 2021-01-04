FROM ubuntu


# Install Rust
RUN apt update
RUN apt install curl -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > install_rustup.sh && sh install_rustup.sh -y

# Install dependencies
COPY install_dependencies.sh .
RUN ./install_dependencies.sh

# Test
COPY . openslide-rs/
RUN ~/.cargo/bin/cargo build -m openslide-rs/Cargo.toml
RUN ~/.cargo/bin/cargo test -m openslide-rs/Cargo.toml