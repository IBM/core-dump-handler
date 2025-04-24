FROM registry.access.redhat.com/ubi7/ubi AS rhel7builder

RUN yum install -y gcc openssl-devel && \
    rm -rf /var/cache/dnf && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y

COPY . /app-build

WORKDIR "/app-build"

ENV PATH=/root/.cargo/bin:${PATH}

RUN cargo build --release -p core-dump-composer

FROM registry.access.redhat.com/ubi8/ubi AS rhel8builder

RUN yum install -y gcc openssl-devel && \
    rm -rf /var/cache/dnf && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y

COPY . /app-build

WORKDIR "/app-build"

ENV PATH=/root/.cargo/bin:${PATH}

RUN cargo build --release

RUN curl -L https://github.com/kubernetes-sigs/cri-tools/releases/download/v1.22.0/crictl-v1.22.0-linux-amd64.tar.gz --output crictl-v1.22.0-linux-amd64.tar.gz
RUN tar zxvf crictl-v1.22.0-linux-amd64.tar.gz

FROM registry.access.redhat.com/ubi8/ubi-minimal

RUN  microdnf update && microdnf install -y procps-ng

WORKDIR "/app"
COPY --from=rhel8builder /app-build/target/release/core-dump-agent ./
WORKDIR "/app/vendor/default"
COPY --from=rhel8builder /app-build/target/release/core-dump-composer ./
RUN mv core-dump-composer cdc
WORKDIR "/app/vendor/rhel7"
COPY --from=rhel7builder /app-build/target/release/core-dump-composer ./
RUN mv core-dump-composer cdc
WORKDIR "/app"
COPY --from=rhel8builder /app-build/crictl ./
CMD ["./core-dump-agent"]