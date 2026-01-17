#!/bin/bash

# Build script for Inventory Management System

echo "Building backend..."
cd backend
cargo build --release
cd ..

echo "Building frontend..."
cd frontend
trunk build --release
cd ..

echo "Build complete!"
echo "To run the backend: cd backend && cargo run"
echo "To serve the frontend: cd frontend && trunk serve"
