#!/bin/bash

# Performance Benchmarking Script for GTFS Rust Service
# This script tests various endpoints and measures response times

BASE_URL="http://localhost:8000"
GTFS_ID="default"  # Change this to your actual GTFS ID
ROUTE_ID="route_1"  # Change this to an actual route ID
STOP_CODE="stop_1"  # Change this to an actual stop code

echo "🚀 Starting Performance Benchmark for GTFS Rust Service"
echo "=================================================="
echo "Base URL: $BASE_URL"
echo "GTFS ID: $GTFS_ID"
echo ""

# Function to measure response time
measure_response_time() {
    local endpoint=$1
    local description=$2
    local iterations=${3:-10}
    
    echo "📊 Testing: $description"
    echo "Endpoint: $endpoint"
    echo "Iterations: $iterations"
    
    total_time=0
    success_count=0
    
    for i in $(seq 1 $iterations); do
        start_time=$(date +%s%N)
        response=$(curl -s -w "%{http_code}" -o /tmp/response.json "$BASE_URL$endpoint")
        end_time=$(date +%s%N)
        
        duration=$(( (end_time - start_time) / 1000000 ))  # Convert to milliseconds
        
        if [ "$response" = "200" ]; then
            total_time=$((total_time + duration))
            success_count=$((success_count + 1))
            echo "  ✅ Request $i: ${duration}ms"
        else
            echo "  ❌ Request $i: Failed (HTTP $response)"
        fi
    done
    
    if [ $success_count -gt 0 ]; then
        avg_time=$((total_time / success_count))
        echo "  📈 Average response time: ${avg_time}ms"
        echo "  🎯 Success rate: $success_count/$iterations"
    else
        echo "  ❌ All requests failed"
    fi
    echo ""
}

# Test readiness endpoint
measure_response_time "/ready" "Readiness Probe" 5

# Test routes endpoint
measure_response_time "/routes/$GTFS_ID" "Get All Routes" 10

# Test single route endpoint
measure_response_time "/route/$GTFS_ID/$ROUTE_ID" "Get Single Route" 10

# Test route-stop mapping by route
measure_response_time "/route-stop-mapping/$GTFS_ID/route/$ROUTE_ID" "Route-Stop Mapping by Route" 10

# Test route-stop mapping by stop
measure_response_time "/route-stop-mapping/$GTFS_ID/stop/$STOP_CODE" "Route-Stop Mapping by Stop" 10

# Test fuzzy search for routes
measure_response_time "/routes/$GTFS_ID/fuzzy/bus" "Fuzzy Search Routes" 10

# Test stops endpoint
measure_response_time "/stops/$GTFS_ID" "Get All Stops" 10

# Test single stop endpoint
measure_response_time "/stop/$GTFS_ID/$STOP_CODE" "Get Single Stop" 10

# Test fuzzy search for stops
measure_response_time "/stops/$GTFS_ID/fuzzy/station" "Fuzzy Search Stops" 10

# Test memory stats
measure_response_time "/memory-stats" "Memory Statistics" 5

# Test config endpoint
measure_response_time "/config" "Configuration" 5

echo "🏁 Performance Benchmark Complete!"
echo "=================================="
echo ""
echo "💡 Performance Tips:"
echo "1. Use the optimized endpoints for better performance"
echo "2. Consider implementing caching for frequently accessed data"
echo "3. Monitor memory usage with /memory-stats endpoint"
echo "4. Use connection pooling for database operations"
echo "5. Consider implementing rate limiting for high-traffic scenarios" 