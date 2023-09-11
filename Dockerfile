# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------
FROM rust:1.70.0-slim as builder

WORKDIR /usr/src/templating

# Create blank project
RUN USER=root cargo new templating
RUN apt update && apt upgrade -y
RUN apt install musl-tools protobuf-compiler -y

## Install target platform (Cross-Compilation) --> Needed for Alpine
RUN rustup target add x86_64-unknown-linux-musl

# Now copy in the rest of the sources
RUN mkdir -p /usr/src/common
COPY ./common ../common
COPY ./templating/ .


# This is the actual application build.
RUN cargo build --target x86_64-unknown-linux-musl --release

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------
FROM alpine:3.16.0 AS runtime 

# Copy application binary from builder image
COPY --from=builder /usr/src/templating/target/x86_64-unknown-linux-musl/release/templating /usr/local/bin

# Run the application
CMD ["/usr/local/bin/templating"]
