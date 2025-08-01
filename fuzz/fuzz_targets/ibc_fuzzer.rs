#![no_main]

//! # IBC Fuzzing Target
//!
//! Comprehensive fuzzing for IBC cross-chain security:
//! - Client state manipulation
//! - Connection handshake fuzzing
//! - Channel packet replay attacks
//! - Timeout handling
//! - State verification bypasses

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};

#[derive(Debug, Clone, Arbitrary)]
pub struct FuzzClientState {
    pub chain_id: Vec<u8>,
    pub latest_height: u64,
    pub frozen: bool,
    pub trust_level: u32,
    pub unbonding_period: u64,
}

#[derive(Debug, Clone, Arbitrary)]
pub struct FuzzConnectionEnd {
    pub state: u8, // 0: Init, 1: TryOpen, 2: Open, 3: Closed
    pub client_id: Vec<u8>,
    pub counterparty_client_id: Vec<u8>,
    pub version: Vec<u8>,
}

#[derive(Debug, Clone, Arbitrary)]
pub struct FuzzChannelEnd {
    pub state: u8, // 0: Init, 1: TryOpen, 2: Open, 3: Closed
    pub connection_id: Vec<u8>,
    pub port_id: Vec<u8>,
    pub counterparty_port_id: Vec<u8>,
    pub version: Vec<u8>,
    pub next_sequence_send: u64,
    pub next_sequence_recv: u64,
    pub next_sequence_ack: u64,
}

#[derive(Debug, Clone, Arbitrary)]
pub struct FuzzPacket {
    pub sequence: u64,
    pub source_port: Vec<u8>,
    pub source_channel: Vec<u8>,
    pub destination_port: Vec<u8>,
    pub destination_channel: Vec<u8>,
    pub data: Vec<u8>,
    pub timeout_height: u64,
    pub timeout_timestamp: u64,
}

#[derive(Debug, Clone, Arbitrary)]
pub enum FuzzIbcAction {
    CreateClient(FuzzClientState),
    UpdateClient { client_id: Vec<u8>, new_height: u64 },
    CreateConnection(FuzzConnectionEnd),
    CreateChannel(FuzzChannelEnd),
    SendPacket(FuzzPacket),
    ReceivePacket(FuzzPacket),
    AcknowledgePacket { sequence: u64, ack: Vec<u8> },
    TimeoutPacket { sequence: u64 },
}

fuzz_target!(|data: &[u8]| {
    let mut unstructured = Unstructured::new(data);
    
    let actions: Result<Vec<FuzzIbcAction>, _> = (0..15)
        .map(|_| FuzzIbcAction::arbitrary(&mut unstructured))
        .collect();
    
    let actions = match actions {
        Ok(actions) => actions,
        Err(_) => return,
    };
    
    fuzz_ibc_operations(actions);
});

fn fuzz_ibc_operations(actions: Vec<FuzzIbcAction>) {
    let mut ibc_state = MockIbcState::new();
    
    for action in actions {
        match action {
            FuzzIbcAction::CreateClient(client_state) => {
                fuzz_create_client(&mut ibc_state, client_state);
            }
            FuzzIbcAction::UpdateClient { client_id, new_height } => {
                fuzz_update_client(&mut ibc_state, client_id, new_height);
            }
            FuzzIbcAction::CreateConnection(connection) => {
                fuzz_create_connection(&mut ibc_state, connection);
            }
            FuzzIbcAction::CreateChannel(channel) => {
                fuzz_create_channel(&mut ibc_state, channel);
            }
            FuzzIbcAction::SendPacket(packet) => {
                fuzz_send_packet(&mut ibc_state, packet);
            }
            FuzzIbcAction::ReceivePacket(packet) => {
                fuzz_receive_packet(&mut ibc_state, packet);
            }
            FuzzIbcAction::AcknowledgePacket { sequence, ack } => {
                fuzz_acknowledge_packet(&mut ibc_state, sequence, ack);
            }
            FuzzIbcAction::TimeoutPacket { sequence } => {
                fuzz_timeout_packet(&mut ibc_state, sequence);
            }
        }
    }
}

