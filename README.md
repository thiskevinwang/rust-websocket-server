<img src="/Rust-Redis-Docker.png">

Topics:

- ⚙️ Rust
- 🟨 Hyper
- 🟥 Redis
- 🐳 Docker

## Prerequisites

### `docker`

[Docker Desktop for Mac](https://hub.docker.com/editions/community/docker-ce-desktop-mac/)

```sh
docker --version
# Docker version 19.03.8, build afacb8b
```

### `rustc` + `rustup` + `cargo`

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

```sh
rustc --version
# rustc 1.44.1 (c7087fe00 2020-06-17)

rustup --version
# rustup 1.21.1 (7832b2ebe 2019-12-20)

cargo version
# cargo 1.44.1 (88ba85757 2020-06-11)
```

### Redis

A local Redis container, running on `localhost:6379`

```sh
docker pull bitnami/redis

# This will eat your terminal
docker run --rm --name redis \
  -e ALLOW_EMPTY_PASSWORD=yes \
  -p 6379:6379 \
  bitnami/redis

# ctrl+c to stop
```

#### Optional

Install the redis-cli

```sh
brew install redis

redis-cli --version
# redis-cli 6.0.5
```

Save your terminal

```sh
# Run docker in detached mode
docker run --rm --name redis \
  -e ALLOW_EMPTY_PASSWORD=yes \
  -d -it -p 6379:6379 bitnami/redis

# Stop the container
docker stop redis
```

Database GUI? Visit [redis://127.0.0.1:6379](redis://127.0.0.1:6379) in a browser. On my machine, I get prompted to open **TablePlus**.

### Postgres

Run a postgres docker container

- https://hub.docker.com/_/postgres

```
docker run --rm --name postboi \
  -e POSTGRES_PASSWORD=mysecretpassword \
  -d -p 8080:5432 postgres:13-alpine
```

Connection string in browser (should open TablePlus, or other app)

`postgres://<POSTGRES_USER>:<POSTGRES_PASSWORD>@<HOST>:<PORT>/<DB>`

`postgres://postgres:mysecretpassword@127.0.0.1:8080/postgres`

## Get started

```sh
# Run with:
cargo run

# Run with hot reloading:
systemfd --no-pid -s http::3000 -- cargo watch -x run

# Find previous task on a port
netstat -vanp tcp | grep 3000
```

## Building Docker Image

```sh
docker build --rm -t rust-redis-docker .
docker images
```

## Running

```sh
docker run --rm \
  --name rrd \
  -p 3000:3000 \
  rust-redis-docker:latest
```

## Helpful things

### Remove Dangling Images

Remove those pesky `<none>` images when building new images

```sh
# fish-shell
docker rmi (docker images -f 'dangling=true' -q)
```

### Remove All Stopped Containers

```sh
# fish-shell
docker rm (docker ps -a -q)
```

### Test POST request to server

```sh
curl -X POST http://localhost:3000/echo/uppercase -d "poop"
```
