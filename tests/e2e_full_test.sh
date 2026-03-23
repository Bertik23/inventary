#!/bin/bash
set -e

# E2E Full Feature Test for Inventary API

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${YELLOW}Starting E2E Full Feature Test...${NC}"

# 0. Setup temp DB
TEST_DB="/tmp/inventary_test_$(date +%s).db"
export DATABASE_URL="sqlite://$TEST_DB"
export PORT=8089
API_BASE="http://127.0.0.1:8089/api"

# 1. Start backend in background
echo "Starting backend with DB: $TEST_DB"
cd backend
cargo run &
BACKEND_PID=$!

# Cleanup on exit
trap "kill $BACKEND_PID; rm -f $TEST_DB" EXIT

# Wait for backend to start
echo "Waiting for backend to start..."
for i in {1..20}; do
    if curl -s "http://127.0.0.1:8089/api/product/nonexistent" > /dev/null; then
        echo "Backend is up!"
        break
    fi
    if [ $i -eq 20 ]; then
        echo -e "${RED}Backend failed to start in time${NC}"
        exit 1
    fi
    sleep 1
done

# 2. Test Registration
echo -n "Testing user registration... "
REG_RESP=$(curl -s -X POST "$API_BASE/users/register" \
  -H "Content-Type: application/json" \
  -d '{"username":"full_test_user","email":"full@test.com","password":"password123"}')

USER_ID=$(echo "$REG_RESP" | grep -oP '"id":"\K[^"]+')
if [ -n "$USER_ID" ]; then
    echo -e "${GREEN}PASS (ID: $USER_ID)${NC}"
else
    echo -e "${RED}FAIL${NC}"
    echo "Response: $REG_RESP"
    exit 1
fi

# 3. Create Inventory
echo -n "Creating inventory... "
INV_RESP=$(curl -s -X POST "$API_BASE/inventories" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Test Home\",\"owner_id\":\"$USER_ID\"}")

INV_ID=$(echo "$INV_RESP" | grep -oP '"id":"\K[^"]+')
if [ -n "$INV_ID" ]; then
    echo -e "${GREEN}PASS (ID: $INV_ID)${NC}"
else
    echo -e "${RED}FAIL${NC}"
    echo "Response: $INV_RESP"
    exit 1
fi

# 4. Add Item to Inventory
echo -n "Adding item 'Apples'... "
ADD_RESP=$(curl -s -X POST "$API_BASE/inventory/add" \
  -H "Content-Type: application/json" \
  -d "{\"inventory_id\":\"$INV_ID\",\"name\":\"Apples\",\"quantity\":5,\"unit\":\"kg\"}")

ITEM_ID=$(echo "$ADD_RESP" | grep -oP '"id":"\K[^"]+')
if [ -n "$ITEM_ID" ]; then
    echo -e "${GREEN}PASS (ID: $ITEM_ID)${NC}"
else
    echo -e "${RED}FAIL${NC}"
    echo "Response: $ADD_RESP"
    exit 1
fi

# 5. List Inventory
echo -n "Listing inventory... "
LIST_RESP=$(curl -s -X GET "$API_BASE/inventory?inventory_id=$INV_ID")
if echo "$LIST_RESP" | grep -q "Apples"; then
    echo -e "${GREEN}PASS${NC}"
else
    echo -e "${RED}FAIL${NC}"
    echo "Response: $LIST_RESP"
    exit 1
fi

# 6. Create Category
echo -n "Creating category 'Fruit'... "
CAT_RESP=$(curl -s -X POST "$API_BASE/inventories/$INV_ID/categories" \
  -H "Content-Type: application/json" \
  -d '{"name":"Fruit"}')

CAT_ID=$(echo "$CAT_RESP" | grep -oP '"id":"\K[^"]+')
if [ -n "$CAT_ID" ]; then
    echo -e "${GREEN}PASS (ID: $CAT_ID)${NC}"
else
    echo -e "${RED}FAIL${NC}"
    echo "Response: $CAT_RESP"
    exit 1
fi

# 7. Update Item with Category
echo -n "Updating item with category... "
UPDATE_RESP=$(curl -s -X PUT "$API_BASE/inventory/items/$ITEM_ID" \
  -H "Content-Type: application/json" \
  -d "{\"categories\":[\"$CAT_ID\"]}")

if echo "$UPDATE_RESP" | grep -q "$CAT_ID"; then
    echo -e "${GREEN}PASS${NC}"
else
    echo -e "${RED}FAIL${NC}"
    echo "Response: $UPDATE_RESP"
    exit 1
fi

# 8. Search Inventory
echo -n "Searching inventory for 'App'... "
SEARCH_RESP=$(curl -s -X GET "$API_BASE/inventory/search?inventory_id=$INV_ID&q=App")
if echo "$SEARCH_RESP" | grep -q "Apples"; then
    echo -e "${GREEN}PASS${NC}"
else
    echo -e "${RED}FAIL${NC}"
    echo "Response: $SEARCH_RESP"
    exit 1
fi

# 9. Buffer Unknown Product
echo -n "Buffering unknown product... "
BUFF_RESP=$(curl -s -X POST "$API_BASE/products/buffer" \
  -H "Content-Type: application/json" \
  -d "{\"barcode\":\"E2E-123\",\"name\":\"Unknown Soda\",\"added_by\":\"$USER_ID\"}")

if echo "$BUFF_RESP" | grep -q "Unknown Soda"; then
    echo -e "${GREEN}PASS${NC}"
else
    echo -e "${RED}FAIL${NC}"
    echo "Response: $BUFF_RESP"
    exit 1
fi

echo -e "\n${GREEN}All E2E features tested successfully!${NC}"
