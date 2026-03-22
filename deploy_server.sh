#!/bin/bash

# Server Deployment Script for Inventory Management System

echo "--- Deploying Inventory Management System ---"

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

# Build and start the system in the background
echo "Building and starting containers using docker-compose.prod.yml..."
docker-compose -f docker-compose.prod.yml up --build -d

echo ""
echo "--- Deployment Complete ---"
echo "The system should now be available on port 8080."
echo "Check logs with: docker-compose -f docker-compose.prod.yml logs -f"
echo ""
echo "Note: For production, you should set up a reverse proxy with SSL (e.g., Nginx, Caddy)."
