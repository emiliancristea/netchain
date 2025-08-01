#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod apis;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod configs;
pub mod performance;
#[cfg(test)]
mod tests;

extern crate alloc;
use alloc::vec::Vec;
use sp_runtime::{
	generic, impl_opaque_keys,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	MultiAddress, MultiSignature,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

// Export runtime types for genesis configuration
pub use pallet_babe::GenesisConfig as BabeConfig;
pub use pallet_balances::GenesisConfig as BalancesConfig;
pub use pallet_contracts::GenesisConfig as ContractsConfig;
pub use pallet_grandpa::GenesisConfig as GrandpaConfig;
pub use pallet_session::GenesisConfig as SessionConfig;
pub use pallet_staking::{GenesisConfig as StakingConfig, StakerStatus};
pub use pallet_sudo::GenesisConfig as SudoConfig;
pub use frame_system::GenesisConfig as SystemConfig;

pub mod genesis_config_presets;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;
	use sp_runtime::{
		generic,
		traits::{BlakeTwo256, Hash as HashT},
	};

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
	/// Opaque block hash type.
	pub type Hash = <BlakeTwo256 as HashT>::Output;
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub babe: Babe,
		pub grandpa: Grandpa,
	}
}

// To learn more about runtime versioning, see:
// https://docs.substrate.io/main-docs/build/upgrade#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: alloc::borrow::Cow::Borrowed("netchain"),
	impl_name: alloc::borrow::Cow::Borrowed("netchain"),
	authoring_version: 1,
	// The version of the runtime specification. A full node will not attempt to use its native
	//   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
	//   `spec_version`, and `authoring_version` are the same between Wasm and native.
	// This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
	//   the compatible custom types.
	spec_version: 100,
	impl_version: 1,
	apis: apis::RUNTIME_API_VERSIONS,
	transaction_version: 1,
	system_version: 1,
};

mod block_times {
	/// This determines the average expected block time that we are targeting. 
	/// For Netchain, we target 3 second blocks for optimal performance.
	/// This can be adjusted from 1-6 seconds based on network conditions.
	///
	/// Change this to adjust the block time.
	pub const MILLI_SECS_PER_BLOCK: u64 = 3000; // 3 second blocks for high performance

	// For BABE, we use epoch duration instead of slot duration
	pub const SLOT_DURATION: u64 = MILLI_SECS_PER_BLOCK;
	
	// BABE expects the next block to be authored SLOT_DURATION ms after the current block
	pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 10 * MINUTES;
}
pub use block_times::*;

// Staking configuration
pub const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
pub const STASH: Balance = ENDOWMENT / 1000;

// BABE Genesis configuration
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
	sp_consensus_babe::BabeEpochConfiguration {
		c: PRIMARY_PROBABILITY,
		allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
	};

// Primary probability for BABE block production
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLI_SECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

pub const BLOCK_HASH_COUNT: BlockNumber = 2400;

// Unit = the base number of indivisible units for balances
pub const UNIT: Balance = 1_000_000_000_000;
pub const MILLI_UNIT: Balance = 1_000_000_000;
pub const MICRO_UNIT: Balance = 1_000_000;

/// Existential deposit.
pub const EXISTENTIAL_DEPOSIT: Balance = MILLI_UNIT;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The `TransactionExtension` to the basic transaction logic.
pub type TxExtension = (
	frame_system::AuthorizeCall<Runtime>,
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
	frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
	frame_system::WeightReclaim<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, TxExtension>;

/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, TxExtension>;

/// All migrations of the runtime, aside from the ones declared in the pallets.
///
/// This can be a tuple of types, each implementing `OnRuntimeUpgrade`.
#[allow(unused_parens)]
type Migrations = ();

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	Migrations,
>;

// Create the runtime by composing the FRAME pallets that were previously configured.
#[frame_support::runtime]
mod runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeSlashReason,
		RuntimeLockId,
		RuntimeTask,
		RuntimeViewFunction
	)]
	pub struct Runtime;

	#[runtime::pallet_index(0)]
	pub type System = frame_system;

	#[runtime::pallet_index(1)]
	pub type Timestamp = pallet_timestamp;

	#[runtime::pallet_index(2)]
	pub type Babe = pallet_babe;

	#[runtime::pallet_index(3)]
	pub type Grandpa = pallet_grandpa;

	#[runtime::pallet_index(4)]
	pub type Balances = pallet_balances;

	#[runtime::pallet_index(5)]
	pub type TransactionPayment = pallet_transaction_payment;

	#[runtime::pallet_index(6)]
	pub type Authorship = pallet_authorship;

	#[runtime::pallet_index(7)]
	pub type Staking = pallet_staking;

	#[runtime::pallet_index(8)]
	pub type Session = pallet_session;

	#[runtime::pallet_index(9)]
	pub type Offences = pallet_offences;

	#[runtime::pallet_index(10)]
	pub type Sudo = pallet_sudo;

	// Smart Contracts support with Ink!
	#[runtime::pallet_index(11)]
	pub type Contracts = pallet_contracts;

	// Keep Aura for compatibility (can be removed later)
	#[runtime::pallet_index(12)]
	pub type Aura = pallet_aura;

	// Include the custom logic from the pallet-template in the runtime.
	#[runtime::pallet_index(13)]
	pub type Template = pallet_template;

	// IBC Core for cross-chain communication
	#[runtime::pallet_index(14)]
	pub type IbcCore = pallet_ibc_core;

	// Oracle for off-chain data integration
	#[runtime::pallet_index(15)]
	pub type Oracle = pallet_oracle;


}
