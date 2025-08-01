// This is free and unencumbered software released into the public domain.
//
// Anyone is free to copy, modify, publish, use, compile, sell, or
// distribute this software, either in source code form or as a compiled
// binary, for any purpose, commercial or non-commercial, and by any
// means.
//
// In jurisdictions that recognize copyright laws, the author or authors
// of this software dedicate any and all copyright interest in the
// software to the public domain. We make this dedication for the benefit
// of the public at large and to the detriment of our heirs and
// successors. We intend this dedication to be an overt act of
// relinquishment in perpetuity of all present and future rights to this
// software under copyright law.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
// OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
// ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
// OTHER DEALINGS IN THE SOFTWARE.
//
// For more information, please refer to <http://unlicense.org>

// Substrate and Polkadot dependencies
use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstBool, ConstU128, ConstU32, ConstU64, ConstU8, VariantCountOf, Get, KeyOwnerProofSystem, Randomness},
	weights::{
		constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
		IdentityFee, Weight,
	},
	PalletId,
};
use frame_system::limits::{BlockLength, BlockWeights};
use pallet_session::historical as pallet_session_historical;
use pallet_transaction_payment::{ConstFeeMultiplier, FungibleAdapter, Multiplier};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	curve::PiecewiseLinear,
	traits::{One, OpaqueKeys, SaturatedConversion, Zero},
	transaction_validity::TransactionPriority,
	Perbill, Perquintill,
};
use sp_staking::{SessionIndex, EraIndex};
use sp_version::RuntimeVersion;

// Local module imports
use super::{
	AccountId, Aura, Babe, Balance, Balances, Block, BlockNumber, Contracts, Hash, Nonce, PalletInfo, Runtime,
	RuntimeCall, RuntimeEvent, RuntimeFreezeReason, RuntimeHoldReason, RuntimeOrigin, RuntimeTask,
	Session, SessionKeys, Sharding, Staking, System, ENDOWMENT, EPOCH_DURATION_IN_BLOCKS, EXISTENTIAL_DEPOSIT, 
	SLOT_DURATION, STASH, VERSION,
};

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	pub const Version: RuntimeVersion = VERSION;

	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::with_sensible_defaults(
		Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
		NORMAL_DISPATCH_RATIO,
	);
	pub RuntimeBlockLength: BlockLength = BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
}

/// The default types are being injected by [`derive_impl`](`frame_support::derive_impl`) from
/// [`SoloChainDefaultConfig`](`struct@frame_system::config_preludes::SolochainDefaultConfig`),
/// but overridden as needed.
#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
	/// The block type for the runtime.
	type Block = Block;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The type for storing how many extrinsics an account has signed.
	type Nonce = Nonce;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

// Staking and session parameters
parameter_types! {
	pub const SessionsPerEra: sp_staking::SessionIndex = 6;
	pub const BondingDuration: sp_staking::EraIndex = 24 * 28; // 28 days
	pub const SlashDeferDuration: sp_staking::EraIndex = 24 * 7; // 7 days
	pub const MaxNominatorRewardedPerValidator: u32 = 256;
	pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(33);
	pub const MaxNominations: u32 = 16;
}

// BABE Configuration for PoS consensus
parameter_types! {
	pub const EpochDuration: u64 = EPOCH_DURATION_IN_BLOCKS as u64;
	pub const ExpectedBlockTime: u64 = SLOT_DURATION;
	pub const ReportLongevity: u64 = 
		BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
}

impl pallet_babe::Config for Runtime {
	type EpochDuration = EpochDuration;
	type ExpectedBlockTime = ExpectedBlockTime;
	type EpochChangeTrigger = pallet_babe::ExternalTrigger;
	type DisabledValidators = Session;
	type WeightInfo = ();
	type MaxAuthorities = ConstU32<100>; // Support up to 100 validators
	type MaxNominators = ConstU32<1000>; // Support up to 1000 nominators  
	type KeyOwnerProof = sp_core::Void; // Simplified for now
	type EquivocationReportSystem = (); // Simplified for now
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<32>;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
	type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type WeightInfo = ();
	type MaxAuthorities = ConstU32<32>;
	type MaxNominators = ConstU32<0>;
	type MaxSetIdSessionEntries = ConstU64<0>;

	type KeyOwnerProof = sp_core::Void;
	type EquivocationReportSystem = ();
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Babe;
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type DoneSlashHandler = ();
}

// Low-fee transaction payment parameters for Netchain
parameter_types! {
	/// Ultra-low transaction fees: 1 unit per byte (adjustable to near-zero)
	pub const TransactionByteFee: Balance = 1;
	/// Minimal weight fee - nearly free transactions
	pub const WeightToFeeConstant: Balance = 1;
	/// Keep fee multiplier stable for predictable low costs
	pub FeeMultiplier: Multiplier = Multiplier::one();
}

