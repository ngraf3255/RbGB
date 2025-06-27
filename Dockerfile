FROM alpine:latest

# Install rust and SDL2
RUN apk add --no-cache rust cargo build-base sdl2-dev

WORKDIR /usr/src/rbgb
COPY . .

# Build all targets to verify compilation
RUN cargo build --all-targets

CMD ["cargo", "test"]
