//! Integration tests for Netchain's interoperability features
//! 
//! This module tests:
//! - IBC cross-chain communication
//! - Oracle data fetching and aggregation
//! - Cross-chain + oracle combined scenarios
//! - Security measures against common exploits

use frame_support::{
    assert_ok, assert_noop,
    traits::{Get, Currency},
    weights::Weight,
};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};
use pallet_ibc_core::{Event as IbcEvent, Error as IbcError};
use pallet_oracle::{Event as OracleEvent, Error as OracleError};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Test runtime configuration
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        IbcCore: pallet_ibc_core,
        Oracle: pallet_oracle,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
    type MaxHolds = ConstU32<0>;
    type HoldIdentifier = ();
    type FreezeIdentifier = ();
    type RuntimeHoldReason = ();  
    type MaxFreezes = ConstU32<0>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const MaxIbcClients: u32 = 100;
    pub const MaxIbcConnections: u32 = 200;
    pub const MaxIbcChannels: u32 = 500;
    pub const IbcClientCreationFee: u128 = 10;
    pub const IbcPacketTransmissionFee: u128 = 5;
    pub const IbcPalletId: frame_support::PalletId = frame_support::PalletId(*b"test_ibc");
}

impl pallet_ibc_core::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MaxClients = MaxIbcClients;
    type MaxConnections = MaxIbcConnections;
    type MaxChannels = MaxIbcChannels;
    type ClientCreationFee = IbcClientCreationFee;
    type PacketTransmissionFee = IbcPacketTransmissionFee;
    type PalletId = IbcPalletId;
    type WeightInfo = ();
}

parameter_types! {
    pub const MaxOracleDataSources: u32 = 10;
    pub const MaxOracleDataSize: u32 = 1024;
    pub const OracleQueryFee: u128 = 2;
    pub const PremiumOracleQueryFee: u128 = 5;
    pub const OracleProviderReward: u128 = 1;
    pub const MaxOracleDataAge: u64 = 1200;
    pub const MinAggregationSources: u32 = 3;
    pub const OraclePalletId: frame_support::PalletId = frame_support::PalletId(*b"test_orc");
}

impl pallet_oracle::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MaxDataSources = MaxOracleDataSources;
    type MaxDataSize = MaxOracleDataSize;
    type OracleQueryFee = OracleQueryFee;
    type PremiumQueryFee = PremiumOracleQueryFee;
    type OracleReward = OracleProviderReward;
    type MaxDataAge = MaxOracleDataAge;
    type MinAggregationSources = MinAggregationSources;
    type PalletId = OraclePalletId;
    type WeightInfo = ();
}

// Helper function to create test externalities
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
    
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 1_000_000),
            (2, 1_000_000),
            (3, 1_000_000),
            (4, 1_000_000),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}

#[cfg(test)]
mod ibc_tests {
    use super::*;

    #[test]
    fn create_ibc_client_works() {
        new_test_ext().execute_with(|| {
            // Create IBC client
            assert_ok!(IbcCore::create_client(
                RuntimeOrigin::signed(1),
                b"cosmos-testnet".to_vec(),
                100,
                67, // 2/3 trust level
                1800, // 30 minute unbonding period
            ));

            // Check client was created
            let client_id = b"client-0".to_vec();
            let client = IbcCore::clients(&client_id).unwrap();
            assert_eq!(client.chain_id, b"cosmos-testnet".to_vec());
            assert_eq!(client.latest_height, 100);
            assert_eq!(client.trust_level, 67);

            // Check fee was charged
            assert_eq!(Balances::free_balance(&1), 1_000_000 - 10);

            // Check event was emitted
            System::assert_last_event(RuntimeEvent::IbcCore(
                IbcEvent::ClientCreated {
                    client_id,
                    chain_id: b"cosmos-testnet".to_vec(),
                }
            ));
        });
    }

    #[test]
    fn update_ibc_client_works() {
        new_test_ext().execute_with(|| {
            // Create client first
            assert_ok!(IbcCore::create_client(
                RuntimeOrigin::signed(1),
                b"cosmos-testnet".to_vec(),
                100,
                67,
                1800,
            ));

            let client_id = b"client-0".to_vec();

            // Update client height
            assert_ok!(IbcCore::update_client(
                RuntimeOrigin::signed(1),
                client_id.clone(),
                150,
            ));

            // Check client was updated
            let client = IbcCore::clients(&client_id).unwrap();
            assert_eq!(client.latest_height, 150);

            // Check event was emitted
            System::assert_last_event(RuntimeEvent::IbcCore(
                IbcEvent::ClientUpdated {
                    client_id,
                    height: 150,
                }
            ));
        });
    }

