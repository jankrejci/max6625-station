FROM debian:bullseye

RUN apt update && apt install -y curl ssh gcc-aarch64-linux-gnu build-essential

RUN mkdir /project
ARG UID
ARG GID
RUN groupadd -g $GID -o pi && \
    useradd -u $UID -g $GID -m pi && \
    usermod -aG sudo pi && \
    echo "pi:pi" | chpasswd

USER pi

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
# For some reason, $HOME is not /home/pi
ENV PATH=/home/pi/.cargo/bin:$PATH
RUN rustup target add aarch64-unknown-linux-gnu

ENV SSH_AUTH_SOCK=/ssh-agent

WORKDIR /project
