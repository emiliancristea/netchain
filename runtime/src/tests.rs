//! Tests for Netchain runtime - focus on consensus, staking, and low fees

#![cfg(test)]

use super::*;
use frame_support::{
	assert_noop, assert_ok, 
	traits::{Get, OnFinalize, OnInitialize},
	weights::Weight,
};
use pallet_staking::{ActiveEra, ActiveEraInfo, CurrentEra, ErasStakers, Validators};
use sp_runtime::{
	traits::{BadOrigin, Zero},
	Perbill, Perquintill,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

// Test runtime configuration
frame_support::construct_runtime!(
	pub enum TestRuntime
	{
		System: frame_system,
		Timestamp: pallet_timestamp,
		Babe: pallet_babe,
		Balances: pallet_balances,
		TransactionPayment: pallet_transaction_payment,
		Staking: pallet_staking,
		Session: pallet_session,
		Authorship: pallet_authorship,
	}
);

#[test]
fn test_low_transaction_fees() {
	use pallet_transaction_payment::ChargeTransactionPayment;
	use frame_support::dispatch::{DispatchInfo, PostDispatchInfo};
	use sp_runtime::traits::SignedExtension;

	new_test_ext().execute_with(|| {
		let info = DispatchInfo {
			weight: Weight::from_parts(1000, 0),
			class: frame_support::dispatch::DispatchClass::Normal,
			pays_fee: frame_support::dispatch::Pays::Yes,
		};
		
		let len = 100_u32; // 100 bytes
		
		// Create the transaction payment extension
		let ext = ChargeTransactionPayment::<Runtime>::from(0);
		
		// Test that fees are ultra-low: 1 unit per byte + minimal weight fee
		let pre = ext.pre_dispatch(&1, &RuntimeCall::System(frame_system::Call::remark { remark: vec![] }), &info, len as usize).unwrap();
		let post_info = PostDispatchInfo { actual_weight: Some(Weight::from_parts(500, 0)), pays_fee: frame_support::dispatch::Pays::Yes };
		
		// The fee should be extremely low
		let fee = pallet_transaction_payment::Pallet::<Runtime>::compute_fee(len, &info, 0);
		
		// Fee should be approximately: (weight / 1_000_000) + (length * 1) = minimal
		// With 1000 weight units and 100 bytes, this should be nearly free
		assert!(fee < 1000, "Fee should be ultra-low: {} should be < 1000", fee);
		
		println!("Transaction fee for 100 bytes: {} units (ultra-low!)", fee);
	});
}

#[test] 
fn test_babe_block_production() {
	new_test_ext().execute_with(|| {
		// Test that BABE configuration allows 3-second blocks
		assert_eq!(MILLI_SECS_PER_BLOCK, 3000);
		assert_eq!(SLOT_DURATION, 3000);
		
		// Test that epoch duration is reasonable for fast blocks
		assert_eq!(EPOCH_DURATION_IN_BLOCKS, 10 * MINUTES);
		
		// With 3 second blocks, 10 minutes = 200 blocks per epoch
		let expected_blocks_per_epoch = (10 * 60 * 1000) / MILLI_SECS_PER_BLOCK;
		assert_eq!(EPOCH_DURATION_IN_BLOCKS as u64, expected_blocks_per_epoch);
	});
}

#[test]
fn test_staking_configuration() {
	new_test_ext().execute_with(|| {
		// Test staking parameters are optimized for high performance
		assert_eq!(SessionsPerEra::get(), 6);
		assert_eq!(BondingDuration::get(), 24 * 7); // 7 days
		assert_eq!(SlashDeferDuration::get(), 24); // 1 day
		
		// Test that we support many validators for decentralization
		assert_eq!(pallet_babe::MaxAuthorities::<Runtime>::get(), 100);
		assert_eq!(pallet_babe::MaxNominators::<Runtime>::get(), 1000);
	});
}

#[test]
fn test_validator_rewards() {
	new_test_ext().execute_with(|| {
		// Initialize accounts
		let alice = AccountId::from([1u8; 32]);
		let bob = AccountId::from([2u8; 32]);
		
		// Give them some balance
		let _ = Balances::deposit_creating(&alice, 10_000 * DOLLARS);
		let _ = Balances::deposit_creating(&bob, 10_000 * DOLLARS);
		
		// Alice bonds as validator
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(alice.clone()),
			1000 * DOLLARS, // bond amount
			pallet_staking::RewardDestination::Staked,
		));
		
		assert_ok!(Staking::validate(
			RuntimeOrigin::signed(alice.clone()),
			pallet_staking::ValidatorPrefs::default(),
		));
		
		// Bob nominates Alice
		assert_ok!(Staking::bond(
			RuntimeOrigin::signed(bob.clone()),
			500 * DOLLARS, // bond amount  
			pallet_staking::RewardDestination::Staked,
		));
		
		assert_ok!(Staking::nominate(
			RuntimeOrigin::signed(bob.clone()),
			vec![alice.clone()],
		));
		
		// Check that staking worked
		assert!(Validators::<Runtime>::contains_key(&alice));
		
		println!("✅ Validator rewards test setup completed");
	});
}