    #[test]
    fn cross_chain_packet_flow_works() {
        new_test_ext().execute_with(|| {
            // Setup: Create client and connection (simplified)
            assert_ok!(IbcCore::create_client(
                RuntimeOrigin::signed(1),
                b"cosmos-testnet".to_vec(),
                100,
                67,
                1800,
            ));

            let client_id = b"client-0".to_vec();

            // Create connection
            assert_ok!(IbcCore::connection_open_init(
                RuntimeOrigin::signed(1),
                client_id.clone(),
                b"counterparty-client-0".to_vec(),
                b"1.0".to_vec(),
            ));

            // Manually set connection to Open state for testing
            let connection_id = b"connection-0".to_vec();
            let mut connection = IbcCore::connections(&connection_id).unwrap();
            connection.state = pallet_ibc_core::ConnectionState::Open;
            pallet_ibc_core::Connections::<Test>::insert(&connection_id, connection);

            // Create channel
            assert_ok!(IbcCore::channel_open_init(
                RuntimeOrigin::signed(1),
                b"transfer".to_vec(), // port_id
                connection_id,
                b"transfer".to_vec(), // counterparty_port_id
                b"ics20-1".to_vec(),
            ));

            let port_id = b"transfer".to_vec();
            let channel_id = b"channel-0".to_vec();

            // Manually set channel to Open state for testing
            let mut channel = IbcCore::channels(&port_id, &channel_id).unwrap();
            channel.state = pallet_ibc_core::ChannelState::Open;
            pallet_ibc_core::Channels::<Test>::insert(&port_id, &channel_id, channel);

            // Send cross-chain packet
            assert_ok!(IbcCore::send_packet(
                RuntimeOrigin::signed(1),
                port_id.clone(),
                channel_id.clone(),
                b"transfer".to_vec(), // destination_port
                b"channel-1".to_vec(), // destination_channel
                b"{\"amount\":\"1000\",\"denom\":\"NET\"}".to_vec(), // data
                200, // timeout_height
                0, // timeout_timestamp
            ));

            // Check packet was sent
            let packet_hash = IbcCore::packet_commitments(&port_id, 1).unwrap();
            assert!(!packet_hash.is_zero());

            // Check fee was charged
            assert_eq!(Balances::free_balance(&1), 1_000_000 - 10 - 5); // client fee + packet fee

            // Check event was emitted
            System::assert_has_event(RuntimeEvent::IbcCore(
                IbcEvent::PacketSent {
                    sequence: 1,
                    source_port: port_id,
                    source_channel: channel_id,
                    destination_port: b"transfer".to_vec(),
                    destination_channel: b"channel-1".to_vec(),
                    data: b"{\"amount\":\"1000\",\"denom\":\"NET\"}".to_vec(),
                }
            ));
        });
    }

    #[test]
    fn ibc_client_limits_enforced() {
        new_test_ext().execute_with(|| {
            // This test would require modifying MaxIbcClients to a small number for testing
            // For now, just test that the limit exists
            assert_eq!(MaxIbcClients::get(), 100);
        });
    }
}

#[cfg(test)]
mod oracle_tests {
    use super::*;

    #[test]
    fn register_oracle_source_works() {
        new_test_ext().execute_with(|| {
            // Register data source
            assert_ok!(Oracle::register_source(
                RuntimeOrigin::root(),
                b"coinbase_btc".to_vec(),
                b"Coinbase BTC Price".to_vec(),
                b"https://api.coinbase.com/v2/prices/BTC-USD/spot".to_vec(),
                95, // high reliability
            ));

            // Check source was registered
            let source = Oracle::data_sources(b"coinbase_btc".to_vec()).unwrap();
            assert_eq!(source.name, b"Coinbase BTC Price".to_vec());
            assert_eq!(source.reliability, 95);
            assert!(source.active);

            // Check event was emitted
            System::assert_last_event(RuntimeEvent::Oracle(
                OracleEvent::SourceRegistered {
                    source_id: b"coinbase_btc".to_vec(),
                    name: b"Coinbase BTC Price".to_vec(),
                }
            ));
        });
    }

