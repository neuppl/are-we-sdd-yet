FROM are-we-sdd-yet/sdd as sddbuilder

FROM are-we-sdd-yet/rsdd as rsddbuilder

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
RUN cargo install --bin are_we_sdd_yet --path .


FROM debian:bullseye-slim

WORKDIR /usr/src

COPY --from=sddbuilder /usr/src/sdd /usr/src/sdd
COPY --from=rsddbuilder /usr/src/rsdd /usr/src/rsdd

COPY --from=builder /usr/local/cargo/bin/are_we_sdd_yet /usr/local/bin/are_we_sdd_yet

COPY fixtures /usr/src/fixtures

# RUN are_we_sdd_yet -f fixtures/cnf/cm152a.cnf
# COPY --from=builder /usr/src/fixtures /usr/fixtures
# CMD ["/usr/rsdd", "-f", "fixtures/cnf/cm152a.cnf" "-m", "sdd_dtree_minfill"]
# rsdd -f fixtures/cnf/cm152a.cnf -m sdd_dtree_minfill