/// Ultra-low fee calculation: flat fee per byte
pub struct UltraLowFeeCalculator;
impl frame_support::weights::WeightToFee for UltraLowFeeCalculator {
	type Balance = Balance;

	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		// Convert weight to a minimal fee - nearly free transactions
		weight.ref_time().saturated_into::<Balance>().saturating_div(1_000_000)
	}
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = FungibleAdapter<Balances, ()>;
	type OperationalFeeMultiplier = ConstU8<5>;
	/// Ultra-low weight-based fees
	type WeightToFee = UltraLowFeeCalculator;
	/// Flat fee per byte: 1 unit per byte (can be adjusted to 0 if needed)
	type LengthToFee = frame_support::weights::ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
	type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

// Session and Historical configurations
parameter_types! {
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
}

type HistoricalSession = pallet_session_historical::NoteHistoricalRoot<Self, Staking>;

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_staking::StashOf<Self>;
	type ShouldEndSession = pallet_babe::ShouldEndSession<Runtime>;
	type NextSessionRotation = pallet_babe::NextSessionRotation<Runtime>;
	type SessionManager = HistoricalSession;
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

// Authorship configuration
impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_babe::FindAuthor<Babe>;
	type EventHandler = (Staking,);
}

// Offences configuration  
impl pallet_offences::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
	type OnOffenceHandler = Staking;
}

// Staking reward curve - more rewards for optimal validator count
pallet_staking_reward_curve::build! {
	const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
		min_inflation: 0_025_000,
		max_inflation: 0_100_000, 
		ideal_stake: 0_500_000,
		falloff: 0_050_000,
		max_piece_count: 40,
		test_precision: 0_005_000,
	);
}

parameter_types! {
	// Staking parameters optimized for high performance PoS
	pub const SessionsPerEra: sp_staking::SessionIndex = 6;
	pub const BondingDuration: sp_staking::EraIndex = 24 * 7; // 7 days (shorter for faster governance)
	pub const SlashDeferDuration: sp_staking::EraIndex = 24; // 1 day
	pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
	pub const MaxExposurePageSize: u32 = 256;
	pub const MaxNominations: u32 = 16;
	pub const StakingPalletId: PalletId = PalletId(*b"py/stake");
	pub const MaxUnlockingChunks: u32 = 32;
}

impl pallet_staking::Config for Runtime {
	type Currency = Balances;
	type CurrencyBalance = Balance;
	type UnixTime = Timestamp;
	type CurrencyToVote = sp_staking::currency_to_vote::U128CurrencyToVote;
	type RewardRemainder = ();
	type RuntimeEvent = RuntimeEvent;
	type Slash = (); // No slashing destination for now
	type Reward = (); // Rewards go to stakers directly
	type SessionsPerEra = SessionsPerEra;
	type BondingDuration = BondingDuration;
	type SlashDeferDuration = SlashDeferDuration;
	type AdminOrigin = frame_system::EnsureRoot<AccountId>;
	type SessionInterface = Self;
	type EraPayout = pallet_staking::ConvertCurve<RewardCurve>;
	type NextNewSession = Session;
	type MaxExposurePageSize = MaxExposurePageSize;
	type MaxControllersInDeprecationBatch = ConstU32<100>;
	type ElectionProvider = frame_election_provider_support::NoElection<(AccountId, BlockNumber, Staking, ())>;
	type GenesisElectionProvider = Self::ElectionProvider;
	type VoterList = pallet_staking::UseNominatorsAndValidatorsMap<Runtime>;
	type TargetList = pallet_staking::UseValidatorsMap<Runtime>;
	type NominationsQuota = pallet_staking::FixedNominationsQuota<MaxNominations>;
	type MaxUnlockingChunks = MaxUnlockingChunks;
	type HistoryDepth = ConstU32<84>; // 84 eras (about 28 days)
	type EventListeners = ();
	type BenchmarkingConfig = pallet_staking::TestBenchmarkingConfig;
	type WeightInfo = pallet_staking::weights::SubstrateWeight<Runtime>;
	type DisablingStrategy = pallet_staking::UpToLimitDisablingStrategy<ConstU32<3>>;
}

// Smart Contracts Configuration - Ultra-low gas for high performance
parameter_types! {
	/// Maximum size of a contract in bytes (1 MB)
	pub const MaxCodeLen: u32 = 1024 * 1024;
	/// Maximum size of storage items
	pub const MaxStorageKeyLen: u32 = 128;
	/// Deposit per byte for storing code
	pub const CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(0);
	/// Ultra-low deposit per storage item (almost free)
	pub const DefaultDepositLimit: Balance = 1000;
	/// Maximum gas per block for contracts - high for throughput
	pub const BlockGasLimit: u64 = 10_000_000_000;
	/// Maximum gas per call - generous for complex contracts
	pub const CallStackLimit: u32 = 1024;
	/// Storage deposit limit for instantiation
	pub const StorageDepositLimit: Balance = Balance::MAX >> 1;
	/// Ultra-low instantiation fee
	pub const InstantiationFee: Balance = 1;
}

