# Build stage
FROM rust:1.83-slim-bullseye as builder

WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libsqlite3-dev libssl-dev

COPY . .
WORKDIR /app/backend
RUN cargo build --release

# Run stage
FROM debian:bullseye-slim

WORKDIR /app
RUN apt-get update && apt-get install -y libsqlite3-0 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/inventary-backend /app/inventary-backend
COPY --from=builder /app/backend/migrations /app/migrations

# Ensure database directory exists
RUN mkdir -p /app/data

ENV DATABASE_URL=sqlite:///app/data/inventory.db
EXPOSE 8080

CMD ["/app/inventary-backend"]
