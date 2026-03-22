#!/bin/bash
set -e

# E2E Smoke Test for Inventary API

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo "Starting E2E Smoke Test..."

# 1. Start backend in background
cd backend
cargo run &
BACKEND_PID=$!

# Wait for backend to start
echo "Waiting for backend to start..."
sleep 10

# Cleanup on exit
trap "kill $BACKEND_PID" EXIT

API_BASE="http://127.0.0.1:8080/api"

# 2. Test Registration
echo -n "Testing user registration... "
REG_RESP=$(curl -s -X POST "$API_BASE/users/register" \
  -H "Content-Type: application/json" \
  -d '{"username":"e2e_user","email":"e2e@test.com","password":"password123"}')

if echo "$REG_RESP" | grep -q "e2e_user"; then
    echo -e "${GREEN}PASS${NC}"
else
    echo -e "${RED}FAIL${NC}"
    echo "Response: $REG_RESP"
    exit 1
fi

# 3. Test Login
echo -n "Testing user login... "
LOGIN_RESP=$(curl -s -X POST "$API_BASE/users/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"e2e_user","password":"password123"}')

if echo "$LOGIN_RESP" | grep -q "e2e_user"; then
    echo -e "${GREEN}PASS${NC}"
else
    echo -e "${RED}FAIL${NC}"
    echo "Response: $LOGIN_RESP"
    exit 1
fi

# 4. Test Product Lookup (expect 404 for random barcode)
echo -n "Testing product lookup (not found)... "
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$API_BASE/product/1234567890123")

if [ "$HTTP_CODE" == "404" ]; then
    echo -e "${GREEN}PASS${NC}"
else
    echo -e "${RED}FAIL${NC} (Got $HTTP_CODE)"
    exit 1
fi

echo -e "\n${GREEN}All smoke tests passed!${NC}"
