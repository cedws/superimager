FROM ubuntu:latest

RUN apt-get update -yq
RUN apt-get install -yq libssl-dev

COPY ./target/release/superimager /root

ENTRYPOINT ["/root/superimager"]
