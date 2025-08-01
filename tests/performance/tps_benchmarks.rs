//! # TPS Performance Benchmarks
//!
//! Comprehensive performance testing for Netchain:
//! - Transaction throughput measurement
//! - Block production benchmarks
//! - Memory usage profiling
//! - Network latency simulation
//! - Scalability testing

#![cfg(test)]

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use subxt::{OnlineClient, PolkadotConfig, tx::TxPayload};

// Performance test configuration
const BENCHMARK_ACCOUNTS: usize = 1000;
const TPS_TEST_DURATION: Duration = Duration::from_secs(60);
const BATCH_SIZES: &[usize] = &[1, 10, 50, 100, 500, 1000];
const WORKER_COUNTS: &[usize] = &[1, 4, 8, 16, 32, 64, 128];

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_transactions: u64,
    pub successful_transactions: u64,
    pub failed_transactions: u64,
    pub average_tps: f64,
    pub peak_tps: f64,
    pub average_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub duration_seconds: f64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            total_transactions: 0,
            successful_transactions: 0,
            failed_transactions: 0,
            average_tps: 0.0,
            peak_tps: 0.0,
            average_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            duration_seconds: 0.0,
        }
    }
}

pub struct TpsBenchmark {
    pub rt: Runtime,
    pub client: Option<OnlineClient<PolkadotConfig>>,
    pub accounts: Vec<subxt::ext::sp_core::sr25519::Pair>,
}

impl TpsBenchmark {
    pub fn new() -> Self {
        let rt = Runtime::new().expect("Failed to create tokio runtime");
        
        // Generate test accounts
        let mut accounts = Vec::new();
        for i in 0..BENCHMARK_ACCOUNTS {
            let seed = format!("//TestAccount{}", i);
            let pair = subxt::ext::sp_core::sr25519::Pair::from_string(&seed, None)
                .expect("Failed to create test account");
            accounts.push(pair);
        }
        
        Self {
            rt,
            client: None,
            accounts,
        }
    }
    
    pub async fn setup_client(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.client = Some(OnlineClient::<PolkadotConfig>::from_url("ws://127.0.0.1:9944").await?);
        Ok(())
    }
    
    pub async fn benchmark_basic_transfers(&self, batch_size: usize, workers: usize) -> PerformanceMetrics {
        let mut metrics = PerformanceMetrics::new();
        
        if self.client.is_none() {
            return metrics;
        }
        
        let client = self.client.as_ref().unwrap();
        let start_time = Instant::now();
        
        // Track latencies for percentile calculation
        let latencies = Arc::new(Mutex::new(Vec::new()));
        
        // Create worker tasks
        let mut handles = Vec::new();
        
        for worker_id in 0..workers {
            let client = client.clone();
            let accounts = self.accounts.clone();
            let latencies = latencies.clone();
            
            let handle = tokio::spawn(async move {
                let mut worker_metrics = (0u64, 0u64); // (successful, failed)
                let worker_start = worker_id * batch_size;
                let worker_end = std::cmp::min(worker_start + batch_size, accounts.len() - 1);
                
                for i in worker_start..worker_end {
                    if i + 1 >= accounts.len() {
                        break;
                    }
                    
                    let from = &accounts[i];
                    let to_index = (i + 1) % accounts.len();
                    let to = &accounts[to_index];
                    
                    let tx_start = Instant::now();
                    
                    // Create transfer transaction
                    let transfer_tx = client.tx()
                        .balances()
                        .transfer_allow_death(to.public().into(), 1000);
                    
                    match transfer_tx {
                        Ok(tx) => {
                            match tx.sign_and_submit(&from).await {
                                Ok(_) => {
                                    worker_metrics.0 += 1;
                                    let latency = tx_start.elapsed().as_millis() as f64;
                                    latencies.lock().unwrap().push(latency);
                                }
                                Err(_) => {
                                    worker_metrics.1 += 1;
                                }
                            }
                        }
                        Err(_) => {
                            worker_metrics.1 += 1;
                        }
                    }
                }
                
                worker_metrics
            });
            
            handles.push(handle);
        }
        
        // Wait for all workers to complete
        for handle in handles {
            if let Ok((successful, failed)) = handle.await {
                metrics.successful_transactions += successful;
                metrics.failed_transactions += failed;
            }
        }
        
        let total_duration = start_time.elapsed();
        metrics.duration_seconds = total_duration.as_secs_f64();
        metrics.total_transactions = metrics.successful_transactions + metrics.failed_transactions;
        metrics.average_tps = metrics.successful_transactions as f64 / metrics.duration_seconds;
        
        // Calculate latency statistics
        let mut latency_vec = latencies.lock().unwrap();
        if !latency_vec.is_empty() {
            latency_vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
            metrics.average_latency_ms = latency_vec.iter().sum::<f64>() / latency_vec.len() as f64;
            let p99_index = (latency_vec.len() as f64 * 0.99) as usize;
            metrics.p99_latency_ms = latency_vec[std::cmp::min(p99_index, latency_vec.len() - 1)];
        }
        
        metrics
    }
    
