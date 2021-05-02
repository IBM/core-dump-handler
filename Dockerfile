FROM docker.io/rust:1.51.0-alpine as builder

COPY . /app-build

WORKDIR "/app-build"

ENV RUSTFLAGS="-C target-feature=-crt-static" 

RUN \
  apk add --no-cache musl-dev openssl-dev && \
  cargo build --release -p core-dump-agent

FROM docker.io/alpine:3.13
RUN apk add --no-cache libgcc
WORKDIR "/app"
COPY --from=builder /app-build/target/release/core-dump-agent ./
RUN mkdir -p vendor/default
WORKDIR "/app/vendor/default"
COPY ./target/release/core-dump-composer ./
RUN mv core-dump-composer cdc
WORKDIR "/app"
CMD ["./core-dump-agent"]