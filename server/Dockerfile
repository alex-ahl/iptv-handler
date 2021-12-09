# Rust as the base image
FROM rust:1.57 as build

# Create a new empty shell project
RUN USER=root cargo new --bin iptv-proxy
WORKDIR /iptv-proxy

# Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# Copy the source code
COPY ./src ./src

# Build for release.
RUN rm ./target/release/deps/iptv_proxy*
RUN cargo build --release

# The final base image
FROM debian:buster-slim

# Copy from the previous build
COPY --from=build /iptv-proxy/target/release/iptv-proxy /usr/src/iptv-proxy

# Run the binary
CMD ["/usr/src/iptv-proxy"]