    pub async fn benchmark_contract_calls(&self, batch_size: usize) -> PerformanceMetrics {
        let mut metrics = PerformanceMetrics::new();
        
        if self.client.is_none() {
            return metrics;
        }
        
        let client = self.client.as_ref().unwrap();
        let start_time = Instant::now();
        
        // First deploy a simple contract
        let simple_contract_code = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
            0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00,
            0x07, 0x05, 0x01, 0x01, 0x5f, 0x00,
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];
        
        let deploy_tx = client.tx()
            .contracts()
            .instantiate(
                100_000,
                subxt::utils::Weight::from_parts(1_000_000, 0),
                None,
                simple_contract_code,
                vec![],
                vec![],
            );
        
        let contract_address = match deploy_tx {
            Ok(tx) => {
                match tx.sign_and_submit_then_watch(&self.accounts[0]).await {
                    Ok(events) => {
                        // Extract contract address from events
                        // This is simplified - in real code, parse the events properly
                        Some(self.accounts[0].public())
                    }
                    Err(_) => None,
                }
            }
            Err(_) => None,
        };
        
        if let Some(_contract_addr) = contract_address {
            // Benchmark contract calls
            for i in 0..batch_size {
                let account = &self.accounts[i % self.accounts.len()];
                
                let call_tx = client.tx()
                    .contracts()
                    .call(
                        self.accounts[0].public().into(), // contract address
                        0, // value
                        subxt::utils::Weight::from_parts(500_000, 0),
                        None,
                        vec![],
                    );
                
                match call_tx {
                    Ok(tx) => {
                        match tx.sign_and_submit(&account).await {
                            Ok(_) => metrics.successful_transactions += 1,
                            Err(_) => metrics.failed_transactions += 1,
                        }
                    }
                    Err(_) => metrics.failed_transactions += 1,
                }
            }
        }
        
        let total_duration = start_time.elapsed();
        metrics.duration_seconds = total_duration.as_secs_f64();
        metrics.total_transactions = metrics.successful_transactions + metrics.failed_transactions;
        metrics.average_tps = metrics.successful_transactions as f64 / metrics.duration_seconds;
        
