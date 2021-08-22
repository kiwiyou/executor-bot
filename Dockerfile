FROM rust:slim-bullseye as builder
RUN USER=root cargo new --bin executor
WORKDIR /executor
COPY ./Cargo.toml ./Cargo.lock ./
RUN apt-get update && apt-get -y install libssl-dev pkg-config && cargo build --release && rm src/*.rs
COPY ./src ./src
RUN rm ./target/release/deps/executor* && cargo build --release

FROM debian:bullseye-slim
RUN export DEBIAN_FRONTEND=noninteractive && \
    apt-get update && \
    apt-get -y upgrade && \
    apt-get -y install --no-install-recommends build-essential g++ gcc python3 \
    curl openssl ca-certificates ghc llvm nodejs bash golang-go default-jdk && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    # rustc is in $HOME
    ln -s $HOME/.cargo/bin/rustc /bin/rustc
COPY --from=builder /executor/target/release/executor /executor
ENTRYPOINT ["/executor"]
