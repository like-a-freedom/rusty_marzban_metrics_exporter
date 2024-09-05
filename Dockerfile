FROM --platform=$BUILDPLATFORM rust:slim AS builder
RUN apt update && apt install -y perl pkg-config make libssl-dev
WORKDIR /app
COPY . .
ARG TARGETARCH TARGETPLATFORM 
RUN echo "Building for ${TARGETARCH} on ${TARGETPLATFORM}" 
RUN if [ "${TARGETARCH}" = "arm64" ]; then \
    apt install -y gcc-aarch64-linux-gnu binutils-aarch64-linux-gnu \
    && rustup target add aarch64-unknown-linux-gnu \
    && mkdir -p .cargo \
    && echo '[target.aarch64-unknown-linux-gnu]\nlinker = "aarch64-linux-gnu-gcc"' > .cargo/config.toml \
    && cargo build --release --target aarch64-unknown-linux-gnu \
    && mv ./target/aarch64-unknown-linux-gnu/release/marzban_exporter . \
    && aarch64-linux-gnu-strip marzban_exporter; \
    else \
    rustup target add x86_64-unknown-linux-gnu \
    && cargo build --release --target x86_64-unknown-linux-gnu \
    && mv ./target/x86_64-unknown-linux-gnu/release/marzban_exporter . \
    && strip marzban_exporter; \
    fi

FROM --platform=$TARGETPLATFORM gcr.io/distroless/cc-debian12
COPY --from=builder /app/marzban_exporter /usr/local/bin/marzban_exporter
EXPOSE 8050
USER 1000:1000
ENTRYPOINT ["/usr/local/bin/marzban_exporter"]