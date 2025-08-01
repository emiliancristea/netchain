# Netchain

A high-performance Proof-of-Stake blockchain built with Substrate, targeting 100,000 TPS with low gas fees and enterprise-grade scalability.

## Overview

Netchain is a next-generation blockchain platform designed for high throughput and low latency. Built on the robust Substrate framework, it provides:

- **High Performance**: Targeting 100,000+ transactions per second
- **Low Gas Fees**: Optimized fee structure for everyday transactions  
- **PoS Consensus**: Aura block production with GRANDPA finality
- **Developer Friendly**: Full Substrate ecosystem compatibility
- **Enterprise Ready**: Production-grade architecture and tooling

## Architecture

- **Consensus**: Aura (Authority Round) + GRANDPA for hybrid consensus
- **Block Time**: 6 seconds
- **Runtime**: FRAME-based modular architecture
- **Network**: libp2p networking stack
- **Database**: RocksDB for state storage

## Getting Started

### Prerequisites

- Rust (nightly toolchain)
- Git
- Build tools (gcc, cmake, pkg-config, libssl-dev, llvm, clang, libclang-dev)

### Installation

#### Install Rust

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Configure current shell
source ~/.cargo/env

# Install nightly toolchain
rustup install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

#### Clone and Build

```bash
# Clone the repository
git clone https://github.com/bunkercorporation/netchain.git
cd netchain

# Build the node (release mode)
cargo build --release
```

### Running the Node

#### Development Mode (Single Node)

Start a development chain with pre-funded accounts:

```bash
./target/release/netchain-node --dev
```

#### Custom Development Chain

Start with custom base path for persistent storage:

```bash
# Create storage directory
mkdir my-chain-data

# Run with custom base path
./target/release/netchain-node --dev --base-path ./my-chain-data
```

#### Detailed Logging

Enable debug logging for development:

```bash
RUST_BACKTRACE=1 ./target/release/netchain-node --dev -ldebug
```

#### Purge Chain Data

Reset the development chain state:

```bash
./target/release/netchain-node purge-chain --dev
```

### Node Commands

#### Help and Options

```bash
# View all available commands and options
./target/release/netchain-node --help

# Export chain specification
./target/release/netchain-node export-chain-spec --dev > netchain-dev-spec.json

# Check blocks
./target/release/netchain-node check-block --dev

# Node information
./target/release/netchain-node chain-info --dev
```

## Verification

### RPC Endpoint Testing

Once the node is running, you can verify block production using RPC calls:

```bash
# Check latest block
curl -H "Content-Type: application/json" \
     -d '{"id":1,"jsonrpc":"2.0","method":"chain_getBlock"}' \
     http://localhost:9944

# Check block by number
curl -H "Content-Type: application/json" \
     -d '{"id":1,"jsonrpc":"2.0","method":"chain_getBlockHash","params":[0]}' \
     http://localhost:9944

# Check chain info
curl -H "Content-Type: application/json" \
     -d '{"id":1,"jsonrpc":"2.0","method":"system_chain"}' \
     http://localhost:9944
```

### WebSocket Connection

The node exposes WebSocket endpoints for real-time interaction:

- **WebSocket RPC**: `ws://localhost:9944`
- **Polkadot.js Apps**: Connect to `ws://localhost:9944` in [Polkadot.js Apps](https://polkadot.js.org/apps)

### Node Health Check

```bash
# Check node health
curl -H "Content-Type: application/json" \
     -d '{"id":1,"jsonrpc":"2.0","method":"system_health"}' \
     http://localhost:9944

# Check peers
curl -H "Content-Type: application/json" \
     -d '{"id":1,"jsonrpc":"2.0","method":"system_peers"}' \
     http://localhost:9944
```

## Development

### Project Structure

```
netchain/
├── node/                    # Node implementation
│   ├── src/
│   │   ├── main.rs         # Binary entry point
│   │   ├── service.rs      # Node service configuration
│   │   ├── chain_spec.rs   # Chain specifications
│   │   ├── cli.rs          # Command line interface
│   │   ├── command.rs      # Command handling
│   │   └── rpc.rs          # RPC extensions
│   └── Cargo.toml          # Node dependencies
├── runtime/                 # Blockchain runtime
│   ├── src/
│   │   ├── lib.rs          # Runtime definition
│   │   ├── configs/        # Pallet configurations
│   │   └── apis.rs         # Runtime APIs
│   └── Cargo.toml          # Runtime dependencies
├── pallets/                 # Custom pallets
│   └── template/           # Template pallet
└── Cargo.toml              # Workspace configuration
```

### Key Features

- **Aura Consensus**: Authority-based block production
- **GRANDPA Finality**: Byzantine fault-tolerant finality gadget
- **Transaction Pool**: Efficient transaction management
- **RPC Interface**: JSON-RPC and WebSocket APIs
- **Telemetry**: Prometheus metrics and monitoring
- **Benchmarking**: Runtime benchmarking capabilities

### Runtime Information

- **Chain ID**: `netchain_dev` (development)
- **Runtime Name**: `netchain`
- **Runtime Version**: 100
- **Block Time**: 6 seconds
- **Existential Deposit**: 500 units
- **SS58 Prefix**: 42 (generic Substrate)

## Testing

### Build Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Integration Testing

```bash
# Build for testing
cargo build

# Run node in test mode
./target/debug/netchain-node --dev --tmp
```

## Advanced Usage

### Multi-Node Local Testnet

For testing multi-node consensus:

```bash
# Node 1
./target/release/netchain-node \
  --base-path /tmp/alice \
  --chain local \
  --alice \
  --port 30333 \
  --rpc-port 9944 \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001

# Node 2
./target/release/netchain-node \
  --base-path /tmp/bob \
  --chain local \
  --bob \
  --port 30334 \
  --rpc-port 9945 \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```

### Performance Monitoring

Monitor node performance:

```bash
# Enable Prometheus metrics
./target/release/netchain-node --dev --prometheus-external

# Metrics available at http://localhost:9615/metrics
```

## Troubleshooting

### Common Issues

1. **Build Failures**: Ensure all dependencies are installed and Rust nightly is active
2. **Port Conflicts**: Change ports using `--rpc-port` and `--port` options
3. **Storage Issues**: Use `purge-chain` to reset blockchain state
4. **Network Issues**: Check firewall settings for P2P and RPC ports

### Logs

Enable detailed logging:

```bash
RUST_LOG=debug ./target/release/netchain-node --dev
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Roadmap

- [ ] BABE consensus implementation for higher throughput
- [ ] Parallel transaction execution
- [ ] State pruning and archival nodes
- [ ] Cross-chain interoperability
- [ ] Smart contract pallet integration
- [ ] Governance and treasury modules
- [ ] EVM compatibility layer
- [ ] Advanced cryptographic primitives

## Support

- **Documentation**: [Substrate Developer Hub](https://docs.substrate.io/)
- **Community**: [Substrate StackExchange](https://substrate.stackexchange.com/)
- **Issues**: [GitHub Issues](https://github.com/bunkercorporation/netchain/issues)

---

Built with ❤️ using [Substrate](https://substrate.io/)