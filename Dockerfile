# Dockerfile
FROM rust:1.70

WORKDIR /app
COPY . .

RUN cargo build --release

EXPOSE 3030
CMD ["./target/release/layer1"]