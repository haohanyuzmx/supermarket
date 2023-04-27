FROM rust:latest

WORKDIR /usr/src/supermarket

RUN sed -i "s/deb.debian.org/mirrors.163.com/g" /etc/apt/sources.list\
    &&sed -i "s/security.debian.org/mirrors.163.com/g" /etc/apt/sources.list\
    &&apt update &&apt upgrade -y\
    &&apt install -y protobuf-compiler libprotobuf-dev

# Rustup 清华镜像源
ENV RUSTUP_DIST_SERVER "https://rsproxy.cn"
ENV USTUP_UPDATE_ROOT "https://rsproxy.cn/rustup"

COPY . .

RUN sh ./deploy/mirror.sh\
    &&cargo build --bin login\
    &&cargo build --bin user_data\
    &&cargo build --bin sale\
    &&cargo build --bin comment



