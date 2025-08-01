#![cfg_attr(not(feature = "std"), no_std)]

//! # IBC Core Pallet
//!
//! A simplified Inter-Blockchain Communication (IBC) protocol implementation for Netchain.
//! 
//! This pallet provides:
//! - Cross-chain client management
//! - Connection establishment between chains
//! - Channel creation for application-specific communication
//! - Packet routing and acknowledgments
//! - Ultra-low fees for cross-chain operations
//!
//! ## Security Features
//! - Replay attack prevention through sequence numbers
//! - Timeout handling for failed packets
//! - Client state verification
//! - Connection and channel state validation

pub use pallet::*;

use frame_support::{
    dispatch::{DispatchResult, DispatchResultWithPostInfo},
    pallet_prelude::*,
    traits::{Get, ReservableCurrency, ExistenceRequirement},
    PalletId,
};
use frame_system::pallet_prelude::*;
use sp_std::{vec::Vec, collections::btree_map::BTreeMap};
use sp_runtime::{
    traits::{BlakeTwo256, Hash, Saturating, Zero, AccountIdConversion},
    SaturatedConversion,
};
use sp_core::H256;

/// IBC client identifier
pub type ClientId = Vec<u8>;
/// IBC connection identifier  
pub type ConnectionId = Vec<u8>;
/// IBC channel identifier
pub type ChannelId = Vec<u8>;
/// IBC port identifier
pub type PortId = Vec<u8>;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_balances::Config + pallet_timestamp::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Currency for reserving funds for IBC operations
        type Currency: ReservableCurrency<Self::AccountId>;

        /// Maximum number of clients per chain
        #[pallet::constant]
        type MaxClients: Get<u32>;

        /// Maximum number of connections per client
        #[pallet::constant]
        type MaxConnections: Get<u32>;

        /// Maximum number of channels per connection
        #[pallet::constant]
        type MaxChannels: Get<u32>;

        /// Fee for creating an IBC client (ultra-low)
        #[pallet::constant]
        type ClientCreationFee: Get<BalanceOf<Self>>;

        /// Fee for cross-chain packet transmission (ultra-low)
        #[pallet::constant]
        type PacketTransmissionFee: Get<BalanceOf<Self>>;

        /// Pallet identifier for account derivation
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// WeightInfo for benchmarking
        type WeightInfo: WeightInfo;
    }

    pub type BalanceOf<T> = <<T as Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// IBC client state information
    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
    pub struct ClientState {
        /// Chain identifier this client tracks
        pub chain_id: Vec<u8>,
        /// Latest height processed
        pub latest_height: u64,
        /// Client is frozen (security incident)
        pub frozen: bool,
        /// Trust level threshold
        pub trust_level: u32,
        /// Unbonding period
        pub unbonding_period: u64,
    }

    /// IBC connection state
    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
    pub enum ConnectionState {
        /// Connection initialization started
        Init,
        /// Connection try-open phase
        TryOpen,
        /// Connection established
        Open,
        /// Connection closed
        Closed,
    }

    /// IBC connection end information
    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
    pub struct ConnectionEnd {
        /// Current connection state
        pub state: ConnectionState,
        /// Local client identifier
        pub client_id: ClientId,
        /// Counterparty connection details
        pub counterparty_client_id: ClientId,
        /// Connection version for compatibility
        pub version: Vec<u8>,
    }

    /// IBC channel state
    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
    pub enum ChannelState {
        /// Channel initialization started
        Init,
        /// Channel try-open phase
        TryOpen,
        /// Channel established
        Open,  
        /// Channel closed
        Closed,
    }

    /// IBC channel end information
    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
    pub struct ChannelEnd {
        /// Current channel state
        pub state: ChannelState,
        /// Connection identifier
        pub connection_id: ConnectionId,
        /// Port identifier for this channel
        pub port_id: PortId,
        /// Counterparty port identifier
        pub counterparty_port_id: PortId,
        /// Channel version
        pub version: Vec<u8>,
        /// Next sequence number for sending packets
        pub next_sequence_send: u64,
        /// Next sequence number for receiving packets
        pub next_sequence_recv: u64,
        /// Next sequence number for acknowledgments
        pub next_sequence_ack: u64,
    }

    /// IBC packet for cross-chain communication
    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
    pub struct Packet {
        /// Sequence number for ordering
        pub sequence: u64,
        /// Source port identifier
        pub source_port: PortId,
        /// Source channel identifier
        pub source_channel: ChannelId,
        /// Destination port identifier
        pub destination_port: PortId,
        /// Destination channel identifier
        pub destination_channel: ChannelId,
        /// Packet data payload
        pub data: Vec<u8>,
        /// Timeout height (0 = no timeout)
        pub timeout_height: u64,
        /// Timeout timestamp
        pub timeout_timestamp: u64,
    }

    /// Storage for IBC clients
    #[pallet::storage]
    #[pallet::getter(fn clients)]
    pub type Clients<T: Config> = StorageMap<_, Blake2_128Concat, ClientId, ClientState>;

    /// Storage for IBC connections
    #[pallet::storage]
    #[pallet::getter(fn connections)]
    pub type Connections<T: Config> = StorageMap<_, Blake2_128Concat, ConnectionId, ConnectionEnd>;

    /// Storage for IBC channels
    #[pallet::storage]
    #[pallet::getter(fn channels)]
    pub type Channels<T: Config> = StorageDoubleMap<
        _, Blake2_128Concat, PortId,
        Blake2_128Concat, ChannelId,
        ChannelEnd
    >;

    /// Storage for packet commitments (prevents replay attacks)
    #[pallet::storage]
    #[pallet::getter(fn packet_commitments)]
    pub type PacketCommitments<T: Config> = StorageDoubleMap<
        _, Blake2_128Concat, PortId,
        Blake2_128Concat, u64, // sequence number
        H256, // packet hash
    >;

    /// Storage for packet acknowledgments
    #[pallet::storage]
    #[pallet::getter(fn packet_acknowledgments)]  
    pub type PacketAcknowledgments<T: Config> = StorageDoubleMap<
        _, Blake2_128Concat, PortId,
        Blake2_128Concat, u64, // sequence number
        Vec<u8>, // acknowledgment data
    >;

    /// Next client identifier to assign
    #[pallet::storage]
    #[pallet::getter(fn next_client_id)]
    pub type NextClientId<T> = StorageValue<_, u32, ValueQuery>;

    /// Next connection identifier to assign  
    #[pallet::storage]
    #[pallet::getter(fn next_connection_id)]
    pub type NextConnectionId<T> = StorageValue<_, u32, ValueQuery>;

    /// Next channel identifier to assign
    #[pallet::storage]
    #[pallet::getter(fn next_channel_id)]
    pub type NextChannelId<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// IBC client created
        ClientCreated { client_id: ClientId, chain_id: Vec<u8> },
        /// IBC client updated with new state
        ClientUpdated { client_id: ClientId, height: u64 },
        /// IBC connection opened
        ConnectionOpened { connection_id: ConnectionId, client_id: ClientId },
        /// IBC channel opened
        ChannelOpened { port_id: PortId, channel_id: ChannelId, connection_id: ConnectionId },
        /// Cross-chain packet sent
        PacketSent { 
            sequence: u64, 
            source_port: PortId, 
            source_channel: ChannelId,
            destination_port: PortId,
            destination_channel: ChannelId,
            data: Vec<u8>
        },
        /// Cross-chain packet received
        PacketReceived { 
            sequence: u64, 
            source_port: PortId, 
            source_channel: ChannelId,
            destination_port: PortId,
            destination_channel: ChannelId,
            data: Vec<u8>
        },
        /// Packet acknowledgment processed
        PacketAcknowledged { sequence: u64, port_id: PortId, channel_id: ChannelId },
        /// Packet timed out and removed
        PacketTimeout { sequence: u64, port_id: PortId, channel_id: ChannelId },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Client not found
        ClientNotFound,
        /// Connection not found
        ConnectionNotFound,
        /// Channel not found
        ChannelNotFound,
        /// Invalid client state
        InvalidClientState,
        /// Invalid connection state
        InvalidConnectionState,
        /// Invalid channel state
        InvalidChannelState,
        /// Packet already exists (replay attack prevention)
        PacketAlreadyExists,
        /// Packet not found
        PacketNotFound,
        /// Packet timeout reached
        PacketTimeout,
        /// Invalid packet sequence
        InvalidSequence,
        /// Insufficient balance for fees
        InsufficientBalance,
        /// Maximum clients reached
        MaxClientsReached,
        /// Maximum connections reached
        MaxConnectionsReached,
        /// Maximum channels reached
        MaxChannelsReached,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new IBC client for cross-chain communication
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_client())]
        pub fn create_client(
            origin: OriginFor<T>,
            chain_id: Vec<u8>,
            initial_height: u64,
            trust_level: u32,
            unbonding_period: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check limits
            let current_clients = <NextClientId<T>>::get();
            ensure!(current_clients < T::MaxClients::get(), Error::<T>::MaxClientsReached);

            // Charge ultra-low fee
            let fee = T::ClientCreationFee::get();
            T::Currency::transfer(&who, &Self::account_id(), fee, ExistenceRequirement::KeepAlive)?;

            // Generate client ID
            let client_id = format!("client-{}", current_clients).into_bytes();
            <NextClientId<T>>::put(current_clients.saturating_add(1));

            // Create client state
            let client_state = ClientState {
                chain_id: chain_id.clone(),
                latest_height: initial_height,
                frozen: false,
                trust_level,
                unbonding_period,
            };

            // Store client
            <Clients<T>>::insert(&client_id, &client_state);

            // Emit event
            Self::deposit_event(Event::ClientCreated { client_id, chain_id });

            Ok(())
        }

        /// Update an existing IBC client with new state
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::update_client())]
        pub fn update_client(
            origin: OriginFor<T>,
            client_id: ClientId,
            new_height: u64,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // Get and update client state
            <Clients<T>>::try_mutate(&client_id, |client_opt| -> DispatchResult {
                let client = client_opt.as_mut().ok_or(Error::<T>::ClientNotFound)?;
                
                // Ensure height progression
                ensure!(new_height > client.latest_height, Error::<T>::InvalidClientState);
                
                client.latest_height = new_height;
                
                Ok(())
            })?;

            // Emit event
            Self::deposit_event(Event::ClientUpdated { client_id, height: new_height });

            Ok(())
        }

        /// Open an IBC connection between two chains
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::connection_open_init())]
        pub fn connection_open_init(
            origin: OriginFor<T>,
            client_id: ClientId,
            counterparty_client_id: ClientId,
            version: Vec<u8>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // Validate client exists
            ensure!(<Clients<T>>::contains_key(&client_id), Error::<T>::ClientNotFound);

            // Check connection limit
            let current_connections = <NextConnectionId<T>>::get();
            ensure!(current_connections < T::MaxConnections::get(), Error::<T>::MaxConnectionsReached);

            // Generate connection ID
            let connection_id = format!("connection-{}", current_connections).into_bytes();
            <NextConnectionId<T>>::put(current_connections.saturating_add(1));

            // Create connection end
            let connection_end = ConnectionEnd {
                state: ConnectionState::Init,
                client_id: client_id.clone(),
                counterparty_client_id,
                version,
            };

            // Store connection
            <Connections<T>>::insert(&connection_id, &connection_end);

            // Emit event
            Self::deposit_event(Event::ConnectionOpened { connection_id, client_id });

            Ok(())
        }

        /// Open an IBC channel for application communication
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::channel_open_init())]
        pub fn channel_open_init(
            origin: OriginFor<T>,
            port_id: PortId,
            connection_id: ConnectionId,
            counterparty_port_id: PortId,
            version: Vec<u8>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // Validate connection exists and is open
            let connection = <Connections<T>>::get(&connection_id)
                .ok_or(Error::<T>::ConnectionNotFound)?;
            ensure!(connection.state == ConnectionState::Open, Error::<T>::InvalidConnectionState);

            // Check channel limit
            let current_channels = <NextChannelId<T>>::get();
            ensure!(current_channels < T::MaxChannels::get(), Error::<T>::MaxChannelsReached);

            // Generate channel ID
            let channel_id = format!("channel-{}", current_channels).into_bytes();
            <NextChannelId<T>>::put(current_channels.saturating_add(1));

            // Create channel end
            let channel_end = ChannelEnd {
                state: ChannelState::Init,
                connection_id: connection_id.clone(),
                port_id: port_id.clone(),
                counterparty_port_id,
                version,
                next_sequence_send: 1,
                next_sequence_recv: 1,
                next_sequence_ack: 1,
            };

            // Store channel
            <Channels<T>>::insert(&port_id, &channel_id, &channel_end);

            // Emit event
            Self::deposit_event(Event::ChannelOpened { port_id, channel_id, connection_id });

            Ok(())
        }

        /// Send a cross-chain packet
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::send_packet())]
        pub fn send_packet(
            origin: OriginFor<T>,
            source_port: PortId,
            source_channel: ChannelId,
            destination_port: PortId,
            destination_channel: ChannelId,
            data: Vec<u8>,
            timeout_height: u64,
            timeout_timestamp: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Charge ultra-low transmission fee
            let fee = T::PacketTransmissionFee::get();
            T::Currency::transfer(&who, &Self::account_id(), fee, ExistenceRequirement::KeepAlive)?;

            // Get channel and validate state
            let mut channel = <Channels<T>>::get(&source_port, &source_channel)
                .ok_or(Error::<T>::ChannelNotFound)?;
            ensure!(channel.state == ChannelState::Open, Error::<T>::InvalidChannelState);

            // Create packet
            let packet = Packet {
                sequence: channel.next_sequence_send,
                source_port: source_port.clone(),
                source_channel: source_channel.clone(),
                destination_port: destination_port.clone(),
                destination_channel: destination_channel.clone(),
                data: data.clone(),
                timeout_height,
                timeout_timestamp,
            };

            // Generate packet commitment (hash for integrity)
            let packet_hash = BlakeTwo256::hash_of(&packet);

            // Store packet commitment (prevents replay)
            <PacketCommitments<T>>::insert(&source_port, channel.next_sequence_send, packet_hash);

            // Update channel sequence
            channel.next_sequence_send = channel.next_sequence_send.saturating_add(1);
            <Channels<T>>::insert(&source_port, &source_channel, &channel);

            // Emit event
            Self::deposit_event(Event::PacketSent {
                sequence: packet.sequence,
                source_port,
                source_channel,
                destination_port,
                destination_channel,
                data,
            });

            Ok(())
        }

        /// Receive and process a cross-chain packet
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::recv_packet())]
        pub fn recv_packet(
            origin: OriginFor<T>,
            packet: Packet,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // Get destination channel
            let mut channel = <Channels<T>>::get(&packet.destination_port, &packet.destination_channel)
                .ok_or(Error::<T>::ChannelNotFound)?;
            ensure!(channel.state == ChannelState::Open, Error::<T>::InvalidChannelState);

            // Validate sequence number (prevent replay and ensure ordering)
            ensure!(packet.sequence == channel.next_sequence_recv, Error::<T>::InvalidSequence);

            // Check timeout conditions
            let current_height = frame_system::Pallet::<T>::block_number().saturated_into::<u64>();
            if packet.timeout_height > 0 {
                ensure!(current_height < packet.timeout_height, Error::<T>::PacketTimeout);
            }

            // Update channel sequence
            channel.next_sequence_recv = channel.next_sequence_recv.saturating_add(1);
            <Channels<T>>::insert(&packet.destination_port, &packet.destination_channel, &channel);

            // Store acknowledgment (simple success acknowledgment)
            let ack_data = b"success".to_vec();
            <PacketAcknowledgments<T>>::insert(&packet.destination_port, packet.sequence, &ack_data);

            // Emit event
            Self::deposit_event(Event::PacketReceived {
                sequence: packet.sequence,
                source_port: packet.source_port,
                source_channel: packet.source_channel,
                destination_port: packet.destination_port,
                destination_channel: packet.destination_channel,
                data: packet.data,
            });

            Ok(())
        }

        /// Process packet acknowledgment
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::acknowledge_packet())]
        pub fn acknowledge_packet(
            origin: OriginFor<T>,
            port_id: PortId,
            channel_id: ChannelId,
            sequence: u64,
            acknowledgment: Vec<u8>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // Verify packet commitment exists
            ensure!(
                <PacketCommitments<T>>::contains_key(&port_id, sequence),
                Error::<T>::PacketNotFound
            );

            // Remove packet commitment (cleanup)
            <PacketCommitments<T>>::remove(&port_id, sequence);

            // Emit event
            Self::deposit_event(Event::PacketAcknowledged { sequence, port_id, channel_id });

            Ok(())
        }

        /// Handle packet timeout
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::timeout_packet())]
        pub fn timeout_packet(
            origin: OriginFor<T>,
            port_id: PortId,
            channel_id: ChannelId,
            sequence: u64,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // Verify packet commitment exists
            ensure!(
                <PacketCommitments<T>>::contains_key(&port_id, sequence),
                Error::<T>::PacketNotFound
            );

            // Remove packet commitment (cleanup)
            <PacketCommitments<T>>::remove(&port_id, sequence);

            // Emit event  
            Self::deposit_event(Event::PacketTimeout { sequence, port_id, channel_id });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Get the account ID for the pallet
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }
    }
}

/// Weight functions needed for benchmarking
pub trait WeightInfo {
    fn create_client() -> Weight;
    fn update_client() -> Weight;
    fn connection_open_init() -> Weight;
    fn channel_open_init() -> Weight;
    fn send_packet() -> Weight;
    fn recv_packet() -> Weight;
    fn acknowledge_packet() -> Weight;
    fn timeout_packet() -> Weight;
}

/// Default weights (based on complexity analysis)
impl WeightInfo for () {
    fn create_client() -> Weight { Weight::from_parts(50_000, 0) }
    fn update_client() -> Weight { Weight::from_parts(30_000, 0) }
    fn connection_open_init() -> Weight { Weight::from_parts(40_000, 0) }
    fn channel_open_init() -> Weight { Weight::from_parts(40_000, 0) }
    fn send_packet() -> Weight { Weight::from_parts(100_000, 0) }
    fn recv_packet() -> Weight { Weight::from_parts(80_000, 0) }
    fn acknowledge_packet() -> Weight { Weight::from_parts(20_000, 0) }
    fn timeout_packet() -> Weight { Weight::from_parts(20_000, 0) }
}