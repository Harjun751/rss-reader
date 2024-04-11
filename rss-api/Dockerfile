FROM rust:1.67 as builder
WORKDIR /usr/src/app
COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /usr/local/bin
COPY ssl/cert.pem ssl/cert.pem
COPY ssl/key.pem ssl/key.pem
COPY --from=builder /usr/src/app/target/release/rss-api /usr/local/bin/rss-api
RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
RUN apt-get update
RUN apt-get -y install ca-certificates
RUN touch /usr/local/bin/error_log.xml
RUN printf "<root>\n</root>" >> error_log.xml
CMD ["rss-api"]