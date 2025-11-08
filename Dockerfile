FROM alpine:3.22.2 AS base

LABEL author=Santio
WORKDIR /home/container

FROM rust:1.91.0-alpine AS builder

WORKDIR /home/container
COPY . .

RUN apk add --no-cache build-base musl-dev gcc g++ make libc-dev linux-headers pkgconf openssl-dev openssl-libs-static
RUN cargo build --release

FROM base AS runtime
COPY --from=builder /home/container/target/release/oval ./oval
RUN chmod +x ./oval

CMD ["/home/container/oval"]