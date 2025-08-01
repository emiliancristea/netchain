# 🚀 Netchain TPS Benchmark Suite

High-performance benchmarking tools for measuring Netchain's transaction throughput, latency, and scalability.

## 🎯 **Features**

- **Parallel Transaction Processing**: Send thousands of transactions concurrently
- **Real-time TPS Monitoring**: Live performance metrics and progress tracking
- **Shard-Aware Testing**: Distribute load across multiple shards
- **Comprehensive Analytics**: Detailed latency, throughput, and hardware metrics
- **Export to CSV**: Data analysis and reporting capabilities
- **Stress Testing**: Push the network to its limits

## 🛠️ **Installation**

```powershell
# Build the benchmark tool
cd benchmarks
cargo build --release
```

## 📊 **Usage Examples**

### **Basic TPS Test**
```powershell
# Test with 10,000 transactions using 100 workers
./target/release/netchain-benchmarks tps -t 10000 -w 100 -d 60
```

### **High-Performance Test**
```powershell
# Push for 100,000 TPS with sharding enabled
./target/release/netchain-benchmarks tps -t 100000 -w 500 -b 200 --sharding -e results.csv
```

### **Cross-Shard Testing**
```powershell
# Test cross-shard transactions across 4 shards
./target/release/netchain-benchmarks cross-shard -t 5000 -s 4
```

### **Stress Testing**
```powershell
# 5-minute stress test targeting 100k TPS
./target/release/netchain-benchmarks stress -d 300 -m 100000
```

## 📈 **Benchmark Metrics**

### **Transaction Metrics**
- Total transactions sent
- Successful/failed transaction counts
- Success rate percentage
- Blocks processed

### **Performance Metrics**
- Average TPS (Transactions Per Second)
- Peak TPS (highest 1-second window)
- Total benchmark duration
- Network efficiency

### **Latency Metrics**
- Average transaction latency
- Min/max latency
- 95th and 99th percentile latency
- Latency distribution

### **Hardware Metrics**
- CPU utilization
- Memory usage
- Network bandwidth utilization
- System resource efficiency

## 🎯 **Performance Targets**

| Metric | Target | Current |
|--------|--------|---------|
| **Average TPS** | 100,000+ | Measured |
| **Peak TPS** | 150,000+ | Measured |
| **Average Latency** | <100ms | Measured |
| **P95 Latency** | <500ms | Measured |
| **Success Rate** | >99.9% | Measured |

## 🔬 **Test Scenarios**

### **Scenario 1: Baseline Performance**
- 10,000 transactions
- 100 concurrent workers
- Single shard
- Target: >10,000 TPS

### **Scenario 2: Sharded Scaling**
- 50,000 transactions
- 4 shards enabled
- 200 concurrent workers
- Target: >50,000 TPS

### **Scenario 3: Maximum Throughput**
- 100,000 transactions
- All optimizations enabled
- 500+ concurrent workers
- Target: 100,000+ TPS

### **Scenario 4: Sustained Load**
- 5-minute duration test
- Consistent high load
- Monitor for degradation
- Target: Sustained 80,000+ TPS

## 📊 **Sample Output**

```
🚀 Netchain TPS Benchmark Results
=====================================
📊 Transaction Metrics:
  Total Sent:           50000
  Successful:           49950
  Failed:                  50
  Success Rate:         99.90%

⚡ Performance Metrics:
  Average TPS:          83245.67
  Peak TPS:             95432.10
  Total Duration:          60.12s
  Blocks Processed:         1203

🕐 Latency Metrics:
  Average:              45.23 ms
  Minimum:                   8 ms
  Maximum:                 234 ms
  95th Percentile:      89.45 ms
  99th Percentile:     156.78 ms

🔀 Sharding Metrics:
  Shards Used:         [0, 1, 2, 3]
  Shard Count:         4

🏆 Performance Comparison:
  🌟 EXCELLENT: 83245 TPS exceeds 10,000 TPS target!
  vs Ethereum:         5549.7x faster
  vs Bitcoin:         11892.2x faster
  🎯 Progress to 100k TPS: 83.2%
```

## 🔧 **Command Line Options**

### **TPS Command**
```
netchain-benchmarks tps [OPTIONS]

Options:
  -t, --transactions <N>    Number of transactions [default: 10000]
  -w, --workers <N>         Concurrent workers [default: 100]
  -d, --duration <SECS>     Test duration [default: 60]
  -b, --batch-size <N>      Batch size [default: 100]
      --sharding            Enable sharding mode
  -e, --export <FILE>       Export to CSV file
```

### **Cross-Shard Command**
```
netchain-benchmarks cross-shard [OPTIONS]

Options:
  -t, --transactions <N>    Cross-shard transactions [default: 1000]
  -s, --shards <N>          Number of shards [default: 4]
```

### **Stress Command**
```
netchain-benchmarks stress [OPTIONS]

Options:
  -d, --duration <SECS>     Test duration [default: 300]
  -m, --max-tps <N>         Maximum TPS target [default: 100000]
```

## 📈 **Performance Optimization Tips**

### **Hardware Recommendations**
- **CPU**: 16+ cores for parallel processing
- **RAM**: 32GB+ for high transaction volumes
- **Network**: Gigabit+ connection for throughput
- **Storage**: NVMe SSD for fast state access

### **Network Configuration**
- Increase file descriptor limits
- Optimize TCP buffer sizes
- Enable connection pooling
- Use dedicated benchmark network

### **Benchmark Settings**
- Start with conservative worker counts
- Gradually increase batch sizes
- Monitor resource utilization
- Use sharding for >50k TPS tests

## 🚀 **Expected Results**

### **Netchain vs Competition**

| Network | TPS | Block Time | Finality |
|---------|-----|------------|----------|
| **Netchain** | **100,000+** | **3s** | **3s** |
| Ethereum | 15 | 12s | 5min+ |
| Bitcoin | 7 | 600s | 60min+ |
| Solana | 65,000 | 0.4s | 0.4s |
| Polygon | 7,000 | 2s | 2min |
| BSC | 300 | 3s | 3s |

### **Cost Comparison**
- **Netchain**: ~$0.0001 per transaction
- **Ethereum**: $1-50+ per transaction
- **Savings**: 99.99% cost reduction

---

## 🎉 **Getting Started**

1. **Start Netchain Node**:
```powershell
./target/release/netchain-node --dev --tmp
```

2. **Run Benchmark**:
```powershell
cd benchmarks
cargo run --release -- tps -t 10000 -w 100 --sharding
```

3. **Analyze Results**:
- Review console output for real-time metrics
- Export to CSV for detailed analysis
- Compare against performance targets

**Ready to push Netchain to 100,000+ TPS!** 🚀