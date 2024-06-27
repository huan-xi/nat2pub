FROM clux/muslrust:stable AS chef
#RUN apt update -y && apt install -y
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN echo '[source.crates-io]' > ~/.cargo/config \
    && echo 'registry = "https://github.com/rust-lang/crates.io-index"'  >> ~/.cargo/config \
    && echo "replace-with = 'tuna'"  >> ~/.cargo/config \
    && echo '[source.tuna]'   >> ~/.cargo/config \
    && echo 'registry = "https://rsproxy.cn/crates.io-index"'  >> ~/.cargo/config \
    && echo '[net]'   >> ~/.cargo/config \
    && echo 'git-fetch-with-cli = true'   >> ~/.cargo/config \
    && echo '' >> ~/.cargo/config

# Notice that we are specifying the --target flag!
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin btnat

FROM alpine AS runtime
WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/btnat /app/btnat
COPY --from=planner /app/config_docker.toml /app/config.toml
#COPY --from=planner /app/docker_config.toml /app/config.toml
#COPY --from=planner /app/dist /app/dist

CMD ["/app/btnat"]