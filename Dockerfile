FROM ghcr.io/rust-cross/cargo-zigbuild:latest@sha256:a23677e0d80ae532cc6a132dcd73a85949a909fd4efe13d642ccf32106532cee AS build
ARG TARGETPLATFORM

RUN apt-get update && apt-get install -y --no-install-recommends \
  ca-certificates && \
  rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock /source/
COPY src /source/src
WORKDIR /source

# Set required args for build
ENV CARGO_TARGET_DIR='./target'
ENV CARGO_TERM_COLOR=always

RUN <<EOF
  # Building for $TARGETPLATFORM
  case "$TARGETPLATFORM" in
    *"linux/amd64"*)
      cargo zigbuild --release --target x86_64-unknown-linux-musl
      mv ./target/x86_64-unknown-linux-musl/release/sm-action ./target/release/sm-action
      ;;
    *"linux/arm64"*)
      cargo zigbuild --release --target aarch64-unknown-linux-musl
      mv ./target/aarch64-unknown-linux-musl/release/sm-action ./target/release/sm-action
      ;;
    *)
      echo "Unsupported target platform: $TARGETPLATFORM"
      exit 1
      ;;
  esac
EOF

FROM scratch AS final
COPY --from=build /source/target/release/sm-action /bin/sm-action
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

FROM final
# USER 65534:65534 # we need permission to write to GITHUB_ENV and GITHUB_OUTPUT, so must be root
ENTRYPOINT ["/bin/sm-action"]
