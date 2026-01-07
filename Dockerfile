FROM alpine:latest

# Install rust and SDL2
RUN apk update ; apk upgrade
RUN apk add --no-cache rust cargo build-base sdl2-dev git cmake=3.31.7-r0

WORKDIR /usr/src/rbgb
COPY . .

# Build all targets to verify compilation
RUN cargo build --all-targets

CMD ["cargo", "test"]
