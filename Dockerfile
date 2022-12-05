# Builder
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

ENV USER root

WORKDIR /pokemon_task
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

RUN cargo build --target x86_64-unknown-linux-musl --release

# Final image
FROM scratch

COPY --from=builder /pokemon_task/target/x86_64-unknown-linux-musl/release/pokemon_task /pokemon_task

CMD ["/pokemon_task"]
