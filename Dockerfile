FROM rust:1.66.0-buster AS BUILD
# Install software
RUN update-ca-certificates && apt-get update && apt-get install -y libsasl2-dev

# Create appuser
ENV USER=scylla
ENV UID=1000
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /tmp
COPY ./ .
# Build binary in release mode
RUN cargo build --release --bin pg_monitor

#
# Run image based on buster-slim to reduce image size while still using glibc
#
FROM debian:buster-slim AS RUN

WORKDIR /opt/build
# Import users from build
COPY --from=BUILD /etc/passwd /etc/passwd
COPY --from=BUILD /etc/group /etc/group
# Copy binary from build
COPY --from=BUILD /tmp/target/release/pg_monitor ./
# Use an unprivileged user
USER ${USER}:${USER}
# Entry point
CMD ["/opt/build/pg_monitor"]