/// Contracts pallet configuration optimized for ultra-low fees
impl pallet_contracts::Config for Runtime {
	type Time = Timestamp;
	type Randomness = pallet_babe::RandomnessFromOneEpochAgo<Runtime>;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	
	/// Ultra-low call filter - allow all calls for maximum flexibility
	type CallFilter = frame_support::traits::Nothing;
	
	/// Deposit configuration - ultra-low for affordable smart contracts
	type DepositPerByte = ConstU128<1>; // 1 unit per byte
	type DepositPerItem = ConstU128<1>; // 1 unit per storage item
	type DefaultDepositLimit = DefaultDepositLimit;
	
	/// Contract size limits - generous for complex applications
	type MaxCodeLen = MaxCodeLen;
	type MaxStorageKeyLen = MaxStorageKeyLen;
	
	/// Gas configuration - high limits with ultra-low costs
	type Schedule = pallet_contracts::DefaultSchedule<Runtime>;
	type CallStack = [pallet_contracts::Frame<Runtime>; 1024];
	type WeightPrice = pallet_transaction_payment::Pallet<Runtime>;
	type WeightInfo = pallet_contracts::weights::SubstrateWeight<Runtime>;
	type ChainExtension = ();
	type AddressGenerator = pallet_contracts::DefaultAddressGenerator;
	type MaxDebugBufferLen = ConstU32<262144>; // 256 KB debug buffer
	type UnsafeUnstableInterface = ConstBool<false>; // Production safety
	type UploadOrigin = frame_system::EnsureRoot<AccountId>;
	type InstantiateOrigin = frame_system::EnsureSigned<AccountId>;
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type MaxDelegateDependencies = ConstU32<32>;
	type RuntimeHoldReason = RuntimeHoldReason;
	type Migrations = ();
	type Debug = ();
	type Environment = ();
	type ApiVersion = ();
	type Xcm = ();
}

// High-Performance Sharding Configuration
parameter_types! {
	/// Maximum validators per shard for optimal performance
	pub const MaxValidatorsPerShard: u32 = 25; // 100 total validators across 4 shards
	/// Target TPS per shard (25,000 each = 100,000 total)
	pub const TargetTpsPerShard: u32 = 25_000;
	/// Cross-shard transaction fee (ultra-low)
	pub const CrossShardFee: Balance = 10; // 10 units for cross-shard txs
	/// Sharding pallet identifier
	pub const ShardingPalletId: PalletId = PalletId(*b"netshrd!");
}

/// Sharding pallet configuration for massive scalability
impl pallet_sharding::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MaxValidatorsPerShard = MaxValidatorsPerShard;
	type TargetTpsPerShard = TargetTpsPerShard;
	type CrossShardFee = CrossShardFee;
	type PalletId = ShardingPalletId;
	type WeightInfo = ();
}

/// Configure the pallet-template in pallets/template.
impl pallet_template::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_template::weights::SubstrateWeight<Runtime>;
}

// ===== IBC and Oracle Configuration =====

parameter_types! {
	/// Maximum IBC clients per chain
	pub const MaxIbcClients: u32 = 100;
	/// Maximum IBC connections per client  
	pub const MaxIbcConnections: u32 = 200;
	/// Maximum IBC channels per connection
	pub const MaxIbcChannels: u32 = 500;
	/// Ultra-low IBC client creation fee (10 units = ~$0.0001)
	pub const IbcClientCreationFee: Balance = 10;
	/// Ultra-low cross-chain packet transmission fee (5 units = ~$0.00005)
	pub const IbcPacketTransmissionFee: Balance = 5;
	/// IBC pallet identifier
	pub const IbcPalletId: PalletId = PalletId(*b"netchain_ibc");
}

/// IBC Core pallet configuration for cross-chain communication
impl pallet_ibc_core::Config for Runtime {
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
	/// Maximum data sources per oracle request
	pub const MaxOracleDataSources: u32 = 10;
	/// Maximum size of oracle data (1KB)
	pub const MaxOracleDataSize: u32 = 1024;
	/// Ultra-low oracle query fee (2 units = ~$0.00002)
	pub const OracleQueryFee: Balance = 2;
	/// Premium oracle query fee (5 units = ~$0.00005)
	pub const PremiumOracleQueryFee: Balance = 5;
	/// Oracle provider reward (1 unit = ~$0.00001)
	pub const OracleProviderReward: Balance = 1;
	/// Maximum age of oracle data (1 hour = 1200 blocks)
	pub const MaxOracleDataAge: u64 = 1200;
	/// Minimum sources for data aggregation
	pub const MinAggregationSources: u32 = 3;
	/// Oracle pallet identifier
	pub const OraclePalletId: PalletId = PalletId(*b"netchain_oracle");
}

/// Oracle pallet configuration for off-chain data integration
impl pallet_oracle::Config for Runtime {
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