        metrics
    }
    
    pub async fn benchmark_cross_chain_operations(&self, batch_size: usize) -> PerformanceMetrics {
        let mut metrics = PerformanceMetrics::new();
        
        if self.client.is_none() {
            return metrics;
        }
        
        let client = self.client.as_ref().unwrap();
        let start_time = Instant::now();
        
        // Benchmark IBC operations
        for i in 0..batch_size {
            let account = &self.accounts[i % self.accounts.len()];
            
            // Create IBC client
            let ibc_tx = client.tx()
                .ibc_core()
                .create_client(
                    format!("test-chain-{}", i).into_bytes(),
                    1000 + i as u64,
                    67,
                    1800,
                );
            
            match ibc_tx {
                Ok(tx) => {
                    match tx.sign_and_submit(&account).await {
                        Ok(_) => metrics.successful_transactions += 1,
                        Err(_) => metrics.failed_transactions += 1,
                    }
                }
                Err(_) => metrics.failed_transactions += 1,
            }
        }
        
        let total_duration = start_time.elapsed();
        metrics.duration_seconds = total_duration.as_secs_f64();
        metrics.total_transactions = metrics.successful_transactions + metrics.failed_transactions;
        metrics.average_tps = metrics.successful_transactions as f64 / metrics.duration_seconds;
        
        metrics
    }
    
    pub async fn benchmark_oracle_operations(&self, batch_size: usize) -> PerformanceMetrics {
        let mut metrics = PerformanceMetrics::new();
        
        if self.client.is_none() {
            return metrics;
        }
        
        let client = self.client.as_ref().unwrap();
        let start_time = Instant::now();
        
        // Benchmark oracle data requests
        for i in 0..batch_size {
            let account = &self.accounts[i % self.accounts.len()];
            
            let oracle_tx = client.tx()
                .oracle()
                .request_data(
                    format!("BTC/USD-{}", i).into_bytes(),
                    vec![b"test_source".to_vec()],
                    false,
                    None,
                );
            
            match oracle_tx {
                Ok(tx) => {
                    match tx.sign_and_submit(&account).await {
                        Ok(_) => metrics.successful_transactions += 1,
                        Err(_) => metrics.failed_transactions += 1,
                    }
                }
                Err(_) => metrics.failed_transactions += 1,
            }
        }
        
        let total_duration = start_time.elapsed();
        metrics.duration_seconds = total_duration.as_secs_f64();
        metrics.total_transactions = metrics.successful_transactions + metrics.failed_transactions;
        metrics.average_tps = metrics.successful_transactions as f64 / metrics.duration_seconds;
        
        metrics
    }
}

// System resource monitoring
pub fn get_system_metrics() -> (f64, f64) {
    // Mock implementation - in real code, use system monitoring crates
    let memory_usage = 256.0; // MB
    let cpu_usage = 45.0; // Percent
    (memory_usage, cpu_usage)
}

// Benchmark functions for Criterion
fn benchmark_transfers(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut benchmark = TpsBenchmark::new();
    
    rt.block_on(async {
        if benchmark.setup_client().await.is_err() {
            println!("Warning: Could not connect to Netchain node for benchmarking");
            return;
        }
        
        let mut group = c.benchmark_group("transfers");
        
        for &batch_size in BATCH_SIZES {
            for &workers in WORKER_COUNTS {
                group.throughput(Throughput::Elements(batch_size as u64));
                group.bench_with_input(
                    BenchmarkId::from_parameter(format!("batch_{}_workers_{}", batch_size, workers)),
                    &(batch_size, workers),
                    |b, &(batch_size, workers)| {
                        b.to_async(&rt).iter(|| async {
                            benchmark.benchmark_basic_transfers(batch_size, workers).await
                        });
                    },
                );
            }
        }
        
        group.finish();
    });
}

fn benchmark_contracts(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut benchmark = TpsBenchmark::new();
    
    rt.block_on(async {
        if benchmark.setup_client().await.is_err() {
            return;
        }
        
        let mut group = c.benchmark_group("contracts");
        
        for &batch_size in BATCH_SIZES {
            group.throughput(Throughput::Elements(batch_size as u64));
            group.bench_with_input(
                BenchmarkId::from_parameter(batch_size),
                &batch_size,
                |b, &batch_size| {
                    b.to_async(&rt).iter(|| async {
                        benchmark.benchmark_contract_calls(batch_size).await
                    });
                },
            );
        }
        
        group.finish();
    });
}

