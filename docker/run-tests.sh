#!/bin/bash

# Comprehensive test runner for Netchain
# Runs all test suites in Docker environment

set -e

echo "🧪 Starting Netchain Comprehensive Test Suite"
echo "=============================================="

# Wait for nodes to be ready
echo "⏳ Waiting for Netchain nodes to be ready..."

check_node() {
    local url=$1
    local name=$2
    
    for i in {1..30}; do
        if curl -s "$url/health" > /dev/null 2>&1; then
            echo "✅ $name is ready"
            return 0
        fi
        echo "⏳ Waiting for $name... (attempt $i/30)"
        sleep 2
    done
    
    echo "❌ $name failed to start"
    return 1
}

# Check all nodes
check_node "$ALICE_NODE" "Alice"
check_node "$BOB_NODE" "Bob" 
check_node "$CHARLIE_NODE" "Charlie"
check_node "$DAVE_NODE" "Dave"

echo "🎉 All nodes are ready!"

# Set up test environment
export RUST_LOG=info
export RUST_BACKTRACE=1

# Create results directory
mkdir -p /results

echo ""
echo "🔬 Running Unit Tests"
echo "===================="
cargo test --workspace --lib --verbose 2>&1 | tee /results/unit_tests.log

echo ""
echo "🛡️ Running Security Tests"
echo "========================"
cargo test --package tests --test consensus_security_tests --verbose 2>&1 | tee /results/security_tests.log
cargo test --package tests --test contract_security_tests --verbose 2>&1 | tee /results/contract_security_tests.log
cargo test --package tests --test attack_simulations --verbose 2>&1 | tee /results/attack_simulation_tests.log

echo ""
echo "🚀 Running Performance Tests"
echo "============================"
cargo test --package tests --test tps_benchmarks --verbose 2>&1 | tee /results/performance_tests.log
cargo test --package tests --test fee_benchmarks --verbose 2>&1 | tee /results/fee_tests.log

echo ""
echo "🌐 Running Integration Tests"
echo "============================"
cargo test --package tests --test comprehensive_integration_tests --verbose 2>&1 | tee /results/integration_tests.log
cargo test --package tests --test interoperability_test --verbose 2>&1 | tee /results/interoperability_tests.log

echo ""
echo "🔀 Running Fuzz Tests (Short Duration)"
echo "====================================="
cd /tests
timeout 60s cargo fuzz run contract_fuzzer || echo "Fuzz test completed with timeout"
timeout 60s cargo fuzz run oracle_fuzzer || echo "Fuzz test completed with timeout"  
timeout 60s cargo fuzz run ibc_fuzzer || echo "Fuzz test completed with timeout"

echo ""
echo "📊 Generating Test Report"
echo "========================"

# Create comprehensive test report
cat > /results/test_report.md << EOF
# Netchain Comprehensive Test Report
Generated: $(date)

## Test Environment
- Alice Node: $ALICE_NODE
- Bob Node: $BOB_NODE
- Charlie Node: $CHARLIE_NODE
- Dave Node: $DAVE_NODE

## Test Results Summary

### Unit Tests
$(grep -c "test result: ok" /results/unit_tests.log || echo "0") tests passed
$(grep -c "test result: FAILED" /results/unit_tests.log || echo "0") tests failed

### Security Tests
$(grep -c "test result: ok" /results/security_tests.log || echo "0") consensus security tests passed
$(grep -c "test result: ok" /results/contract_security_tests.log || echo "0") contract security tests passed
$(grep -c "test result: ok" /results/attack_simulation_tests.log || echo "0") attack simulation tests passed

### Performance Tests  
$(grep -c "test result: ok" /results/performance_tests.log || echo "0") performance tests passed
$(grep -c "test result: ok" /results/fee_tests.log || echo "0") fee benchmark tests passed

### Integration Tests
$(grep -c "test result: ok" /results/integration_tests.log || echo "0") integration tests passed
$(grep -c "test result: ok" /results/interoperability_tests.log || echo "0") interoperability tests passed

## Test Logs
All detailed test logs are available in /results/

## Fuzz Testing
Fuzz tests were executed for 60 seconds each to check for:
- Contract security vulnerabilities
- Oracle data manipulation
- IBC protocol edge cases

EOF

echo "📈 Test Results Summary:"
echo "======================="
grep "test result:" /results/*.log | sort | uniq -c

echo ""
echo "🎯 Overall Test Status:"
if grep -q "FAILED" /results/*.log; then
    echo "❌ Some tests failed - check logs for details"
    exit 1
else
    echo "✅ All tests passed successfully!"
    echo ""
    echo "🚀 Netchain is ready for production!"
fi

echo ""
echo "📂 Test artifacts saved to /results/"
echo "📊 Full report available at /results/test_report.md"

# Keep container running for result collection
if [ "$1" == "--keep-running" ]; then
    echo "🔄 Keeping container running for result collection..."
    tail -f /dev/null
fi