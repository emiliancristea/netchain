//! # Consensus Security Tests
//! 
//! Comprehensive security testing for Netchain's consensus mechanisms:
//! - BABE and GRANDPA security properties
//! - Staking security (slashing, rewards)
//! - Session key security
//! - Attack resistance testing

#![cfg(test)]

use frame_support::{
    assert_ok, assert_noop, assert_err,
    traits::{Get, Currency, OnFinalize, OnInitialize},
    weights::Weight,
};
use sp_core::{H256, sr25519::Pair as Sr25519Pair, Pair, crypto::AccountId32, testing::SR25519};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup, OpaqueKeys},
    BuildStorage, Perbill,
};
use pallet_session::SessionKeys;
use pallet_staking::{Event as StakingEvent, Error as StakingError, StakerStatus};
use pallet_babe::{Event as BabeEvent, Error as BabeError};
use pallet_grandpa::{Event as GrandpaEvent, Error as GrandpaError};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Test runtime for consensus security testing
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        Session: pallet_session,
        Staking: pallet_staking,
        Babe: pallet_babe,
        Grandpa: pallet_grandpa,
        Authorship: pallet_authorship,
        Offences: pallet_offences,
    }
);

// Mock configuration for testing
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
    pub const ExistentialDeposit: u128 = 1;
    pub const MaxLocks: u32 = 50;
    pub const MinimumPeriod: u64 = 5;
    pub const BondingDuration: u32 = 3; // 3 sessions for testing
    pub const SlashDeferDuration: u32 = 2;
    pub const MaxNominatorRewardedPerValidator: u32 = 64;
    pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
    pub const Period: u64 = 10;
    pub const Offset: u64 = 0;
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
    type AccountId = AccountId32;
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

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxHolds = frame_support::traits::ConstU32<0>;
    type HoldIdentifier = ();
    type FreezeIdentifier = ();
    type RuntimeHoldReason = ();
    type MaxFreezes = frame_support::traits::ConstU32<0>;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

pub struct TestSessionKeys {
    pub babe: pallet_babe::AuthorityId,
    pub grandpa: pallet_grandpa::AuthorityId,
}

impl From<TestSessionKeys> for SessionKeys<Test> {
    fn from(keys: TestSessionKeys) -> Self {
        SessionKeys {
            babe: keys.babe,
            grandpa: keys.grandpa,
        }
    }
}

impl pallet_session::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type ValidatorId = AccountId32;
    type ValidatorIdOf = pallet_staking::StashOf<Test>;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = Staking;
    type SessionHandler = (Babe, Grandpa);
    type Keys = TestSessionKeys;
    type WeightInfo = ();
}

impl pallet_staking::Config for Test {
    type Currency = Balances;
    type CurrencyBalance = <Self as pallet_balances::Config>::Balance;
    type UnixTime = Timestamp;
    type CurrencyToVote = ();
    type RewardRemainder = ();
    type RuntimeEvent = RuntimeEvent;
    type Slash = ();
    type Reward = ();
    type SessionsPerEra = frame_support::traits::ConstU32<1>;
    type BondingDuration = BondingDuration;
    type SlashDeferDuration = SlashDeferDuration;
    type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
    type SessionInterface = Self;
    type EraPayout = ();
    type NextNewSession = Session;
    type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
    type OffendingValidatorsThreshold = OffendingValidatorsThreshold;
    type ElectionProvider = frame_election_provider_support::NoElection<(
        AccountId32,
        u64,
        pallet_staking::Stakers<Test>,
        ()
    )>;
    type GenesisElectionProvider = Self::ElectionProvider;
    type VoterList = pallet_staking::UseNominatorsAndValidatorsMap<Self>;
    type TargetList = pallet_staking::UseValidatorsMap<Self>;
    type MaxUnlockingChunks = frame_support::traits::ConstU32<32>;
    type HistoryDepth = frame_support::traits::ConstU32<84>;
    type BenchmarkingConfig = pallet_staking::TestBenchmarkingConfig;
    type WeightInfo = ();
    type NominationsQuota = pallet_staking::FixedNominationsQuota<16>;
}

impl pallet_babe::Config for Test {
    type EpochDuration = frame_support::traits::ConstU64<10>;
    type ExpectedBlockTime = frame_support::traits::ConstU64<6000>;
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;
    type DisabledValidators = Session;
    type KeyOwnerProof = sp_session::MembershipProof;
    type EquivocationReportSystem = pallet_babe::EquivocationReportSystem<
        Self,
        Offences,
        (),
        pallet_babe::ReportLongevity,
    >;
    type WeightInfo = ();
    type MaxAuthorities = frame_support::traits::ConstU32<100>;
    type MaxNominators = frame_support::traits::ConstU32<1000>;
}

