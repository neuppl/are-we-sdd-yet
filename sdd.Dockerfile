FROM buildpack-deps:bullseye as builder

WORKDIR /usr/src

RUN apt-get update && apt-get -y install scons

COPY ./sdd-package-2.0/libsdd-2.0 .

RUN scons

FROM buildpack-deps:bullseye as builder2

WORKDIR /usr/src

COPY ./sdd-package-2.0/sdd-2.0 .
COPY --from=builder /usr/src/build/libsdd.a /usr/src/lib/Linux/libsdd.a

RUN make build/sdd

FROM debian:bullseye-slim

WORKDIR /usr/src

COPY --from=builder2 /usr/src/build/sdd /usr/src/sdd
