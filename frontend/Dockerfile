# Build stage
FROM rust:1.85-slim-bullseye as builder

# Install trunk and wasm-target
RUN apt-get update && apt-get install -y pkg-config libssl-dev curl && \
    rustup target add wasm32-unknown-unknown && \
    curl -L https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- -C /usr/local/bin

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