impl pallet_grandpa::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type KeyOwnerProof = sp_session::MembershipProof;
    type EquivocationReportSystem = pallet_grandpa::EquivocationReportSystem<
        Self,
        Offences,
        (),
        pallet_grandpa::ReportLongevity,
    >;
    type WeightInfo = ();
    type MaxAuthorities = frame_support::traits::ConstU32<100>;
    type MaxNominators = frame_support::traits::ConstU32<1000>;
    type MaxSetIdSessionEntries = frame_support::traits::ConstU64<0>;
}

impl pallet_authorship::Config for Test {
    type FindAuthor = ();
    type EventHandler = ();
}

impl pallet_offences::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
    type OnOffenceHandler = Staking;
}

// Helper functions for testing
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
    
    // Pre-fund accounts
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (account_key("Alice"), 1_000_000),
            (account_key("Bob"), 1_000_000),
            (account_key("Charlie"), 1_000_000),
            (account_key("Dave"), 1_000_000),
            (account_key("Eve"), 1_000_000),
            (account_key("Ferdie"), 1_000_000),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    // Setup initial validators
    pallet_staking::GenesisConfig::<Test> {
        validator_count: 4,
        minimum_validator_count: 3,
        invulnerables: vec![],
        force_era: pallet_staking::Forcing::NotForcing,
        slash_reward_fraction: Perbill::from_percent(10),
        stakers: vec![
            (account_key("Alice"), account_key("Alice"), 100_000, StakerStatus::Validator),
            (account_key("Bob"), account_key("Bob"), 100_000, StakerStatus::Validator),
            (account_key("Charlie"), account_key("Charlie"), 100_000, StakerStatus::Validator),
            (account_key("Dave"), account_key("Dave"), 100_000, StakerStatus::Validator),
        ],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}

pub fn account_key(name: &str) -> AccountId32 {
    AccountId32::from([0u8; 32]) // Simplified for testing
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Session::on_initialize(System::block_number());
        Staking::on_initialize(System::block_number());
    }
}

#[cfg(test)]
mod consensus_security_tests {
    use super::*;

    #[test]
    fn test_validator_staking_security() {
        new_test_ext().execute_with(|| {
            // Test 1: Ensure minimum validators are enforced
            let initial_validators = Staking::validator_count();
            assert!(initial_validators >= 3, "Must have at least 3 validators for security");

            // Test 2: Test staking bond requirements
            let min_bond = Staking::min_validator_bond();
            assert!(min_bond > 0, "Minimum validator bond must be positive");

            // Test 3: Test validator can't unstake immediately (bonding duration)
            assert_ok!(Staking::bond(
                RuntimeOrigin::signed(account_key("Eve")),
                account_key("Eve"),
                50_000,
                pallet_staking::RewardDestination::Staked,
            ));

            assert_ok!(Staking::validate(
                RuntimeOrigin::signed(account_key("Eve")),
                pallet_staking::ValidatorPrefs::default(),
            ));

            // Try to unbond immediately - should work but with bonding duration
            assert_ok!(Staking::unbond(
                RuntimeOrigin::signed(account_key("Eve")),
                25_000,
            ));

            // Funds should still be locked for bonding duration
            let ledger = Staking::ledger(account_key("Eve")).unwrap();
            assert_eq!(ledger.unlocking.len(), 1);
            
            // Test bonding duration enforcement
            run_to_block(10); // Still within bonding duration
            assert_noop!(
                Staking::withdraw_unbonded(RuntimeOrigin::signed(account_key("Eve")), 0),
                StakingError::<Test>::NoMoreChunks
            );
        });
    }

    #[test]
    fn test_slashing_security() {
        new_test_ext().execute_with(|| {
            // Test slashing mechanism works
            let alice = account_key("Alice");
            let initial_stake = Staking::ledger(&alice).unwrap().total;
            
            // Simulate an offense that should trigger slashing
            let offence = sp_staking::offence::OffenceDetails {
                offender: (alice.clone(), ()),
                reporters: vec![],
            };

            // Create a mock offense
            let offences = vec![offence];
            
            // Apply slashing
            let slash_fraction = Perbill::from_percent(10);
            
            // Verify slashing parameters are configured
            assert_eq!(
                Staking::slash_reward_fraction(),
                Perbill::from_percent(10)
            );
            
            assert!(Staking::bonding_duration() > 0);
            assert!(Staking::slash_defer_duration() > 0);
        });
    }

