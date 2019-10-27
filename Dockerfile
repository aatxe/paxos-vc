FROM rustlang/rust:nightly

RUN cargo install cargo-build-deps

RUN USER=root cargo new --bin app
WORKDIR /app/
COPY Cargo.toml ./
RUN cargo build-deps

ADD hosts /app/
ADD src/ /app/src/
RUN cargo build

ENV RUST_BACKTRACE=1
ENTRYPOINT ["/app/target/debug/prj2"]