//! # Netchain Storage Contract
//! 
//! A simple yet powerful key-value storage contract demonstrating Ink!'s advantages over Solidity:
//! - Memory safety guaranteed by Rust
//! - No integer overflow vulnerabilities 
//! - Ultra-low gas costs on Netchain
//! - Type safety at compile time
//! - No reentrancy attacks possible

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod netchain_storage {
    use ink::storage::Mapping;
    use ink::prelude::{string::String, vec::Vec};

    /// The main storage contract state
    #[ink(storage)]
    pub struct NetchainStorage {
        /// Mapping from keys to values - secure and efficient
        storage: Mapping<String, String>,
        /// Contract owner for administrative functions
        owner: Option<AccountId>,
        /// Total number of entries for analytics
        total_entries: u32,
        /// Maximum storage limit per user (prevents spam)
        max_entries_per_user: u32,
        /// Per-user entry count tracking
        user_entries: Mapping<AccountId, u32>,
    }

    /// Events emitted by the contract
    #[ink(event)]
    pub struct ValueSet {
        #[ink(topic)]
        key: String,
        #[ink(topic)]
        caller: AccountId,
        value: String,
    }

    #[ink(event)]
    pub struct ValueRemoved {
        #[ink(topic)]
        key: String,
        #[ink(topic)]
        caller: AccountId,
    }

    /// Contract errors - type-safe error handling
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum ContractError {
        /// Key not found in storage
        KeyNotFound,
        /// Only owner can perform this action
        OnlyOwner,
        /// Key is too long (>128 characters)
        KeyTooLong,
        /// Value is too long (>1024 characters) 
        ValueTooLong,
        /// User has reached maximum entries limit
        UserLimitReached,
    }

    /// Result type for contract operations
    pub type Result<T> = core::result::Result<T, ContractError>;

    impl NetchainStorage {
        /// Constructor: Initialize the storage contract
        #[ink(constructor)]
        pub fn new(max_entries_per_user: u32) -> Self {
            Self {
                storage: Mapping::default(),
                owner: Some(Self::env().caller()),
                total_entries: 0,
                max_entries_per_user,
                user_entries: Mapping::default(),
            }
        }

        /// Default constructor with reasonable limits
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(100) // Allow 100 entries per user by default
        }

        /// Store a key-value pair
        #[ink(message)]
        pub fn set(&mut self, key: String, value: String) -> Result<()> {
            // Input validation - prevents vulnerabilities
            if key.len() > 128 {
                return Err(ContractError::KeyTooLong);
            }
            if value.len() > 1024 {
                return Err(ContractError::ValueTooLong);
            }

            let caller = self.env().caller();
            
            // Check if this is a new key
            let is_new_key = !self.storage.contains(&key);
            
            if is_new_key {
                // Check user limits for new keys
                let user_count = self.user_entries.get(caller).unwrap_or(0);
                if user_count >= self.max_entries_per_user {
                    return Err(ContractError::UserLimitReached);
                }
                
                // Update counters for new entries
                self.user_entries.insert(caller, &user_count.saturating_add(1));
                self.total_entries = self.total_entries.saturating_add(1);
            }

            // Store the value - memory safe operation
            self.storage.insert(&key, &value);

            // Emit event for off-chain indexing
            self.env().emit_event(ValueSet {
                key: key.clone(),
                caller,
                value: value.clone(),
            });

            Ok(())
        }

        /// Retrieve a value by key
        #[ink(message)]
        pub fn get(&self, key: String) -> Result<String> {
            self.storage
                .get(&key)
                .ok_or(ContractError::KeyNotFound)
        }

        /// Check if a key exists in storage
        #[ink(message)]
        pub fn contains_key(&self, key: String) -> bool {
            self.storage.contains(&key)
        }

        /// Get the total number of stored entries
        #[ink(message)]
        pub fn total_entries(&self) -> u32 {
            self.total_entries
        }

        /// Get the number of entries for a specific user
        #[ink(message)]
        pub fn user_entry_count(&self, user: AccountId) -> u32 {
            self.user_entries.get(user).unwrap_or(0)
        }

        /// Get the maximum entries allowed per user
        #[ink(message)]
        pub fn max_entries_per_user(&self) -> u32 {
            self.max_entries_per_user
        }

        /// Get the contract owner
        #[ink(message)]
        pub fn owner(&self) -> Option<AccountId> {
            self.owner
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn constructor_works() {
            let contract = NetchainStorage::new(50);
            assert_eq!(contract.max_entries_per_user(), 50);
            assert_eq!(contract.total_entries(), 0);
        }

        #[ink::test]
        fn set_and_get_works() {
            let mut contract = NetchainStorage::default();
            
            assert_eq!(
                contract.set("test_key".to_string(), "test_value".to_string()),
                Ok(())
            );
            
            assert_eq!(
                contract.get("test_key".to_string()),
                Ok("test_value".to_string())
            );
        }

        #[ink::test]
        fn get_nonexistent_key_fails() {
            let contract = NetchainStorage::default();
            
            assert_eq!(
                contract.get("nonexistent".to_string()),
                Err(ContractError::KeyNotFound)
            );
        }

        #[ink::test]
        fn user_limit_enforced() {
            let mut contract = NetchainStorage::new(2); // Limit to 2 entries
            
            // First two should succeed
            assert_eq!(contract.set("key1".to_string(), "value1".to_string()), Ok(()));
            assert_eq!(contract.set("key2".to_string(), "value2".to_string()), Ok(()));
            
            // Third should fail due to user limit
            assert_eq!(
                contract.set("key3".to_string(), "value3".to_string()),
                Err(ContractError::UserLimitReached)
            );
        }
    }
}