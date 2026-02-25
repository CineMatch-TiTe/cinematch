FROM rust:alpine AS chef
RUN apk add --no-cache curl musl-dev build-base pkgconfig openssl-dev openssl-libs-static
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Notice that we are specifying the --target flag!
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo install --path cinematch-server --target x86_64-unknown-linux-musl 

FROM scratch
COPY --from=builder /usr/local/cargo/bin/cinematch-server /usr/local/bin/cinematch-server

USER 1000:1000

EXPOSE 8085
CMD ["cinematch-server"]