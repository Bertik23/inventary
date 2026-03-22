# Build stage
FROM rust:1.83-slim-bullseye as builder

# Install trunk and wasm-target
RUN apt-get update && apt-get install -y pkg-config libssl-dev && \
    cargo install trunk && \
    rustup target add wasm32-unknown-unknown

WORKDIR /app
COPY . .
WORKDIR /app/frontend
RUN trunk build --release

# Run stage
FROM nginx:alpine

COPY --from=builder /app/frontend/dist /usr/share/nginx/html
# Custom nginx config to handle SPA routing if needed
COPY frontend/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
