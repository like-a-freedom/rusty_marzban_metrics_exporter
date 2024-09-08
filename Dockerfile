# Got an error cross-compiling x86_64 on my apple silicon, but it is compiles normally on native amd64
# If you want to cross-compile, try compiling on messense/cargo-zigbuild - https://github.com/rust-cross/cargo-zigbuild
# Like cargo zigbuild --release --target for x86_64-unknown-linux-musl

FROM --platform=$BUILDPLATFORM rust:slim AS builder
RUN apt update && apt install -y --no-install-recommends \
    musl-tools musl-dev clang llvm perl cmake\
    &&rm -rf /var/lib/apt/lists/*
RUN update-ca-certificates
WORKDIR /app
COPY . .
ARG TARGETARCH TARGETPLATFORM
RUN echo "Building for ${TARGETARCH} on ${TARGETPLATFORM}"
RUN if [ "${TARGETARCH}" = "arm64" ]; then \
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-musl-gcc \
    && export CC=aarch64-linux-musl-gcc \
    && export RUSTFLAGS="-C target-feature=+crt-static" \
    && rustup target add aarch64-unknown-linux-musl \
    && cargo build --release --target aarch64-unknown-linux-musl \
    && mv ./target/aarch64-unknown-linux-musl/release/marzban_exporter . \
    && strip marzban_exporter; \
    else \
    export CC_x86_64_unknown_linux_musl=clang \
    && rustup target add x86_64-unknown-linux-musl \
    && cargo build --release --target x86_64-unknown-linux-musl \
    && mv ./target/x86_64-unknown-linux-musl/release/marzban_exporter . \
    && strip marzban_exporter; \
    fi

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/marzban_exporter /usr/local/bin/marzban_exporter
EXPOSE 8050
USER 1000:1000
ENTRYPOINT ["/usr/local/bin/marzban_exporter"]

# [target.aarch64-unknown-linux-musl]
# linker = "aarch64-linux-musl-gcc"
# rustflags = ["-C", "target-feature=-crt-static"]