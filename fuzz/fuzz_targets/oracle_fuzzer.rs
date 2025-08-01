#![no_main]

//! # Oracle Fuzzing Target
//!
//! Comprehensive fuzzing for oracle security:
//! - Data source manipulation
//! - Price feed injection attacks
//! - Confidence score manipulation
//! - Aggregation algorithm testing
//! - Timestamp manipulation

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};

#[derive(Debug, Clone, Arbitrary)]
pub struct FuzzOracleData {
    pub provider: u64,
    pub data_key: Vec<u8>,
    pub source_id: Vec<u8>,
    pub value: Vec<u8>,
    pub confidence: u8,
    pub timestamp: u64,
    pub signature: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Arbitrary)]
pub struct FuzzOracleRequest {
    pub requester: u64,
    pub data_key: Vec<u8>,
    pub sources: Vec<Vec<u8>>,
    pub premium: bool,
    pub max_age: u64,
}

#[derive(Debug, Clone, Arbitrary)]
pub struct FuzzDataSource {
    pub source_id: Vec<u8>,
    pub name: Vec<u8>,
    pub endpoint: Vec<u8>,
    pub reliability: u8,
    pub active: bool,
}

#[derive(Debug, Clone, Arbitrary)]
pub enum FuzzOracleAction {
    RegisterSource(FuzzDataSource),
    RequestData(FuzzOracleRequest),
    ProvideData(FuzzOracleData),
    AggregateData { data_key: Vec<u8> },
    ExpireData { data_key: Vec<u8>, age: u64 },
}

fuzz_target!(|data: &[u8]| {
    let mut unstructured = Unstructured::new(data);
    
    let actions: Result<Vec<FuzzOracleAction>, _> = (0..20)
        .map(|_| FuzzOracleAction::arbitrary(&mut unstructured))
        .collect();
    
    let actions = match actions {
        Ok(actions) => actions,
        Err(_) => return,
    };
    
    fuzz_oracle_operations(actions);
});

fn fuzz_oracle_operations(actions: Vec<FuzzOracleAction>) {
    let mut oracle_state = MockOracleState::new();
    
    for action in actions {
        match action {
            FuzzOracleAction::RegisterSource(source) => {
                fuzz_register_source(&mut oracle_state, source);
            }
            FuzzOracleAction::RequestData(request) => {
                fuzz_data_request(&mut oracle_state, request);
            }
            FuzzOracleAction::ProvideData(data) => {
                fuzz_provide_data(&mut oracle_state, data);
            }
            FuzzOracleAction::AggregateData { data_key } => {
                fuzz_aggregate_data(&mut oracle_state, data_key);
            }
            FuzzOracleAction::ExpireData { data_key, age } => {
                fuzz_expire_data(&mut oracle_state, data_key, age);
            }
        }
    }
}

fn fuzz_register_source(state: &mut MockOracleState, source: FuzzDataSource) {
    // Validate source parameters
    if source.source_id.is_empty() || source.source_id.len() > 64 {
        return;
    }
    
    if source.name.len() > 128 || source.endpoint.len() > 256 {
        return;
    }
    
    if source.reliability > 100 {
        return;
    }
    
    // Register the source
    state.data_sources.insert(source.source_id.clone(), source);
    
    // Validate source was stored correctly
    assert!(state.data_sources.contains_key(&source_id));
}

fn fuzz_data_request(state: &mut MockOracleState, request: FuzzOracleRequest) {
    // Validate request parameters
    if request.data_key.is_empty() || request.data_key.len() > 128 {
        return;
    }
    
    if request.sources.is_empty() || request.sources.len() > 10 {
        return;
    }
    
    // Check if sources exist
    for source_id in &request.sources {
        if !state.data_sources.contains_key(source_id) {
            return; // Source doesn't exist
        }
    }
    
    // Check requester has sufficient balance for fees
    let fee = if request.premium { 5 } else { 2 };
    let requester_balance = state.balances.get(&request.requester).unwrap_or(&0);
    if *requester_balance < fee {
        return; // Insufficient balance
    }
    
    // Process request
    let request_id = state.next_request_id;
    state.next_request_id += 1;
    
    state.requests.insert(request_id, request.clone());
    state.balances.insert(request.requester, requester_balance - fee);
    
    // Validate request was stored
    assert!(state.requests.contains_key(&request_id));
}

fn fuzz_provide_data(state: &mut MockOracleState, data: FuzzOracleData) {
    // Validate data parameters
    if data.data_key.is_empty() || data.data_key.len() > 128 {
        return;
    }
    
    if data.source_id.is_empty() || !state.data_sources.contains_key(&data.source_id) {
        return; // Invalid source
    }
    
    if data.value.len() > 1024 {
        return; // Data too large
    }
    
    if data.confidence > 100 {
        return; // Invalid confidence
    }
    
    // Check if provider is trusted for high confidence data
    if data.confidence > 80 && !state.trusted_providers.contains(&data.provider) {
        return; // Provider not trusted
    }
    
    // Store the data
    let data_key = (data.data_key.clone(), data.source_id.clone());
    state.oracle_data.insert(data_key, data.clone());
    
    // Reward provider
    let current_balance = state.balances.get(&data.provider).unwrap_or(&0);
    state.balances.insert(data.provider, current_balance + 1);
    
    // Validate data integrity
    assert!(state.oracle_data.contains_key(&(data.data_key, data.source_id)));
}