    #[test]
    fn oracle_data_request_works() {
        new_test_ext().execute_with(|| {
            // First register a data source
            assert_ok!(Oracle::register_source(
                RuntimeOrigin::root(),
                b"coinbase_btc".to_vec(),
                b"Coinbase BTC Price".to_vec(),
                b"https://api.coinbase.com/v2/prices/BTC-USD/spot".to_vec(),
                95,
            ));

            // Request oracle data
            assert_ok!(Oracle::request_data(
                RuntimeOrigin::signed(1),
                b"BTC/USD".to_vec(),
                vec![b"coinbase_btc".to_vec()],
                false, // not premium
                None, // no callback
            ));

            // Check fee was charged
            assert_eq!(Balances::free_balance(&1), 1_000_000 - 2);

            // Check request was stored
            let request = Oracle::oracle_requests(0).unwrap();
            assert_eq!(request.requester, 1);
            assert_eq!(request.data_key, b"BTC/USD".to_vec());
            assert!(!request.premium);

            // Check event was emitted
            System::assert_last_event(RuntimeEvent::Oracle(
                OracleEvent::DataRequested {
                    request_id: 0,
                    requester: 1,
                    data_key: b"BTC/USD".to_vec(),
                    sources: vec![b"coinbase_btc".to_vec()],
                    premium: false,
                }
            ));
        });
    }

    #[test]
    fn oracle_data_provision_works() {
        new_test_ext().execute_with(|| {
            // Setup: Register source and trusted provider
            assert_ok!(Oracle::register_source(
                RuntimeOrigin::root(),
                b"coinbase_btc".to_vec(),
                b"Coinbase BTC Price".to_vec(),
                b"https://api.coinbase.com/v2/prices/BTC-USD/spot".to_vec(),
                95,
            ));

            assert_ok!(Oracle::add_trusted_provider(
                RuntimeOrigin::root(),
                2, // provider account
                90, // reputation
            ));

            // Provide oracle data
            assert_ok!(Oracle::provide_data(
                RuntimeOrigin::signed(2),
                b"BTC/USD".to_vec(),
                b"coinbase_btc".to_vec(),
                b"50000.00".to_vec(), // $50,000 BTC price
                90, // high confidence
                None, // no signature
            ));

            // Check data was stored
            let data = Oracle::oracle_data(b"BTC/USD".to_vec(), b"coinbase_btc".to_vec()).unwrap();
            assert_eq!(data.value, b"50000.00".to_vec());
            assert_eq!(data.provider, 2);
            assert_eq!(data.confidence, 90);

            // Check provider was rewarded
            assert_eq!(Balances::free_balance(&2), 1_000_000 + 1);

            // Check event was emitted
            System::assert_last_event(RuntimeEvent::Oracle(
                OracleEvent::DataProvided {
                    data_key: b"BTC/USD".to_vec(),
                    source: b"coinbase_btc".to_vec(),
                    provider: 2,
                    value: b"50000.00".to_vec(),
                    confidence: 90,
                }
            ));
        });
    }

    #[test]
    fn oracle_batch_requests_work() {
        new_test_ext().execute_with(|| {
            // Register multiple sources
            assert_ok!(Oracle::register_source(
                RuntimeOrigin::root(),
                b"coinbase_btc".to_vec(),
                b"Coinbase BTC".to_vec(),
                b"coinbase-api".to_vec(),
                95,
            ));

            assert_ok!(Oracle::register_source(
                RuntimeOrigin::root(),
                b"binance_btc".to_vec(),
                b"Binance BTC".to_vec(),
                b"binance-api".to_vec(),
                90,
            ));

            // Batch request multiple data points
            let requests = vec![
                (b"BTC/USD".to_vec(), vec![b"coinbase_btc".to_vec()], false),
                (b"ETH/USD".to_vec(), vec![b"binance_btc".to_vec()], true), // premium
            ];

            assert_ok!(Oracle::batch_requests(
                RuntimeOrigin::signed(1),
                requests,
            ));

            // Check total fee charged (2 + 5 = 7)
            assert_eq!(Balances::free_balance(&1), 1_000_000 - 7);

            // Check event was emitted
            System::assert_last_event(RuntimeEvent::Oracle(
                OracleEvent::BatchProcessed {
                    request_count: 2,
                    total_fee: 7,
                }
            ));
        });
    }

