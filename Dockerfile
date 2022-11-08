# Alpine Linux Image
FROM rustlang/rust:nightly-alpine3.12

# Install Rust and other dependencies
RUN apk add curl qemu-system-x86_64 binutils cargo nasm grub grub-bios xorriso

# Install Rust nightly and target for musl 
RUN rustup toolchain install nightly 
RUN rustup default nightly
RUN rustup component add rust-src

# Instal cargo make
RUN cargo install --no-default-features --force cargo-make

WORKDIR /buzz