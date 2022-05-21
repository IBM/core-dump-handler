FROM docker.io/alpine:3.15.4 as builder

ARG TARGETARCH

RUN apk update && apk add curl binutils build-base

RUN if [ $TARGETARCH == "amd64" ]; then curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable-x86_64-unknown-linux-musl -y; fi

RUN if [ $TARGETARCH == "arm64" ]; then curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable-aarch64-unknown-linux-musl -y; fi

RUN ls -a /root/.cargo/bin

COPY . /app-build

WORKDIR "/app-build"

ENV PATH=/root/.cargo/bin:${PATH}
RUN cargo build --release

RUN curl -L https://github.com/kubernetes-sigs/cri-tools/releases/download/v1.22.0/crictl-v1.22.0-linux-$TARGETARCH.tar.gz --output crictl-v1.22.0-linux-$TARGETARCH.tar.gz
RUN tar zxvf crictl-v1.22.0-linux-$TARGETARCH.tar.gz

FROM docker.io/alpine:3.15.4

RUN apk update && apk add procps

WORKDIR "/app"
COPY --from=builder /app-build/target/release/core-dump-agent ./
WORKDIR "/app/vendor/default"
COPY --from=builder /app-build/target/release/core-dump-composer ./
RUN mv core-dump-composer cdc

WORKDIR "/app"
COPY --from=builder /app-build/crictl ./
CMD ["./core-dump-agent"]