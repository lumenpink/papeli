FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app

# Prepara cache de dependências
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN cargo build --release || true

# Copia código real e compila release final
COPY src ./src
COPY tests ./tests
RUN touch src/main.rs src/lib.rs && cargo build --release

# Imagem de execução mínima
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /etc/papeli/templates

COPY --from=builder /app/target/release/papeli /usr/local/bin/papeli
RUN chmod +x /usr/local/bin/papeli

EXPOSE 8899

CMD ["/usr/local/bin/papeli", "--config", "/etc/papeli/papeli.toml"]
