FROM ngraf3/rbgb

WORKDIR /usr/src/rbgb
COPY . .

# Build all targets to verify compilation
RUN cargo build --all-targets

CMD ["cargo", "test"]
