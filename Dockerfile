# Alpine Linux Image
FROM rustlang/rust:nightly-alpine3.12

# Install Rust dependencies
RUN apk add curl cargo xorriso

# Install Rust nightly and target for musl 
RUN rustup self update
RUN rustup toolchain install nightly 
RUN rustup default nightly
RUN rustup component add rust-src

# Install cargo make
RUN cargo install --no-default-features --force cargo-make

# Install rest of dependencies 
RUN apk add qemu-system-i386 binutils-gold nasm gdb

WORKDIR /buzz