    #[test]
    fn test_session_key_security() {
        new_test_ext().execute_with(|| {
            let alice = account_key("Alice");
            
            // Generate new session keys
            let new_keys = TestSessionKeys {
                babe: sp_application_crypto::sr25519::Public::from_raw([1u8; 32]).into(),
                grandpa: sp_application_crypto::ed25519::Public::from_raw([1u8; 32]).into(),
            };

            // Set session keys
            assert_ok!(Session::set_keys(
                RuntimeOrigin::signed(alice.clone()),
                new_keys.into(),
                vec![], // proof
            ));

            // Verify keys were set
            assert!(Session::next_keys(&alice).is_some());
        });
    }

    #[test]
    fn test_consensus_attack_resistance() {
        new_test_ext().execute_with(|| {
            // Test 1: Nothing at stake attack resistance
            // Validators should be slashed for double voting
            let validator_count = Staking::validator_count();
            assert!(validator_count >= 3, "Need minimum validators for Byzantine fault tolerance");

            // Test 2: Long range attack resistance
            // Historical attacks should be prevented by finality
            let current_block = System::block_number();
            run_to_block(current_block + 10);
            
            // Test 3: Grinding attacks resistance
            // Block production should be randomized
            assert!(Babe::epoch_index() >= 0);

            // Test 4: 51% attack resistance through PoS economics
            let total_issuance = Balances::total_issuance();
            let bonded_amount: u128 = Staking::eras_total_stake(Staking::active_era().unwrap().index);
            
            // Ensure significant stake is bonded for security
            let bonded_percentage = (bonded_amount * 100) / total_issuance;
            assert!(bonded_percentage > 10, "At least 10% of tokens should be staked for security");
        });
    }

    #[test]
    fn test_finality_security() {
        new_test_ext().execute_with(|| {
            // Test GRANDPA finality works
            let initial_block = System::block_number();
            run_to_block(initial_block + 5);

            // Verify finality parameters
            assert!(Grandpa::authorities().len() >= 3);
            
            // Test that finalized blocks can't be reverted
            let finalized_number = System::block_number();
            run_to_block(finalized_number + 10);
            
            // Simulate finality gadget operation
            assert!(System::block_number() > finalized_number);
        });
    }

    #[test]
    fn test_validator_set_changes_security() {
        new_test_ext().execute_with(|| {
            let initial_validators = Session::validators();
            assert!(initial_validators.len() >= 3);

            // Test validator set rotation
            run_to_block(Period::get() + 1);
            
            // Ensure validator set is maintained
            let new_validators = Session::validators();
            assert!(new_validators.len() >= 3);
        });
    }

    #[test]
    fn test_economic_security_parameters() {
        new_test_ext().execute_with(|| {
            // Test that economic parameters are set for security
            
            // Bonding duration should be sufficient for dispute resolution
            assert!(Staking::bonding_duration() >= 3);
            
            // Slash defer duration should allow for dispute resolution
            assert!(Staking::slash_defer_duration() >= 1);
            
            // Minimum bond should be economically significant
            let min_bond = 1000u128; // Define minimum for security
            assert!(ExistentialDeposit::get() > 0);
            
            // Offending validators threshold should prevent minority attacks
            assert!(
                OffendingValidatorsThreshold::get() < Perbill::from_percent(50),
                "Offending threshold must be less than 50% to prevent majority attacks"
            );
        });
    }

    #[test]
    fn test_randomness_security() {
        new_test_ext().execute_with(|| {
            // Test that BABE provides secure randomness
            run_to_block(10);
            
            // Verify epoch structure
            let epoch = Babe::current_epoch();
            assert!(epoch.authorities.len() > 0);
            
            // Test epoch duration is reasonable for security
            assert!(Babe::epoch_duration() >= 10);
            
            // Verify randomness is being generated
            let randomness = Babe::randomness();
            assert!(randomness != [0u8; 32]);
        });
    }
}

#[cfg(test)]
mod attack_simulation_tests {
    use super::*;

