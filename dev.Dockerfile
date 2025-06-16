FROM rust:latest
EXPOSE 3000 

WORKDIR /app
COPY Cargo.toml .
RUN rustup default nightly 
RUN cargo install cargo-watch --locked
RUN cargo install sqlx-cli --locked

ENTRYPOINT cargo watch -c -w src -w static -w templates -x 'run --bin=tis100-pad start'

