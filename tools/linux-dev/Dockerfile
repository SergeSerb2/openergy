FROM rust:1.99-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends xvfb xdotool xinput xclip x11-utils python3 ca-certificates && rm -rf /var/lib/apt/lists/*
RUN rustup component add clippy rustfmt
ENV DISPLAY=:99 CARGO_TARGET_DIR=/target CARGO_HOME=/cargo
WORKDIR /work
ENTRYPOINT ["/bin/bash", "/src/tools/linux-dev/entrypoint.sh"]
