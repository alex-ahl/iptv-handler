# Rust as the base image
FROM rust:1.57 as build

# Create a new empty shell project
RUN USER=root cargo new --bin iptvmanager
WORKDIR /iptvmanager

# Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

COPY ./db ./db
COPY ./api ./api
COPY ./iptv ./iptv
COPY ./rest-client ./rest-client
COPY ./server ./server

# Build only the dependencies to cache them
ENV SQLX_OFFLINE true
RUN cargo build --release

# # Build for release.
RUN rm ./target/release/deps/iptv* && \
    rm ./target/release/deps/db* && \
    rm ./target/release/deps/api* && \
    rm ./target/release/deps/rest* && \
    rm ./target/release/deps/server* 

RUN cargo build --release

# # The final base image
FROM debian:stable-slim
RUN apt-get update && \ 
    apt-get install libcurl4 -y 

# Copy from the previous build
COPY --from=build /iptvmanager/target/release/server /usr/bin/server

# Run the binary
CMD ["/usr/bin/server"]