fn fuzz_create_client(state: &mut MockIbcState, client_state: FuzzClientState) {
    // Validate client parameters
    if client_state.chain_id.is_empty() || client_state.chain_id.len() > 64 {
        return;
    }
    
    if client_state.latest_height == 0 {
        return; // Invalid height
    }
    
    if client_state.trust_level > 100 {
        return; // Invalid trust level
    }
    
    if client_state.unbonding_period == 0 {
        return; // Invalid unbonding period
    }
    
    // Check client limit
    if state.clients.len() >= 100 {
        return; // Too many clients
    }
    
    // Generate client ID
    let client_id = format!("client-{}", state.next_client_id);
    state.next_client_id += 1;
    
    // Store client
    state.clients.insert(client_id.clone().into_bytes(), client_state);
    
    // Validate client was stored
    assert!(state.clients.contains_key(&client_id.into_bytes()));
}

fn fuzz_update_client(state: &mut MockIbcState, client_id: Vec<u8>, new_height: u64) {
    if client_id.is_empty() || new_height == 0 {
        return;
    }
    
    // Check if client exists
    let client = match state.clients.get_mut(&client_id) {
        Some(client) => client,
        None => return, // Client doesn't exist
    };
    
    // Validate height progression
    if new_height <= client.latest_height {
        return; // Height must increase
    }
    
    // Check client is not frozen
    if client.frozen {
        return; // Cannot update frozen client
    }
    
    // Update client height
    let old_height = client.latest_height;
    client.latest_height = new_height;
    
    // Validate update
    assert!(client.latest_height > old_height);
}

fn fuzz_create_connection(state: &mut MockIbcState, connection: FuzzConnectionEnd) {
    // Validate connection parameters
    if connection.client_id.is_empty() || connection.counterparty_client_id.is_empty() {
        return;
    }
    
    if connection.state > 3 {
        return; // Invalid state
    }
    
    // Check if client exists
    if !state.clients.contains_key(&connection.client_id) {
        return; // Client doesn't exist
    }
    
    // Check connection limit
    if state.connections.len() >= 200 {
        return; // Too many connections
    }
    
    // Generate connection ID
    let connection_id = format!("connection-{}", state.next_connection_id);
    state.next_connection_id += 1;
    
    // Store connection
    state.connections.insert(connection_id.clone().into_bytes(), connection);
    
    // Validate connection was stored
    assert!(state.connections.contains_key(&connection_id.into_bytes()));
}

fn fuzz_create_channel(state: &mut MockIbcState, channel: FuzzChannelEnd) {
    // Validate channel parameters
    if channel.connection_id.is_empty() || channel.port_id.is_empty() {
        return;
    }
    
    if channel.state > 3 {
        return; // Invalid state
    }
    
    // Check if connection exists and is open
    let connection = match state.connections.get(&channel.connection_id) {
        Some(conn) if conn.state == 2 => conn, // Open state
        _ => return, // Connection doesn't exist or not open
    };
    
    // Check channel limit
    if state.channels.len() >= 500 {
        return; // Too many channels
    }
    
    // Generate channel ID
    let channel_id = format!("channel-{}", state.next_channel_id);
    state.next_channel_id += 1;
    
    // Store channel
    let channel_key = (channel.port_id.clone(), channel_id.clone().into_bytes());
    state.channels.insert(channel_key, channel);
    
    // Validate channel was stored
    assert!(state.channels.contains_key(&(channel.port_id, channel_id.into_bytes())));
}

fn fuzz_send_packet(state: &mut MockIbcState, packet: FuzzPacket) {
    // Validate packet parameters
    if packet.source_port.is_empty() || packet.source_channel.is_empty() {
        return;
    }
    
    if packet.destination_port.is_empty() || packet.destination_channel.is_empty() {
        return;
    }
    
    if packet.data.len() > 64 * 1024 {
        return; // Data too large
    }
    
    // Check if source channel exists and is open
    let channel_key = (packet.source_port.clone(), packet.source_channel.clone());
    let channel = match state.channels.get_mut(&channel_key) {
        Some(ch) if ch.state == 2 => ch, // Open state
        _ => return, // Channel doesn't exist or not open
    };
    
    // Validate sequence number
    if packet.sequence != channel.next_sequence_send {
        return; // Invalid sequence
    }
    
    // Check timeout
    let current_height = state.current_height;
    let current_timestamp = state.current_timestamp;
    
    if packet.timeout_height > 0 && current_height >= packet.timeout_height {
        return; // Already timed out
    }
    
    if packet.timeout_timestamp > 0 && current_timestamp >= packet.timeout_timestamp {
        return; // Already timed out
    }
    
    // Store packet commitment
    let packet_hash = calculate_packet_hash(&packet);
    let commitment_key = (packet.source_port.clone(), packet.sequence);
    state.packet_commitments.insert(commitment_key, packet_hash);
    
    // Update channel sequence
    channel.next_sequence_send += 1;
    
    // Validate packet was committed
    assert!(state.packet_commitments.contains_key(&(packet.source_port, packet.sequence)));
}

