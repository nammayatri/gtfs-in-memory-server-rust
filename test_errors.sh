#!/bin/bash

# Test script to demonstrate improved error messages
BASE_URL="http://localhost:8000"

echo "🧪 Testing Improved Error Messages"
echo "=================================="
echo ""

# Test 1: Non-existent route
echo "📋 Test 1: Non-existent route"
echo "Endpoint: GET /route/invalid_gtfs/invalid_route"
curl -s -w "\nHTTP Status: %{http_code}\n" "$BASE_URL/route/invalid_gtfs/invalid_route" | jq '.'
echo ""

# Test 2: Non-existent GTFS ID
echo "📋 Test 2: Non-existent GTFS ID"
echo "Endpoint: GET /routes/invalid_gtfs"
curl -s -w "\nHTTP Status: %{http_code}\n" "$BASE_URL/routes/invalid_gtfs" | jq '.'
echo ""

# Test 3: Non-existent stop
echo "📋 Test 3: Non-existent stop"
echo "Endpoint: GET /stop/invalid_gtfs/invalid_stop"
curl -s -w "\nHTTP Status: %{http_code}\n" "$BASE_URL/stop/invalid_gtfs/invalid_stop" | jq '.'
echo ""

# Test 4: Non-existent route-stop mapping
echo "📋 Test 4: Non-existent route-stop mapping"
echo "Endpoint: GET /route-stop-mapping/invalid_gtfs/route/invalid_route"
curl -s -w "\nHTTP Status: %{http_code}\n" "$BASE_URL/route-stop-mapping/invalid_gtfs/route/invalid_route" | jq '.'
echo ""

# Test 5: Non-existent provider stop code
echo "📋 Test 5: Non-existent provider stop code"
echo "Endpoint: GET /stop-code/invalid_gtfs/invalid_provider_code"
curl -s -w "\nHTTP Status: %{http_code}\n" "$BASE_URL/stop-code/invalid_gtfs/invalid_provider_code" | jq '.'
echo ""

# Test 6: Service not ready (if applicable)
echo "📋 Test 6: Service readiness"
echo "Endpoint: GET /ready"
curl -s -w "\nHTTP Status: %{http_code}\n" "$BASE_URL/ready" | jq '.'
echo ""

echo "✅ Error message testing complete!"
echo ""
echo "💡 Key improvements in error messages:"
echo "1. ✅ Clear error codes (NOT_FOUND, INTERNAL_ERROR, etc.)"
echo "2. ✅ Descriptive messages with context"
echo "3. ✅ HTTP status codes"
echo "4. ✅ Timestamps for debugging"
echo "5. ✅ Structured JSON responses"
echo "6. ✅ Proper logging with context" 