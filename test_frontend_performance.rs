// Test frontend HTTP client performance
// This simulates what the frontend does when communicating with the backend

use std::time::Instant;
use tokio;

#[tokio::main]
async fn main() {
    println!("═════════════════════════════════════════════════");
    println!("🔍 FRONTEND HTTP CLIENT PERFORMANCE TEST");
    println!("═════════════════════════════════════════════════");
    println!();

    let backend_url = "http://localhost:5000";
    let test_audio = vec![0u8; 960]; // 480 samples @ 16-bit = 960 bytes
    
    // Test 1: Single request with reqwest (what frontend uses)
    println!("Test 1️⃣  Single Request (reqwest - Frontend HTTP Client)");
    println!("-" * 50);
    
    let start = Instant::now();
    let client = reqwest::Client::new();
    
    let payload = serde_json::json!({
        "audio_data": test_audio,
        "sample_rate": 48000
    });
    
    let response = match client
        .post(&format!("{}/analyze", backend_url))
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis();
            println!("✓ Request successful: {}ms", elapsed);
            resp
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis();
            println!("✗ Request failed after {}ms: {}", elapsed, e);
            return;
        }
    };

    let elapsed = start.elapsed().as_millis();
    
    if response.status().is_success() {
        match response.json::<serde_json::Value>().await {
            Ok(data) => {
                println!("✓ Response parsed: {}ms total", elapsed);
                println!("  Notes: {}", data["notes"].as_array().map(|a| a.len()).unwrap_or(0));
            }
            Err(e) => println!("✗ Parse failed: {}", e),
        }
    }
    println!();
    
    // Test 2: Sequential requests
    println!("Test 2️⃣  Sequential Requests (10 requests)");
    println!("-" * 50);
    
    let mut times = vec![];
    let client = reqwest::Client::new();
    
    for i in 1..=10 {
        let start = Instant::now();
        let payload = serde_json::json!({
            "audio_data": test_audio.clone(),
            "sample_rate": 48000
        });
        
        match client
            .post(&format!("{}/analyze", backend_url))
            .json(&payload)
            .send()
            .await
        {
            Ok(_) => {
                let elapsed = start.elapsed().as_millis() as f64;
                times.push(elapsed);
                println!("  Request {:2}: ✓ {}ms", i, elapsed);
            }
            Err(e) => println!("  Request {:2}: ✗ {}", i, e),
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    if !times.is_empty() {
        let avg = times.iter().sum::<f64>() / times.len() as f64;
        let min = times.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = times.iter().cloned().fold(0.0, f64::max);
        
        println!();
        println!("Sequential Stats:");
        println!("  Avg: {:.1}ms", avg);
        println!("  Min: {:.1}ms", min);
        println!("  Max: {:.1}ms", max);
    }
    println!();
    
    // Test 3: Concurrent requests
    println!("Test 3️⃣  Concurrent Requests (20 parallel)");
    println!("-" * 50);
    
    let start = Instant::now();
    let mut handles = vec![];
    
    for i in 0..20 {
        let handle = tokio::spawn(async move {
            let client = reqwest::Client::new();
            let test_audio = vec![0u8; 960];
            let payload = serde_json::json!({
                "audio_data": test_audio,
                "sample_rate": 48000
            });
            
            let req_start = Instant::now();
            let result = client
                .post("http://localhost:5000/analyze")
                .json(&payload)
                .send()
                .await;
            
            (i, req_start.elapsed().as_millis())
        });
        
        handles.push(handle);
    }
    
    let mut concurrent_times = vec![];
    for handle in handles {
        if let Ok((id, elapsed)) = handle.await {
            concurrent_times.push(elapsed);
            if (id + 1) % 5 == 0 {
                println!("  Completed {}/20 requests", id + 1);
            }
        }
    }
    
    let total_elapsed = start.elapsed().as_millis();
    
    if !concurrent_times.is_empty() {
        let avg = concurrent_times.iter().sum::<u128>() as f64 / concurrent_times.len() as f64;
        let min = concurrent_times.iter().copied().min().unwrap_or(0) as f64;
        let max = concurrent_times.iter().copied().max().unwrap_or(0) as f64;
        
        println!();
        println!("Concurrent Stats:");
        println!("  Total Time: {}ms", total_elapsed);
        println!("  Avg Response: {:.1}ms", avg);
        println!("  Min Response: {:.1}ms", min);
        println!("  Max Response: {:.1}ms", max);
        println!("  Throughput: {:.1} req/sec", 20.0 * 1000.0 / total_elapsed as f64);
    }
    println!();
    
    println!("═════════════════════════════════════════════════");
    println!("✓ Frontend Performance Test Complete!");
    println!("═════════════════════════════════════════════════");
    println!();
    println!("ANALYSIS:");
    println!("  ✅ reqwest is a high-performance HTTP client");
    println!("  ✅ Should see similar performance to raw sockets");
    println!("  ✅ Frontend should get ~1-5ms response times");
}
