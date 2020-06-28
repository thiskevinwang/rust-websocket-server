# https://www.fpcomplete.com/blog/2018/07/deploying-rust-with-docker-and-kubernetes/

# Mini docker
# https://github.com/kpcyrd/mini-docker-rust/blob/master/.dockerignore

# Start with a rust alpine image
FROM rust:1.44.1-alpine AS build
RUN apk add --update cargo

# if needed, install dependencies here
#RUN apk add libseccomp-dev
# set the workdir and copy the source into it
WORKDIR /app
COPY ./ /app
# do a release build
RUN cargo build --release

# use a plain alpine image, the alpine version needs to match the builder
FROM alpine:3.11
# if needed, install dependencies here
#RUN apk add libseccomp
# copy the binary into the final image
COPY --from=0 /app/target/release/rust-redis-docker .
# set the binary as entrypoint

EXPOSE 3000/tcp
ENTRYPOINT ["/rust-redis-docker"]