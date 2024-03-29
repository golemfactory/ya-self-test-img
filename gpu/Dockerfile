FROM rust:1.70.0 as rust_builder
RUN rustup target add x86_64-unknown-linux-musl

FROM rust_builder as builder
WORKDIR /data
COPY . .
RUN cargo install --path . --target x86_64-unknown-linux-musl

FROM ghcr.io/golemfactory/golem_nvidia_base:0.1.0-rc30
VOLUME [ "/mnt/out" ]

RUN DEBIAN_FRONTEND=noninteractive apt-get update
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y \
    libglvnd-dev \
    libx11-6 \
    libxi6 \
    libxxf86vm1 \
    libxrender1 \
    libxfixes3 \
    unzip \
    mesa-utils \
    xorg \
    inetutils-syslogd \
    xserver-xorg-core \
    libxv1 \
    x11-xserver-utils \
    && rm -rf /var/lib/apt/lists/*

RUN cp /usr/lib64/xorg/modules/drivers/nvidia_drv.so /usr/lib/xorg/modules/drivers/
RUN cp /usr/lib64/xorg/modules/extensions/libglxserver_nvidia.so /usr/lib/xorg/modules/extensions/

WORKDIR /
COPY --from=builder /usr/local/cargo/bin/ya-self-test-gpu /ya-self-test-exe
COPY xorg.conf.nvidia-headless /etc/X11/xorg.conf
COPY ya-self-test /ya-self-test

ENTRYPOINT [ "/ya-self-test" ]