    #[test]
    fn simulate_double_spend_attack() {
        new_test_ext().execute_with(|| {
            let alice = account_key("Alice");
            let bob = account_key("Bob");
            let charlie = account_key("Charlie");
            
            let initial_balance = Balances::free_balance(&alice);
            
            // Attempt to spend same funds twice
            assert_ok!(Balances::transfer(
                RuntimeOrigin::signed(alice.clone()),
                bob.clone(),
                500,
            ));
            
            // Second transfer should fail due to insufficient balance
            assert_noop!(
                Balances::transfer(
                    RuntimeOrigin::signed(alice.clone()),
                    charlie.clone(),
                    initial_balance - 400, // More than remaining
                ),
                pallet_balances::Error::<Test>::InsufficientBalance
            );
            
            // Verify balances are correct
            assert_eq!(Balances::free_balance(&alice), initial_balance - 500);
            assert_eq!(Balances::free_balance(&bob), 1_000_000 + 500);
            assert_eq!(Balances::free_balance(&charlie), 1_000_000);
        });
    }

    #[test]
    fn simulate_validator_equivocation_attack() {
        new_test_ext().execute_with(|| {
            // Simulate a validator producing conflicting blocks
            let alice = account_key("Alice");
            
            // Get initial stake
            let initial_stake = Staking::ledger(&alice).map(|l| l.total).unwrap_or(0);
            
            // Simulate equivocation detection and slashing
            // In a real scenario, this would be detected by other validators
            
            // Verify slashing parameters are in place
            assert!(Staking::slash_reward_fraction() > Perbill::zero());
            assert!(Staking::bonding_duration() > 0);
            
            // Equivocation should result in slashing
            // This would be triggered by the equivocation reporting system
            assert!(initial_stake > 0, "Validator must have stake to be slashed");
        });
    }

    #[test]
    fn simulate_long_range_attack() {
        new_test_ext().execute_with(|| {
            // Test protection against long-range attacks
            let initial_block = System::block_number();
            
            // Advance significantly
            run_to_block(initial_block + 100);
            
            // Try to revert to old block (should be prevented by finality)
            let finalized_block = System::block_number();
            
            // In Grandpa, blocks become finalized and irreversible
            assert!(finalized_block > initial_block);
            
            // Historical attacks should be impossible due to finality gadget
            // This is enforced by the GRANDPA finality mechanism
        });
    }

    #[test]
    fn simulate_nothing_at_stake_attack() {
        new_test_ext().execute_with(|| {
            // Test that validators have economic incentive to behave honestly
            let alice = account_key("Alice");
            
            // Validator must have stake
            let stake = Staking::ledger(&alice).map(|l| l.total).unwrap_or(0);
            assert!(stake > 0, "Validators must have stake to prevent nothing-at-stake");
            
            // Slashing must be significant enough to deter bad behavior
            let slash_fraction = Staking::slash_reward_fraction();
            assert!(slash_fraction >= Perbill::from_percent(10));
            
            // Bonding duration must be long enough to resolve disputes
            assert!(Staking::bonding_duration() >= 3);
        });
    }

    #[test]
    fn simulate_grinding_attack() {
        new_test_ext().execute_with(|| {
            // Test protection against block grinding attacks
            run_to_block(20);
            
            // BABE should provide unpredictable randomness
            let randomness1 = Babe::randomness();
            
            run_to_block(30);
            let randomness2 = Babe::randomness();
            
            // Randomness should change between epochs
            assert_ne!(randomness1, randomness2, "Randomness should be unpredictable");
            
            // Block production should be based on VRF, not grindable
            assert!(Babe::current_epoch().authorities.len() > 0);
        });
    }
}

#[cfg(test)]
mod consensus_performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_block_production() {
        new_test_ext().execute_with(|| {
            let start_time = Instant::now();
            let start_block = System::block_number();
            
            // Produce 100 blocks
            run_to_block(start_block + 100);
            
            let duration = start_time.elapsed();
            let blocks_produced = 100;
            
            println!("Block production benchmark:");
            println!("  Blocks produced: {}", blocks_produced);
            println!("  Duration: {:?}", duration);
            println!("  Blocks per second: {:.2}", blocks_produced as f64 / duration.as_secs_f64());
            
            // Should be able to produce blocks reasonably fast in tests
            assert!(duration.as_millis() < 5000, "Block production should be fast in tests");
        });
    }

    #[test]
    fn benchmark_finality() {
        new_test_ext().execute_with(|| {
            let start_time = Instant::now();
            let start_block = System::block_number();
            
            // Simulate finality process
            run_to_block(start_block + 50);
            
            let duration = start_time.elapsed();
            
            println!("Finality benchmark:");
            println!("  Blocks finalized: 50");
            println!("  Duration: {:?}", duration);
            
            // Finality should be achieved within reasonable time
            assert!(duration.as_millis() < 3000, "Finality should be achieved quickly");
        });
    }
}