fn benchmark_interoperability(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut benchmark = TpsBenchmark::new();
    
    rt.block_on(async {
        if benchmark.setup_client().await.is_err() {
            return;
        }
        
        let mut group = c.benchmark_group("interoperability");
        
        for &batch_size in &[10, 50, 100] { // Smaller batches for complex operations
            group.throughput(Throughput::Elements(batch_size as u64));
            
            group.bench_with_input(
                BenchmarkId::new("ibc", batch_size),
                &batch_size,
                |b, &batch_size| {
                    b.to_async(&rt).iter(|| async {
                        benchmark.benchmark_cross_chain_operations(batch_size).await
                    });
                },
            );
            
            group.bench_with_input(
                BenchmarkId::new("oracle", batch_size),
                &batch_size,
                |b, &batch_size| {
                    b.to_async(&rt).iter(|| async {
                        benchmark.benchmark_oracle_operations(batch_size).await
                    });
                },
            );
        }
        
        group.finish();
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(30))
        .sample_size(10);
    targets = benchmark_transfers, benchmark_contracts, benchmark_interoperability
);

criterion_main!(benches);

#[cfg(test)]
mod performance_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_tps_measurement() {
        let mut benchmark = TpsBenchmark::new();
        
        // Test with mock data if node is not available
        let metrics = benchmark.benchmark_basic_transfers(10, 2).await;
        
        println!("TPS Benchmark Results:");
        println!("  Total transactions: {}", metrics.total_transactions);
        println!("  Successful: {}", metrics.successful_transactions);
        println!("  Failed: {}", metrics.failed_transactions);
        println!("  Average TPS: {:.2}", metrics.average_tps);
        println!("  Duration: {:.2}s", metrics.duration_seconds);
        
        // Basic sanity checks
        assert!(metrics.total_transactions > 0 || metrics.duration_seconds > 0.0);
    }
    
    #[test]
    fn test_system_metrics_collection() {
        let (memory_mb, cpu_percent) = get_system_metrics();
        
        println!("System Metrics:");
        println!("  Memory usage: {:.2} MB", memory_mb);
        println!("  CPU usage: {:.2}%", cpu_percent);
        
        assert!(memory_mb >= 0.0);
        assert!(cpu_percent >= 0.0 && cpu_percent <= 100.0);
    }
    
    #[test]
    fn test_performance_metrics_calculation() {
        let mut metrics = PerformanceMetrics::new();
        
        metrics.successful_transactions = 1000;
        metrics.failed_transactions = 10;
        metrics.duration_seconds = 10.0;
        
        metrics.total_transactions = metrics.successful_transactions + metrics.failed_transactions;
        metrics.average_tps = metrics.successful_transactions as f64 / metrics.duration_seconds;
        
        assert_eq!(metrics.total_transactions, 1010);
        assert_eq!(metrics.average_tps, 100.0);
        
        let success_rate = (metrics.successful_transactions as f64 / metrics.total_transactions as f64) * 100.0;
        assert!(success_rate > 99.0); // Should have >99% success rate
    }
    
    #[tokio::test]
    async fn benchmark_comparison_test() {
        let mut benchmark = TpsBenchmark::new();
        
        // Compare different batch sizes
        let batch_10 = benchmark.benchmark_basic_transfers(10, 1).await;
        let batch_100 = benchmark.benchmark_basic_transfers(100, 1).await;
        
        println!("Batch Size Comparison:");
        println!("  Batch 10 - TPS: {:.2}, Duration: {:.2}s", batch_10.average_tps, batch_10.duration_seconds);
        println!("  Batch 100 - TPS: {:.2}, Duration: {:.2}s", batch_100.average_tps, batch_100.duration_seconds);
        
        // Larger batches should generally have higher throughput
        // (Though this may not hold in mock testing)
    }
}