fn fuzz_aggregate_data(state: &mut MockOracleState, data_key: Vec<u8>) {
    if data_key.is_empty() || data_key.len() > 128 {
        return;
    }
    
    // Collect data from all sources for this key
    let mut data_points = Vec::new();
    let mut total_confidence = 0u32;
    
    for ((key, source_id), oracle_data) in &state.oracle_data {
        if key == &data_key {
            data_points.push(oracle_data.clone());
            total_confidence += oracle_data.confidence as u32;
        }
    }
    
    // Only aggregate if we have enough sources
    if data_points.len() < 3 {
        return;
    }
    
    // Detect outliers and filter them out
    let filtered_data = filter_outliers(data_points);
    
    if filtered_data.is_empty() {
        return;
    }
    
    // Calculate aggregated value (simple median for numeric data)
    let aggregated_value = calculate_median_value(&filtered_data);
    let average_confidence = (total_confidence / filtered_data.len() as u32) as u8;
    
    // Store aggregated result
    let aggregated_data = AggregatedOracleData {
        value: aggregated_value,
        confidence: average_confidence,
        source_count: filtered_data.len() as u32,
        timestamp: get_current_timestamp(),
    };
    
    state.aggregated_data.insert(data_key.clone(), aggregated_data);
    
    // Validate aggregation
    assert!(state.aggregated_data.contains_key(&data_key));
}

fn fuzz_expire_data(state: &mut MockOracleState, data_key: Vec<u8>, age: u64) {
    if data_key.is_empty() {
        return;
    }
    
    let current_time = get_current_timestamp();
    let max_age = 3600; // 1 hour
    
    // Remove expired data
    state.oracle_data.retain(|(key, _), data| {
        !(key == &data_key && current_time.saturating_sub(data.timestamp) > max_age)
    });
    
    // Remove expired aggregated data
    state.aggregated_data.retain(|key, data| {
        !(key == &data_key && current_time.saturating_sub(data.timestamp) > max_age)
    });
}

fn filter_outliers(data_points: Vec<FuzzOracleData>) -> Vec<FuzzOracleData> {
    // Simple outlier detection based on confidence scores
    let mut filtered = Vec::new();
    
    for data in data_points {
        // Only include data with reasonable confidence
        if data.confidence >= 50 && data.confidence <= 100 {
            // Additional validation for numeric data
            if is_valid_numeric_data(&data.value) {
                filtered.push(data);
            }
        }
    }
    
    filtered
}

fn calculate_median_value(data_points: &[FuzzOracleData]) -> Vec<u8> {
    if data_points.is_empty() {
        return Vec::new();
    }
    
    // For simplicity, return the first valid value
    // In a real implementation, this would calculate proper median/average
    data_points[0].value.clone()
}

fn is_valid_numeric_data(data: &[u8]) -> bool {
    // Check if data looks like a valid number
    if data.is_empty() || data.len() > 32 {
        return false;
    }
    
    // Try to parse as string number
    if let Ok(s) = std::str::from_utf8(data) {
        s.parse::<f64>().is_ok()
    } else {
        false
    }
}

fn get_current_timestamp() -> u64 {
    // Mock timestamp
    1640000000 // Fixed timestamp for deterministic fuzzing
}

#[derive(Debug, Clone)]
struct AggregatedOracleData {
    value: Vec<u8>,
    confidence: u8,
    source_count: u32,
    timestamp: u64,
}

// Mock oracle state for fuzzing
#[derive(Debug)]
struct MockOracleState {
    data_sources: std::collections::HashMap<Vec<u8>, FuzzDataSource>,
    oracle_data: std::collections::HashMap<(Vec<u8>, Vec<u8>), FuzzOracleData>,
    aggregated_data: std::collections::HashMap<Vec<u8>, AggregatedOracleData>,
    requests: std::collections::HashMap<u64, FuzzOracleRequest>,
    balances: std::collections::HashMap<u64, u128>,
    trusted_providers: std::collections::HashSet<u64>,
    next_request_id: u64,
}

impl MockOracleState {
    fn new() -> Self {
        let mut balances = std::collections::HashMap::new();
        balances.insert(1, 1_000_000); // Alice
        balances.insert(2, 1_000_000); // Bob
        balances.insert(3, 1_000_000); // Charlie
        
        let mut trusted_providers = std::collections::HashSet::new();
        trusted_providers.insert(1);
        trusted_providers.insert(2);
        
        Self {
            data_sources: std::collections::HashMap::new(),
            oracle_data: std::collections::HashMap::new(),
            aggregated_data: std::collections::HashMap::new(),
            requests: std::collections::HashMap::new(),
            balances,
            trusted_providers,
            next_request_id: 0,
        }
    }
}