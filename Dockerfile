FROM rust:alpine AS builder
RUN apk add --update gcc g++ build-base alpine-sdk sqlite-dev
WORKDIR /app
COPY . /app
RUN cargo build --release

FROM scratch AS chartered-git
WORKDIR /app
COPY --from=builder /app/target/release/chartered-git /app/chartered-git
ENV RUST_LOG=debug
ENTRYPOINT ["/app/chartered-git"]

FROM scratch AS chartered-web
WORKDIR /app
COPY --from=builder /app/target/release/chartered-web /app/chartered-web
ENV RUST_LOG=debug
ENTRYPOINT ["/app/chartered-web"]