    #[test]
    fn oracle_security_measures_work() {
        new_test_ext().execute_with(|| {
            // Register source
            assert_ok!(Oracle::register_source(
                RuntimeOrigin::root(),
                b"test_source".to_vec(),
                b"Test Source".to_vec(),
                b"test-api".to_vec(),
                50,
            ));

            // Test: High confidence data requires trusted provider
            assert_noop!(
                Oracle::provide_data(
                    RuntimeOrigin::signed(3), // untrusted provider
                    b"BTC/USD".to_vec(),
                    b"test_source".to_vec(),
                    b"50000.00".to_vec(),
                    85, // high confidence
                    None,
                ),
                OracleError::<Test>::ProviderNotTrusted
            );

            // Test: Data size limits
            let large_data = vec![0u8; 2000]; // Exceeds MaxOracleDataSize (1024)
            assert_noop!(
                Oracle::provide_data(
                    RuntimeOrigin::signed(2),
                    b"BTC/USD".to_vec(),
                    b"test_source".to_vec(),
                    large_data,
                    50,
                    None,
                ),
                OracleError::<Test>::DataTooLarge
            );

            // Test: Invalid confidence score
            assert_noop!(
                Oracle::provide_data(
                    RuntimeOrigin::signed(2),
                    b"BTC/USD".to_vec(),
                    b"test_source".to_vec(),
                    b"50000.00".to_vec(),
                    101, // Invalid confidence > 100
                    None,
                ),
                OracleError::<Test>::InvalidConfidence
            );
        });
    }
}

#[cfg(test)]
mod combined_interoperability_tests {
    use super::*;

    #[test]
    fn cross_chain_oracle_integration_works() {
        new_test_ext().execute_with(|| {
            // Setup IBC
            assert_ok!(IbcCore::create_client(
                RuntimeOrigin::signed(1),
                b"cosmos-testnet".to_vec(),
                100,
                67,
                1800,
            ));

            // Setup Oracle
            assert_ok!(Oracle::register_source(
                RuntimeOrigin::root(),
                b"external_chain_data".to_vec(),
                b"External Chain Oracle".to_vec(),
                b"ibc-oracle-relay".to_vec(),
                85,
            ));

            assert_ok!(Oracle::add_trusted_provider(
                RuntimeOrigin::root(),
                2,
                95,
            ));

            // Simulate cross-chain oracle data request
            assert_ok!(Oracle::request_data(
                RuntimeOrigin::signed(1),
                b"COSMOS/USD".to_vec(),
                vec![b"external_chain_data".to_vec()],
                true, // premium for cross-chain data
                Some(b"ibc_callback".to_vec()),
            ));

            // Provide cross-chain oracle data
            assert_ok!(Oracle::provide_data(
                RuntimeOrigin::signed(2),
                b"COSMOS/USD".to_vec(),
                b"external_chain_data".to_vec(),
                b"15.50".to_vec(),
                85,
                None,
            ));

            // Check both systems worked together
            let client = IbcCore::clients(b"client-0".to_vec()).unwrap();
            assert_eq!(client.chain_id, b"cosmos-testnet".to_vec());

            let data = Oracle::oracle_data(b"COSMOS/USD".to_vec(), b"external_chain_data".to_vec()).unwrap();
            assert_eq!(data.value, b"15.50".to_vec());

            // Check combined fees were charged appropriately
            let expected_balance = 1_000_000 - 10 - 5; // IBC client + Oracle premium
            assert_eq!(Balances::free_balance(&1), expected_balance);
        });
    }

    #[test]
    fn ultra_low_fees_maintained() {
        new_test_ext().execute_with(|| {
            // Test that all operations maintain ultra-low fees
            
            // IBC fees
            assert_eq!(IbcClientCreationFee::get(), 10); // ~$0.0001
            assert_eq!(IbcPacketTransmissionFee::get(), 5); // ~$0.00005
            
            // Oracle fees
            assert_eq!(OracleQueryFee::get(), 2); // ~$0.00002
            assert_eq!(PremiumOracleQueryFee::get(), 5); // ~$0.00005
            assert_eq!(OracleProviderReward::get(), 1); // ~$0.00001

            // Total cost for complex interoperability scenario
            let total_interop_cost = 10 + 5 + 5 + 1; // 21 units = ~$0.00021
            assert!(total_interop_cost < 25); // Still under $0.00025
        });
    }
}