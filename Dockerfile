FROM alpine:3.16
RUN apk add --no-cache qemu-system-x86_64 binutils cargo rust rustup
RUN rustup-init -y --no-modify-path --default-toolchain nightly
RUN cargo install --no-default-features --force cargo-make
WORKDIR /buzz