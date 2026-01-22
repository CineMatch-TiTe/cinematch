FROM rust:alpine AS builder

# swagger ui needs curl at comptime
RUN apk add --no-cache curl 

COPY . /usr/src/cinematch
WORKDIR /usr/src/cinematch

RUN cargo install --path cinematch-server

FROM scratch
COPY --from=builder /usr/local/cargo/bin/cinematch-server /usr/local/bin/cinematch-server

EXPOSE 8080
CMD ["cinematch-server"]