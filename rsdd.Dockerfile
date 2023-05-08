# for more on cargo chef, see:
# https://github.com/LukeMathWalker/cargo-chef

FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /usr/src

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR /usr/src
COPY --from=planner /usr/src/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --bin rsdd --release

FROM debian:bullseye-slim

WORKDIR /usr/src

COPY --from=builder /usr/src/target/release/rsdd /usr/src/rsdd