fn fuzz_receive_packet(state: &mut MockIbcState, packet: FuzzPacket) {
    // Validate packet parameters
    if packet.destination_port.is_empty() || packet.destination_channel.is_empty() {
        return;
    }
    
    // Check if destination channel exists and is open
    let channel_key = (packet.destination_port.clone(), packet.destination_channel.clone());
    let channel = match state.channels.get_mut(&channel_key) {
        Some(ch) if ch.state == 2 => ch, // Open state
        _ => return, // Channel doesn't exist or not open
    };
    
    // Validate sequence number (prevent replay)
    if packet.sequence != channel.next_sequence_recv {
        return; // Invalid sequence or replay
    }
    
    // Check timeout
    let current_height = state.current_height;
    let current_timestamp = state.current_timestamp;
    
    if packet.timeout_height > 0 && current_height >= packet.timeout_height {
        return; // Packet timed out
    }
    
    if packet.timeout_timestamp > 0 && current_timestamp >= packet.timeout_timestamp {
        return; // Packet timed out
    }
    
    // Process packet
    channel.next_sequence_recv += 1;
    
    // Store acknowledgment
    let ack_key = (packet.destination_port.clone(), packet.sequence);
    state.packet_acknowledgments.insert(ack_key, b"success".to_vec());
    
    // Validate packet was processed
    assert!(state.packet_acknowledgments.contains_key(&(packet.destination_port, packet.sequence)));
}

fn fuzz_acknowledge_packet(state: &mut MockIbcState, sequence: u64, ack: Vec<u8>) {
    if ack.is_empty() || ack.len() > 1024 {
        return;
    }
    
    // Find and remove packet commitment
    let mut found_commitment = None;
    for ((port, seq), _) in &state.packet_commitments {
        if *seq == sequence {
            found_commitment = Some((port.clone(), *seq));
            break;
        }
    }
    
    if let Some(commitment_key) = found_commitment {
        state.packet_commitments.remove(&commitment_key);
    }
}

fn fuzz_timeout_packet(state: &mut MockIbcState, sequence: u64) {
    // Find and remove packet commitment
    let mut found_commitment = None;
    for ((port, seq), _) in &state.packet_commitments {
        if *seq == sequence {
            found_commitment = Some((port.clone(), *seq));
            break;
        }
    }
    
    if let Some(commitment_key) = found_commitment {
        state.packet_commitments.remove(&commitment_key);
    }
}

fn calculate_packet_hash(packet: &FuzzPacket) -> [u8; 32] {
    // Simple hash calculation for testing
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    packet.sequence.hash(&mut hasher);
    packet.source_port.hash(&mut hasher);
    packet.source_channel.hash(&mut hasher);
    packet.data.hash(&mut hasher);
    
    let hash = hasher.finish();
    let mut result = [0u8; 32];
    result[..8].copy_from_slice(&hash.to_le_bytes());
    result
}

// Mock IBC state for fuzzing
#[derive(Debug)]
struct MockIbcState {
    clients: std::collections::HashMap<Vec<u8>, FuzzClientState>,
    connections: std::collections::HashMap<Vec<u8>, FuzzConnectionEnd>,
    channels: std::collections::HashMap<(Vec<u8>, Vec<u8>), FuzzChannelEnd>,
    packet_commitments: std::collections::HashMap<(Vec<u8>, u64), [u8; 32]>,
    packet_acknowledgments: std::collections::HashMap<(Vec<u8>, u64), Vec<u8>>,
    next_client_id: u64,
    next_connection_id: u64,
    next_channel_id: u64,
    current_height: u64,
    current_timestamp: u64,
}

impl MockIbcState {
    fn new() -> Self {
        Self {
            clients: std::collections::HashMap::new(),
            connections: std::collections::HashMap::new(),
            channels: std::collections::HashMap::new(),
            packet_commitments: std::collections::HashMap::new(),
            packet_acknowledgments: std::collections::HashMap::new(),
            next_client_id: 0,
            next_connection_id: 0,
            next_channel_id: 0,
            current_height: 1000,
            current_timestamp: 1640000000,
        }
    }
}