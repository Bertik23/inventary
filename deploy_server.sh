#!/bin/bash

# Server Deployment Script for Inventory Management System

echo "--- Deploying Inventory Management System ---"

# Check for --prebuilt flag
USE_PREBUILT=false
for arg in "$@"; do
    if [ "$arg" == "--prebuilt" ]; then
        USE_PREBUILT=true
    fi
done

# Check if docker-compose is installed
if ! command -v docker-compose &> /dev/null
then
    echo "Error: docker-compose is not installed."
    exit 1
fi

# Create data directory if it doesn't exist
mkdir -p ./data

# Copy example environment file if it doesn't exist
if [ ! -f .env ]; then
    echo "Warning: .env file not found. Creating one from backend/.env.example..."
    cp backend/.env.example .env
fi

if [ "$USE_PREBUILT" = true ]; then
    echo "Using prebuilt images from GHCR..."
    COMPOSE_FILE="docker-compose.prebuilt.yml"
    docker-compose -f $COMPOSE_FILE pull
    docker-compose -f $COMPOSE_FILE up -d
else
    echo "Building and starting containers from source..."
    COMPOSE_FILE="docker-compose.prod.yml"
    docker-compose -f $COMPOSE_FILE up --build -d
fi

echo ""
echo "--- Deployment Complete ---"
echo "The system should now be available on port 8080."
echo "Check logs with: docker-compose -f $COMPOSE_FILE logs -f"
echo ""
echo "Note: For production, you should set up a reverse proxy with SSL (e.g., Nginx, Caddy)."
