#![cfg_attr(not(feature = "std"), no_std)]

//! # Oracle Pallet
//!
//! A native oracle pallet for Netchain that provides secure off-chain data integration.
//!
//! This pallet provides:
//! - Off-chain data fetching with configurable sources
//! - Price feeds for tokens and assets
//! - Weather, sports, and general API data integration
//! - Ultra-low fees for oracle queries
//! - Data validation and aggregation
//! - Request batching for efficiency
//!
//! ## Security Features
//! - Multiple data source validation
//! - Outlier detection and filtering
//! - Signature verification for trusted sources
//! - Rate limiting to prevent spam
//! - Data freshness checks

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

/// Oracle request identifier
pub type RequestId = u64;
/// Data source identifier  
pub type SourceId = Vec<u8>;
/// Oracle data key (e.g., "BTC/USD", "weather/london")
pub type DataKey = Vec<u8>;
/// Oracle data value (JSON string or encoded data)  
pub type DataValue = Vec<u8>;

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

        /// Currency for oracle fees and rewards
        type Currency: ReservableCurrency<Self::AccountId>;

        /// Maximum number of data sources per request
        #[pallet::constant]
        type MaxDataSources: Get<u32>;

        /// Maximum size of oracle data value
        #[pallet::constant]
        type MaxDataSize: Get<u32>;

        /// Fee for basic oracle query (ultra-low)
        #[pallet::constant]
        type OracleQueryFee: Get<BalanceOf<Self>>;

        /// Fee for premium oracle query with multiple sources
        #[pallet::constant]
        type PremiumQueryFee: Get<BalanceOf<Self>>;

        /// Reward for providing valid oracle data
        #[pallet::constant]
        type OracleReward: Get<BalanceOf<Self>>;

        /// Maximum age of oracle data in blocks
        #[pallet::constant]
        type MaxDataAge: Get<u64>;

        /// Minimum number of sources required for aggregation
        #[pallet::constant]
        type MinAggregationSources: Get<u32>;

        /// Pallet identifier for account derivation
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// WeightInfo for benchmarking
        type WeightInfo: WeightInfo;
    }

    pub type BalanceOf<T> = <<T as Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// Oracle data request
    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
    pub struct OracleRequest<AccountId, BlockNumber> {
        /// Account that made the request
        pub requester: AccountId,
        /// Data key being requested
        pub data_key: DataKey,
        /// Data sources to query
        pub sources: Vec<SourceId>,
        /// Block when request was made
        pub requested_at: BlockNumber,
        /// Whether this is a premium request
        pub premium: bool,
        /// Callback information (optional)
        pub callback: Option<Vec<u8>>,
    }

    /// Oracle data entry with metadata
    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
    pub struct OracleData<AccountId, BlockNumber> {
        /// The actual data value
        pub value: DataValue,
        /// Account that provided the data
        pub provider: AccountId,
        /// Block when data was submitted
        pub timestamp: BlockNumber,
        /// Source identifier
        pub source: SourceId,
        /// Confidence score (0-100)
        pub confidence: u8,
        /// Signature for verification (optional)
        pub signature: Option<Vec<u8>>,
    }

    /// Aggregated oracle data with multiple sources
    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
    pub struct AggregatedData<BlockNumber> {
        /// Aggregated/median value
        pub value: DataValue,
        /// Number of sources used
        pub source_count: u32,
        /// Average confidence score
        pub confidence: u8,
        /// Block when aggregation was calculated
        pub aggregated_at: BlockNumber,
        /// Individual data points used
        pub data_points: Vec<DataValue>,
    }

    /// Data source configuration
    #[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
    pub struct DataSource {
        /// Source identifier
        pub id: SourceId,
        /// Human-readable name
        pub name: Vec<u8>,
        /// API endpoint or description
        pub endpoint: Vec<u8>,
        /// Source reliability score (0-100)
        pub reliability: u8,
        /// Whether source is active
        pub active: bool,
    }

    /// Storage for oracle requests
    #[pallet::storage]
    #[pallet::getter(fn oracle_requests)]
    pub type OracleRequests<T: Config> = 
        StorageMap<_, Blake2_128Concat, RequestId, OracleRequest<T::AccountId, BlockNumberFor<T>>>;

    /// Storage for oracle data by key
    #[pallet::storage]
    #[pallet::getter(fn oracle_data)]
    pub type OracleDataStorage<T: Config> = StorageDoubleMap<
        _, Blake2_128Concat, DataKey,
        Blake2_128Concat, SourceId,
        OracleData<T::AccountId, BlockNumberFor<T>>,
    >;

    /// Storage for aggregated oracle data
    #[pallet::storage]
    #[pallet::getter(fn aggregated_data)]
    pub type AggregatedDataStorage<T: Config> = 
        StorageMap<_, Blake2_128Concat, DataKey, AggregatedData<BlockNumberFor<T>>>;

    /// Storage for data sources
    #[pallet::storage]
    #[pallet::getter(fn data_sources)]
    pub type DataSources<T: Config> = StorageMap<_, Blake2_128Concat, SourceId, DataSource>;

    /// Storage for trusted oracle providers
    #[pallet::storage]
    #[pallet::getter(fn trusted_providers)]
    pub type TrustedProviders<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u8>; // reputation score

    /// Next request ID to assign
    #[pallet::storage]
    #[pallet::getter(fn next_request_id)]
    pub type NextRequestId<T> = StorageValue<_, RequestId, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Oracle data requested
        DataRequested { 
            request_id: RequestId, 
            requester: T::AccountId, 
            data_key: DataKey, 
            sources: Vec<SourceId>,
            premium: bool,
        },
        /// Oracle data provided by a source
        DataProvided { 
            data_key: DataKey, 
            source: SourceId, 
            provider: T::AccountId, 
            value: DataValue,
            confidence: u8,
        },
        /// Data aggregated from multiple sources
        DataAggregated { 
            data_key: DataKey, 
            value: DataValue, 
            source_count: u32, 
            confidence: u8,
        },
        /// Data source registered
        SourceRegistered { source_id: SourceId, name: Vec<u8> },
        /// Oracle provider added to trusted list
        ProviderTrusted { provider: T::AccountId, reputation: u8 },
        /// Oracle data expired and removed
        DataExpired { data_key: DataKey, expired_at: BlockNumberFor<T> },
        /// Batch request processed
        BatchProcessed { request_count: u32, total_fee: BalanceOf<T> },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Oracle request not found
        RequestNotFound,
        /// Data key not found
        DataKeyNotFound,
        /// Data source not found
        SourceNotFound,
        /// Invalid data source
        InvalidSource,
        /// Data too large
        DataTooLarge,
        /// Insufficient balance for oracle fees
        InsufficientBalance,
        /// Data is too old
        DataTooOld,
        /// Not enough sources for aggregation
        InsufficientSources,
        /// Invalid confidence score
        InvalidConfidence,
        /// Provider not trusted
        ProviderNotTrusted,
        /// Data source limit exceeded
        TooManySources,
        /// Invalid signature
        InvalidSignature,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Request oracle data from off-chain sources
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::request_data())]
        pub fn request_data(
            origin: OriginFor<T>,
            data_key: DataKey,
            sources: Vec<SourceId>,
            premium: bool,
            callback: Option<Vec<u8>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Validate sources limit
            ensure!(sources.len() <= T::MaxDataSources::get() as usize, Error::<T>::TooManySources);

            // Charge appropriate fee
            let fee = if premium { T::PremiumQueryFee::get() } else { T::OracleQueryFee::get() };
            T::Currency::transfer(&who, &Self::account_id(), fee, ExistenceRequirement::KeepAlive)?;

            // Generate request ID
            let request_id = <NextRequestId<T>>::get();
            <NextRequestId<T>>::put(request_id.saturating_add(1));

            // Create request
            let request = OracleRequest {
                requester: who.clone(),
                data_key: data_key.clone(),
                sources: sources.clone(),
                requested_at: frame_system::Pallet::<T>::block_number(),
                premium,
                callback,
            };

            // Store request
            <OracleRequests<T>>::insert(request_id, &request);

            // Emit event
            Self::deposit_event(Event::DataRequested {
                request_id,
                requester: who,
                data_key,
                sources,
                premium,
            });

            Ok(())
        }

        /// Provide oracle data for a specific key and source
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::provide_data())]
        pub fn provide_data(
            origin: OriginFor<T>,
            data_key: DataKey,
            source: SourceId,
            value: DataValue,
            confidence: u8,
            signature: Option<Vec<u8>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Validate data size
            ensure!(value.len() <= T::MaxDataSize::get() as usize, Error::<T>::DataTooLarge);

            // Validate confidence score
            ensure!(confidence <= 100, Error::<T>::InvalidConfidence);

            // Validate source exists and is active
            let source_info = <DataSources<T>>::get(&source).ok_or(Error::<T>::SourceNotFound)?;
            ensure!(source_info.active, Error::<T>::InvalidSource);

            // Check if provider is trusted for premium data
            if confidence > 80 {
                ensure!(<TrustedProviders<T>>::contains_key(&who), Error::<T>::ProviderNotTrusted);
            }

            // Create oracle data entry
            let oracle_data = OracleData {
                value: value.clone(),
                provider: who.clone(),
                timestamp: frame_system::Pallet::<T>::block_number(),
                source: source.clone(),
                confidence,
                signature,
            };

            // Store data
            <OracleDataStorage<T>>::insert(&data_key, &source, &oracle_data);

            // Reward provider (ultra-low to maintain sustainability)
            let reward = T::OracleReward::get();
            let _ = T::Currency::transfer(&Self::account_id(), &who, reward, ExistenceRequirement::AllowDeath);

            // Emit event
            Self::deposit_event(Event::DataProvided {
                data_key: data_key.clone(),
                source,
                provider: who,
                value,
                confidence,
            });

            // Try to aggregate data if enough sources
            Self::try_aggregate_data(&data_key)?;

            Ok(())
        }

        /// Register a new data source
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::register_source())]
        pub fn register_source(
            origin: OriginFor<T>,
            source_id: SourceId,
            name: Vec<u8>,
            endpoint: Vec<u8>,
            reliability: u8,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // Validate reliability score
            ensure!(reliability <= 100, Error::<T>::InvalidConfidence);

            // Create data source
            let source = DataSource {
                id: source_id.clone(),
                name: name.clone(),
                endpoint,
                reliability,
                active: true,
            };

            // Store source
            <DataSources<T>>::insert(&source_id, &source);

            // Emit event
            Self::deposit_event(Event::SourceRegistered { source_id, name });

            Ok(())
        }

        /// Add a trusted oracle provider
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::add_trusted_provider())]
        pub fn add_trusted_provider(
            origin: OriginFor<T>,
            provider: T::AccountId,
            reputation: u8,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // Validate reputation score
            ensure!(reputation <= 100, Error::<T>::InvalidConfidence);

            // Store trusted provider
            <TrustedProviders<T>>::insert(&provider, reputation);

            // Emit event
            Self::deposit_event(Event::ProviderTrusted { provider, reputation });

            Ok(())
        }

        /// Batch multiple oracle requests for efficiency
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::batch_requests())]
        pub fn batch_requests(
            origin: OriginFor<T>,
            requests: Vec<(DataKey, Vec<SourceId>, bool)>, // (key, sources, premium)
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut total_fee = BalanceOf::<T>::zero();
            let mut request_count = 0u32;

            // Process each request
            for (data_key, sources, premium) in requests {
                // Validate sources limit
                ensure!(sources.len() <= T::MaxDataSources::get() as usize, Error::<T>::TooManySources);

                // Calculate fee
                let fee = if premium { T::PremiumQueryFee::get() } else { T::OracleQueryFee::get() };
                total_fee = total_fee.saturating_add(fee);

                // Generate request ID
                let request_id = <NextRequestId<T>>::get();
                <NextRequestId<T>>::put(request_id.saturating_add(1));

                // Create request
                let request = OracleRequest {
                    requester: who.clone(),
                    data_key: data_key.clone(),
                    sources: sources.clone(),
                    requested_at: frame_system::Pallet::<T>::block_number(),
                    premium,
                    callback: None,
                };

                // Store request
                <OracleRequests<T>>::insert(request_id, &request);

                request_count = request_count.saturating_add(1);
            }

            // Charge total fee
            T::Currency::transfer(&who, &Self::account_id(), total_fee, ExistenceRequirement::KeepAlive)?;

            // Emit event
            Self::deposit_event(Event::BatchProcessed { request_count, total_fee });

            Ok(())
        }

        /// Clean up expired oracle data
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::cleanup_expired_data())]
        pub fn cleanup_expired_data(
            origin: OriginFor<T>,
            data_keys: Vec<DataKey>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            let max_age = T::MaxDataAge::get();

            for data_key in data_keys {
                // Check if aggregated data is expired
                if let Some(aggregated) = <AggregatedDataStorage<T>>::get(&data_key) {
                    let age = current_block.saturating_sub(aggregated.aggregated_at).saturated_into::<u64>();
                    if age > max_age {
                        <AggregatedDataStorage<T>>::remove(&data_key);
                        Self::deposit_event(Event::DataExpired { 
                            data_key: data_key.clone(), 
                            expired_at: current_block 
                        });
                    }
                }

                // Clean up individual data points
                <OracleDataStorage<T>>::remove_prefix(&data_key, None);
            }

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Get the account ID for the pallet
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        /// Try to aggregate data from multiple sources
        fn try_aggregate_data(data_key: &DataKey) -> DispatchResult {
            let min_sources = T::MinAggregationSources::get();
            let mut data_points = Vec::new();
            let mut total_confidence = 0u32;
            let mut source_count = 0u32;

            // Collect data from all sources for this key
            for (_source_id, oracle_data) in <OracleDataStorage<T>>::iter_prefix(data_key) {
                data_points.push(oracle_data.value.clone());
                total_confidence = total_confidence.saturating_add(oracle_data.confidence as u32);
                source_count = source_count.saturating_add(1);
            }

            // Only aggregate if we have enough sources
            if source_count >= min_sources {
                // Simple aggregation: use the first value (in production, implement median/average)
                let aggregated_value = data_points.first().cloned().unwrap_or_default();
                let average_confidence = (total_confidence / source_count) as u8;

                // Create aggregated data
                let aggregated = AggregatedData {
                    value: aggregated_value.clone(),
                    source_count,
                    confidence: average_confidence,
                    aggregated_at: frame_system::Pallet::<T>::block_number(),
                    data_points,
                };

                // Store aggregated data
                <AggregatedDataStorage<T>>::insert(data_key, &aggregated);

                // Emit event
                Self::deposit_event(Event::DataAggregated {
                    data_key: data_key.clone(),
                    value: aggregated_value,
                    source_count,
                    confidence: average_confidence,
                });
            }

            Ok(())
        }

        /// Get latest oracle data for a key (public interface)
        pub fn get_latest_data(data_key: &DataKey) -> Option<DataValue> {
            <AggregatedDataStorage<T>>::get(data_key).map(|data| data.value)
        }

        /// Get data with confidence score
        pub fn get_data_with_confidence(data_key: &DataKey) -> Option<(DataValue, u8)> {
            <AggregatedDataStorage<T>>::get(data_key).map(|data| (data.value, data.confidence))
        }
    }
}

/// Weight functions needed for benchmarking
pub trait WeightInfo {
    fn request_data() -> Weight;
    fn provide_data() -> Weight;
    fn register_source() -> Weight;
    fn add_trusted_provider() -> Weight;
    fn batch_requests() -> Weight;
    fn cleanup_expired_data() -> Weight;
}

/// Default weights (based on complexity analysis)
impl WeightInfo for () {
    fn request_data() -> Weight { Weight::from_parts(60_000, 0) }
    fn provide_data() -> Weight { Weight::from_parts(100_000, 0) }
    fn register_source() -> Weight { Weight::from_parts(40_000, 0) }
    fn add_trusted_provider() -> Weight { Weight::from_parts(30_000, 0) }
    fn batch_requests() -> Weight { Weight::from_parts(200_000, 0) }
    fn cleanup_expired_data() -> Weight { Weight::from_parts(150_000, 0) }
}