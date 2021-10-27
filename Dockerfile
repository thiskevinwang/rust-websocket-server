# https://www.fpcomplete.com/blog/2018/07/deploying-rust-with-docker-and-kubernetes/

# Mini docker
# https://github.com/kpcyrd/mini-docker-rust/blob/master/.dockerignore

# -----------------
# Cargo Build Stage
# -----------------

# Start with a rust alpine image
FROM rust:1.56.0-alpine AS cargo-build
RUN apk add --update cargo

# if needed, install dependencies here
#RUN apk add libseccomp-dev
# set the workdir and copy the source into it
WORKDIR /app
COPY Cargo.lock /app
COPY Cargo.toml /app
RUN mkdir .cargo
RUN cargo vendor > /app/.cargo/config

COPY ./src src
RUN cargo build --release
RUN cargo install --path . --verbose

# -----------------
# Final Stage
# -----------------

# use a plain alpine image, the alpine version needs to match the builder
FROM alpine:3.11
# if needed, install dependencies here
#RUN apk add libseccomp
# copy the binary into the final image
COPY --from=cargo-build /app/target/release/rust-websocket-server .
# set the binary as entrypoint

ENTRYPOINT ["/rust-websocket-server"]