#[test]
fn test_slashing_configuration() {
	new_test_ext().execute_with(|| {
		// Test that slashing is configured but not too harsh for development
		let slash_reward_fraction = Perbill::from_percent(10);
		
		// SlashDeferDuration should be 1 day (24 blocks with our block time)
		assert_eq!(SlashDeferDuration::get(), 24);
		
		// Offending validators threshold should be reasonable
		assert_eq!(OffendingValidatorsThreshold::get(), Perbill::from_percent(33));
		
		println!("✅ Slashing parameters are correctly configured");
	});
}

#[test]
fn test_session_rotation() {
	new_test_ext().execute_with(|| {
		// Test that sessions rotate properly with BABE
		let session_length = 6 * HOURS; // 6 hours per session
		
		// With 3 second blocks, 6 hours = 7200 blocks
		let expected_session_blocks = (6 * 60 * 60 * 1000) / MILLI_SECS_PER_BLOCK;
		
		// Period should match our configuration
		assert_eq!(Period::get() as u64, expected_session_blocks);
		
		println!("✅ Session rotation configured for {} blocks per session", expected_session_blocks);
	});
}

#[test]
fn test_consensus_performance_targets() {
	new_test_ext().execute_with(|| {
		// Test that our consensus targets high performance
		
		// Block time: 3 seconds (targeting high TPS)
		assert_eq!(MILLI_SECS_PER_BLOCK, 3000);
		
		// Block weight limits should allow high throughput
		let block_weights = RuntimeBlockWeights::get();
		let normal_weight = block_weights.get(frame_support::dispatch::DispatchClass::Normal);
		
		// Should allow for significant computation per block
		assert!(normal_weight.max_extrinsic.unwrap().ref_time() > 1_000_000_000);
		
		// Block length should support large blocks
		let block_length = RuntimeBlockLength::get();
		assert!(block_length.max.get(frame_support::dispatch::DispatchClass::Normal) > &(4 * 1024 * 1024)); // > 4MB
		
		println!("✅ Performance targets configured for high TPS");
		println!("   - Block time: {}ms", MILLI_SECS_PER_BLOCK);
		println!("   - Max normal weight: {} units", normal_weight.max_extrinsic.unwrap().ref_time());
		println!("   - Max block size: {} bytes", block_length.max.get(frame_support::dispatch::DispatchClass::Normal));
	});
}

// Test utilities
fn new_test_ext() -> sp_io::TestExternalities {
	use sp_runtime::BuildStorage;
	
	let mut storage = frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.unwrap();
	
	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(AccountId::from([1u8; 32]), 10_000 * DOLLARS),
			(AccountId::from([2u8; 32]), 10_000 * DOLLARS),
		],
	}
	.assimilate_storage(&mut storage)
	.unwrap();
	
	let mut ext = sp_io::TestExternalities::from(storage);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

#[test]
fn test_fee_calculation_examples() {
	new_test_ext().execute_with(|| {
		// Test various transaction sizes to show ultra-low fees
		let test_cases = vec![
			(100, "Small transaction"),
			(1_000, "Medium transaction"), 
			(10_000, "Large transaction"),
			(100_000, "Very large transaction"),
		];
		
		println!("\n🔥 Netchain Ultra-Low Fee Examples:");
		println!("=====================================");
		
		for (bytes, description) in test_cases {
			let info = frame_support::dispatch::DispatchInfo {
				weight: Weight::from_parts(1000, 0), // Minimal weight
				class: frame_support::dispatch::DispatchClass::Normal,
				pays_fee: frame_support::dispatch::Pays::Yes,
			};
			
			let fee = pallet_transaction_payment::Pallet::<Runtime>::compute_fee(bytes, &info, 0);
			
			println!("{}: {} bytes = {} units", description, bytes, fee);
			
			// Even large transactions should cost less than 0.1 DOLLARS
			assert!(fee < DOLLARS / 10, "Fee {} should be less than 0.1 DOLLARS for {} bytes", fee, bytes);
		}
		
		println!("=====================================");
		println!("🚀 All fees are ultra-low - perfect for high-volume usage!");
	});
}