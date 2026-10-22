# syntax=docker/dockerfile:1
# Multi-arch Debian 13.1 image; the reference peer versions are fixed below.
FROM debian:13.1-slim@sha256:a347fd7510ee31a84387619a492ad6c8eb0af2f2682b916ff3e643eb076f925a AS peers
ARG TARGETARCH
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && printf 'deb http://deb.debian.org/debian bookworm main\n' > /etc/apt/sources.list.d/barrier.list \
    && apt-get update && mkdir /peers && cd /peers \
    && apt-get download barrier=2.4.0+dfsg-3 \
    && dpkg-deb -x barrier_*.deb /barrier \
    && case "$TARGETARCH" in \
         arm64) arch=aarch64; hash=4780856cca83dc8bcead0495584301d4f83f87f53341bdf447f0915b7b15c596 ;; \
         amd64) arch=x86_64; hash=1f7de2f1fbc68de410339dbdb11ed0e5439bd49bfbfbf66f4467def1c6bb5f96 ;; \
         *) echo 'Only arm64 and amd64 release packages are supported' >&2; exit 1 ;; \
       esac \
    && curl -fL --retry 3 "https://github.com/deskflow/deskflow/releases/download/v1.27.0/deskflow-1.27.0-debian-trixie-${arch}.deb" -o deskflow.deb \
    && printf '%s  deskflow.deb\n' "$hash" | sha256sum -c -

FROM debian:13.1-slim@sha256:a347fd7510ee31a84387619a492ad6c8eb0af2f2682b916ff3e643eb076f925a
COPY --from=peers /peers/deskflow.deb /tmp/deskflow.deb
# Use the distro's unmodified headless Barrier binaries. Avoid the Qt5 GUI stack.
COPY --from=peers /barrier/usr/bin/barriers /barrier/usr/bin/barrierc /usr/local/bin/
COPY --from=peers /barrier/usr/share/doc/barrier/copyright /usr/share/doc/barrier/copyright
RUN apt-get update && apt-get install -y --no-install-recommends \
      /tmp/deskflow.deb xvfb xdotool x11-utils x11-xserver-utils xclip xsel xinput \
      openssl ca-certificates iproute2 procps python3 tcpdump tini \
    && rm -f /tmp/deskflow.deb && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /lab /run/lab && chmod 700 /run/lab
COPY container.sh /usr/local/bin/lab-peer
COPY session.sh /usr/local/bin/lab-session
RUN chmod 755 /usr/local/bin/lab-peer /usr/local/bin/lab-session
ENV DISPLAY=:99 QT_QPA_PLATFORM=xcb XDG_RUNTIME_DIR=/run/lab LC_ALL=C.UTF-8
WORKDIR /lab
ENTRYPOINT ["/usr/bin/tini", "-g", "--", "/usr/local/bin/lab-session"]
