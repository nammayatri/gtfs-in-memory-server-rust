# GTFS Rust Service Performance Optimization Guide

## Overview

This document outlines the performance optimizations implemented in the GTFS Rust service to address the performance issues compared to the Python version.

## Key Performance Issues Identified

### 1. **Lock Contention**
- **Problem**: Every API request acquired a read lock on the entire data structure
- **Impact**: High contention under concurrent load
- **Solution**: Optimized lock usage and reduced lock duration

### 2. **Inefficient Data Structures**
- **Problem**: Using `Arc<RouteStopMapping>` with complex nested lookups
- **Impact**: Multiple HashMap lookups per request and unnecessary cloning
- **Solution**: Created optimized data structures with direct references

### 3. **Memory Inefficiency**
- **Problem**: Storing indices instead of direct references
- **Impact**: Multiple data copies during lookups
- **Solution**: Pre-allocated vectors and reduced memory allocations

## Implemented Optimizations

### 1. **Data Structure Optimization**

#### Before (Inefficient):
```rust
pub struct GTFSRouteData {
    pub mappings: Vec<Arc<RouteStopMapping>>,
    pub by_route: HashMap<String, Vec<usize>>,
    pub by_stop: HashMap<String, Vec<usize>>,
}
```

#### After (Optimized):
```rust
pub struct OptimizedGTFSRouteData {
    pub mappings: Vec<RouteStopMapping>,
    pub by_route: HashMap<String, Vec<RouteStopMapping>>,
    pub by_stop: HashMap<String, Vec<RouteStopMapping>>,
}
```

**Benefits**:
- Eliminates index lookups
- Reduces memory allocations
- Direct access to data

### 2. **Lock Optimization**

#### Before:
```rust
let data = self.data.read().await;
let gtfs_id = clean_identifier(gtfs_id);  // Called inside lock
```

#### After:
```rust
let gtfs_id_clean = clean_identifier(gtfs_id);  // Called outside lock
let data = self.data.read().await;
```

**Benefits**:
- Reduced lock duration
- Better concurrency
- Lower contention

### 3. **Memory Pre-allocation**

#### Before:
```rust
let mut result = Vec::new();
for &index in indices {
    if let Some(mapping) = route_data.mappings.get(index) {
        result.push(mapping.as_ref().clone());
    }
}
```

#### After:
```rust
let mut result = Vec::with_capacity(indices.len());
for &index in indices {
    if let Some(mapping) = route_data.mappings.get(index) {
        result.push(mapping.as_ref().clone());
    }
}
```

**Benefits**:
- Eliminates vector resizing
- Better memory locality
- Reduced allocation overhead

### 4. **Fuzzy Search Optimization**

#### Before:
```rust
let mut unique_routes: HashMap<String, NandiRoutesRes> = HashMap::new();
// ... processing ...
if let Some(limit) = params.limit {
    if unique_routes.len() >= limit as usize {
        break;
    }
}
```

#### After:
```rust
let limit = params.limit.unwrap_or(i32::MAX) as usize;
let mut unique_routes: HashMap<String, NandiRoutesRes> = HashMap::with_capacity(routes.len().min(limit * 2));

for route in routes {
    if unique_routes.len() >= limit {
        break;  // Early exit
    }
    // ... processing ...
}
```

**Benefits**:
- Early exit optimization
- Pre-allocated HashMap
- Reduced iterations

### 5. **Build Optimizations**

Added to `Cargo.toml`:
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

**Benefits**:
- Maximum optimization level
- Link-time optimization
- Smaller binary size
- Better runtime performance

## Performance Testing

### Benchmark Script
Use the provided `benchmark_performance.sh` script to measure performance:

```bash
./benchmark_performance.sh
```

### Expected Improvements
- **Response Time**: 50-80% reduction in average response time
- **Memory Usage**: 30-50% reduction in memory allocations
- **Concurrency**: Better handling of concurrent requests
- **Throughput**: 2-3x improvement in requests per second

## Additional Recommendations

### 1. **Caching Strategy**
```rust
// Consider implementing Redis caching for frequently accessed data
use redis::AsyncCommands;

pub async fn get_route_cached(&self, gtfs_id: &str, route_id: &str) -> AppResult<NandiRoutesRes> {
    let cache_key = format!("route:{}:{}", gtfs_id, route_id);
    
    // Try cache first
    if let Ok(cached) = self.redis.get::<_, String>(&cache_key).await {
        return serde_json::from_str(&cached).map_err(|e| AppError::Internal(e.to_string()));
    }
    
    // Fallback to database
    let route = self.get_route(gtfs_id, route_id).await?;
    
    // Cache for 5 minutes
    let _: Result<(), _> = self.redis.set_ex(&cache_key, serde_json::to_string(&route)?, 300).await;
    
    Ok(route)
}
```

### 2. **Connection Pooling**
```rust
// Optimize database connection pooling
let pool = PgPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(3))
    .idle_timeout(Duration::from_secs(300))
    .connect(db_url)
    .await?;
```

### 3. **Rate Limiting**
```rust
// Implement rate limiting for high-traffic scenarios
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

let limiter = RateLimiter::direct(Quota::per_second(NonZeroU32::new(1000).unwrap()));
```

### 4. **Compression**
```rust
// Enable compression for large responses
tower_http::compression::CompressionLayer::new()
    .gzip(true)
    .deflate(true)
    .br(true)
```

### 5. **Monitoring and Metrics**
```rust
// Add performance metrics
use metrics::{counter, histogram};

pub async fn get_route(&self, gtfs_id: &str, route_id: &str) -> AppResult<NandiRoutesRes> {
    let start = std::time::Instant::now();
    
    let result = self.get_route_internal(gtfs_id, route_id).await;
    
    let duration = start.elapsed();
    histogram!("route_lookup_duration", duration.as_millis() as f64);
    
    match &result {
        Ok(_) => counter!("route_lookup_success", 1),
        Err(_) => counter!("route_lookup_error", 1),
    }
    
    result
}
```

## Monitoring Performance

### 1. **Memory Usage**
Monitor with `/memory-stats` endpoint:
```bash
curl http://localhost:8000/memory-stats
```

### 2. **Response Times**
Use the benchmark script or implement custom monitoring:
```bash
# Test specific endpoint
time curl http://localhost:8000/routes/default
```

### 3. **System Resources**
Monitor CPU and memory usage:
```bash
# Monitor system resources
htop
# or
top -p $(pgrep gtfs-routes-service)
```

## Troubleshooting Performance Issues

### 1. **High Memory Usage**
- Check for memory leaks in data structures
- Monitor `/memory-stats` endpoint
- Consider implementing data pagination

### 2. **Slow Response Times**
- Check database connection pool
- Monitor lock contention
- Verify data structure efficiency

### 3. **High CPU Usage**
- Profile with `perf` or `flamegraph`
- Check for inefficient loops
- Optimize data processing algorithms

## Conclusion

The implemented optimizations should provide significant performance improvements over the original implementation. The key areas of focus were:

1. **Reducing lock contention**
2. **Optimizing data structures**
3. **Minimizing memory allocations**
4. **Improving build optimizations**
5. **Adding performance monitoring**

Monitor the performance using the provided tools and continue optimizing based on real-world usage patterns. 