FROM docker.io/alpine:3.23.3 AS builder
ARG ARCH
ARG CRICTL_VERSION=1.33.0

RUN apk update && apk add curl binutils build-base openssl-dev openssl-libs-static

RUN if [ $ARCH == "amd64" ]; then curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable-x86_64-unknown-linux-musl -y; fi

RUN if [ $ARCH == "arm64" ]; then curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable-aarch64-unknown-linux-musl -y; fi

RUN ls -a /root/.cargo/bin

COPY . /app-build

WORKDIR "/app-build"

ENV PATH=/root/.cargo/bin:${PATH}
RUN cargo build --verbose --release

RUN curl -L https://github.com/kubernetes-sigs/cri-tools/releases/download/v${CRICTL_VERSION}/crictl-v${CRICTL_VERSION}-linux-$ARCH.tar.gz --output crictl-v${CRICTL_VERSION}-linux-$ARCH.tar.gz
RUN tar zxvf crictl-v${CRICTL_VERSION}-linux-$ARCH.tar.gz

FROM docker.io/alpine:3.23.3

RUN apk update && apk add procps

WORKDIR "/app"
COPY --from=builder /app-build/target/release/core-dump-agent ./
WORKDIR "/app/vendor/default"
COPY --from=builder /app-build/target/release/core-dump-composer ./
RUN mv core-dump-composer cdc

WORKDIR "/app"
COPY --from=builder /app-build/crictl ./
CMD ["./core-dump-agent"]
