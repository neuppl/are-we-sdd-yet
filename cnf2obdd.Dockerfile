FROM buildpack-deps:bullseye as builder

WORKDIR /usr/src

COPY ./cnf2obdd .

RUN make rs

FROM debian:bullseye-slim

WORKDIR /usr/src

COPY --from=builder /usr/src/bdd_minisat_all_static /usr/src/bdd_minisat_all_static
