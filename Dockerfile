FROM rust as builder
WORKDIR /usr/src/power-clash/
COPY Cargo.toml .
COPY Cargo.lock .

# This is a dummy build to get the dependencies cached.
RUN mkdir src \
    && echo "fn main() {}" > dummy.rs \
    && sed -i 's#src/main.rs#dummy.rs#' Cargo.toml \
    && cargo build --release \
    && rm dummy.rs \
    && sed -i 's#dummy.rs#src/main.rs#' Cargo.toml

COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12 as runtime
COPY --from=builder /usr/src/power-clash/target/release/power-clash /power-clash
ENTRYPOINT ["/power-clash"]