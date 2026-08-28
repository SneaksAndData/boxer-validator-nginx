FROM rust:1.96.0-bookworm AS build-env

WORKDIR /app/src
COPY . .
RUN cargo build --release

FROM --platform=$BUILDPLATFORM public.ecr.aws/amazonlinux/amazonlinux:2 AS aws-cli-stage

ARG TARGETARCH
ARG KUBECTL_VERSION="v1.34.0"

RUN yum update -y && yum install -y unzip curl \
    && cli_arch=$(test "$TARGETARCH" = "amd64" && echo "x86_64" || echo "aarch64") \
    && curl "https://awscli.amazonaws.com/awscli-exe-linux-$cli_arch.zip" -o "awscliv2.zip" \
    && unzip awscliv2.zip \
  # The --bin-dir is specified so that we can copy the
  # entire bin directory from the installer stage into
  # into /usr/local/bin of the final stage without
  # accidentally copying over any other executables that
  # may be present in /usr/local/bin of the installer stage.
    && ./aws/install --bin-dir /aws-cli-bin/ \
    && curl -L "https://dl.k8s.io/release/$KUBECTL_VERSION/bin/linux/$TARGETARCH/kubectl" -o /usr/local/bin/kubectl \
    && chmod +x /usr/local/bin/kubectl

FROM debian:bookworm-slim AS build-release-stage

RUN apt-get update -y && \
    apt-get install -y curl netcat-openbsd && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*


WORKDIR /app
COPY --from=build-env /app/src/target/release .
COPY --from=build-env /app/src/settings.toml /app/settings.toml


COPY --from=aws-cli-stage /usr/local/aws-cli/ /usr/local/aws-cli/
COPY --from=aws-cli-stage /aws-cli-bin/ /usr/local/bin/
COPY --from=aws-cli-stage /usr/local/bin/kubectl /usr/local/bin/


USER 65534:65534

CMD ["/app/boxer-validator-nginx"]
