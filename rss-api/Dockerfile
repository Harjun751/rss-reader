FROM rust:1.67 as builder
WORKDIR /usr/src/app
COPY . .
ENV RSS_API_DB=mysql://root:test@172.18.0.2:3306/rss
ENV RSS_FE=http://localhost
RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /usr/local/bin
ENV RSS_API_DB=mysql://root:test@172.18.0.2:3306/rss
ENV RSS_FE=http://localhost
RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/rss-api /usr/local/bin/rss-api
RUN touch /usr/local/bin/error_log.xml
RUN printf "<root>\n</root>" >> error_log.xml
RUN apt-get update
RUN apt-get -y install ca-certificates
CMD ["rss-api"]
