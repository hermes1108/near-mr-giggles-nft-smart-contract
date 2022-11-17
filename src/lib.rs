use std::collections::HashMap;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near_bindgen, AccountId, Balance, CryptoHash, PanicOnDefault, Promise, PromiseOrValue,
};

use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;
pub use crate::approval::*;
pub use crate::royalty::*;
pub use crate::events::*;
pub use crate::util::*;

mod internal;
mod approval; 
mod enumeration; 
mod metadata; 
mod mint; 
mod nft_core; 
mod royalty; 
mod events;
mod util;

/// This spec can be treated like a version of the standard.
pub const NFT_METADATA_SPEC: &str = "nft-1.0.0";
/// This is the name of the NFT standard we're using
pub const NFT_STANDARD_NAME: &str = "nep171";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    //contract owner
    pub owner_id: AccountId,

    //keeps track of all the token IDs for a given account
    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,

    //keeps track of the token struct for a given token ID
    pub tokens_by_id: LookupMap<TokenId, Token>,

    //keeps track of the token metadata for a given token ID
    pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,

    //keeps track of the metadata for the contract
    pub metadata: LazyOption<NFTContractMetadata>,

    //keeps track of presale status for a given account
    pub whitelist: LookupMap<AccountId, bool>,
    pub oglist: LookupMap<AccountId, bool>,

    pub presale_minted: LookupMap<AccountId, u128>,
    pub pubsale_minted: LookupMap<AccountId, u128>,

    pub token_ids: Vec<u16>,
}

/// Helper structure for keys of the persistent collections.
#[derive(BorshSerialize)]
pub enum StorageKey {
    TokensPerOwner,
    TokenPerOwnerInner { account_id_hash: CryptoHash },
    TokensById,
    TokenMetadataById,
    NFTContractMetadata,
    TokensPerType,
    TokensPerTypeInner { token_type_hash: CryptoHash },
    TokenTypesLocked,
    Whitelist,
    Oglist,
    PresaleMinted,
    PubsaleMinted,
}

#[near_bindgen]
impl Contract {
    /*
        initialization function (can only be called once).
        this initializes the contract with default metadata so the
        user doesn't have to manually type metadata.
    */
    #[init]
    pub fn new_default_meta(owner_id: AccountId) -> Self {
        //calls the other function "new: with some default metadata and the owner_id passed in 
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: "nft-1.0.0".to_string(),
                name: "Mr Giggles".to_string(),
                symbol: "MrG".to_string(),
                icon: Some(String::from("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='32' height='32'%3E%3Cpath style='fill:%23fafafa; stroke:none;' d='M0 0L1 1L0 0z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M1 0L0 2L1 0z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M2 0L3 1L2 0z'/%3E%3Cpath style='fill:%23030303; stroke:none;' d='M12 21L5 26L4 7C12.258 11.4114 18.6243 29.9562 27.8951 30.963C31.184 31.3201 31.8043 27.3875 31.956 24.9961C32.3449 18.8674 33.2991 -9.06173 23.0193 5.99074C21.9103 7.61468 20.9548 9.28513 20 11L27 6L28 25C19.742 20.5886 13.3757 2.04377 4.10494 1.03704C0.816038 0.67989 0.195724 4.61246 0.0439815 7.00386C-0.343145 13.1048 -1.58328 40.5888 8.94136 26.0193C10.0992 24.4164 11.0403 22.7217 12 21z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M5 0L6 1L5 0z'/%3E%3Cpath style='fill:%23fafafa; stroke:none;' d='M6 0L27 24L27 8L19 13L26 0L6 0z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M26 0L27 1L26 0z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M27 0L28 1L27 0M29 0L30 1L29 0z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M30 0L32 2L30 0z'/%3E%3Cpath style='fill:%23fafafa; stroke:none;' d='M31 0L32 1L31 0z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M6 1L7 2L6 1z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M25 1L26 2L25 1z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M0 2L1 3L0 2z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M7 2L8 3L7 2z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M25 2L26 3L25 2M31 2L32 3L31 2z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M8 3L9 4L8 3M24 3L25 4L24 3z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M9 4L10 5L9 4M23 4L24 5L23 4M10 5L12 7L10 5z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M23 5L24 6L23 5M10 6L11 7L10 6z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M22 6L23 7L22 6z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M26 6L27 25L28 25L26 6M4 7L5 26L6 26L4 7z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M5 7L8 10L5 7z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M6 7L7 8L6 7M11 7L12 8L11 7z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M21 7L22 8L21 7z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M25 7L26 8L25 7z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M26 7L27 8L26 7z'/%3E%3Cpath style='fill:%23fafafa; stroke:none;' d='M5 8L5 24L13 19L6 32L26 32L5 8z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M12 8L13 9L12 8z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M21 8L22 9L21 8z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M24 8L25 9L24 8M13 9L14 10L13 9M20 9L21 10L20 9z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M23 9L24 10L23 9z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M8 10L9 11L8 10z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M14 10L15 11L14 10M19 10L20 11L19 10z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M21 10L19 11L19 12L21 10z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M22 10L23 11L22 10z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M9 11L10 12L9 11z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M15 11L17 13L15 11M21 11L22 12L21 11z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M10 12L12 14L10 12M15 12L16 13L15 12z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M19 12L20 13L19 12z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M20 12L21 13L20 12M10 13L11 14L10 13z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M16 13L17 14L16 13z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M11 14L12 15L11 14z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M17 14L18 15L17 14z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M12 15L13 16L12 15z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M18 15L19 16L18 15M13 16L14 17L13 16z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M19 16L20 17L19 16z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M14 17L15 18L14 17z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M20 17L22 19L20 17z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M15 18L17 20L15 18M20 18L21 19L20 18z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M11 19L10 21L11 19z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M12 19L13 20L12 19z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M15 19L16 20L15 19z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M21 19L22 20L21 19M11.6667 20.3333L12.3333 20.6667L11.6667 20.3333z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M16 20L17 21L16 20z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M22 20L23 21L22 20z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M9 21L10 22L9 21M12 21L13 22L12 21M17 21L18 22L17 21z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M23 21L24 22L23 21z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M8 22L9 23L8 22z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M11 22L12 23L11 22M18 22L19 23L18 22z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M24 22L25 23L24 22z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M7 23L8 24L7 23z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M10 23L11 24L10 23z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M19 23L20 24L19 23z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M25 23L27 25L25 23M5 24L6 25L5 24z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M6 24L7 25L6 24z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M10 24L11 25L10 24z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M20 24L22 26L20 24M25 24L26 25L25 24z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M9 25L10 26L9 25z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M20 25L21 26L20 25z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M8 26L9 27L8 26z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M21 26L22 27L21 26M8 27L9 28L8 27M22 27L23 28L22 27z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M7 28L8 29L7 28M23 28L24 29L23 28z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M0 29L1 30L0 29M6 29L4 32L6 29z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M24 29L25 30L24 29z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M31 29L32 30L31 29z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M0 30L2 32L0 30M6 30L7 31L6 30z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M25 30L26 31L25 30z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M31 30L30 32L31 30z'/%3E%3Cpath style='fill:%23fafafa; stroke:none;' d='M0 31L1 32L0 31z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M2 31L3 32L2 31z'/%3E%3Cpath style='fill:%239e9e9e; stroke:none;' d='M5 31L6 32L5 31z'/%3E%3Cpath style='fill:%23626262; stroke:none;' d='M26 31L27 32L26 31z'/%3E%3Cpath style='fill:%23292929; stroke:none;' d='M29 31L30 32L29 31z'/%3E%3Cpath style='fill:%23fafafa; stroke:none;' d='M31 31L32 32L31 31z'/%3E%3C/svg%3E")),
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }

    /*
        initialization function (can only be called once).
        this initializes the contract with metadata that was passed in and
        the owner_id. 
    */
    #[init]
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
        //create a variable of type Self with all the fields initialized. 
        let mut ids:Vec<u16> = Vec::new();
        for i in 1..667 {
            ids.push(i);
        }
        let this = Self {
            //Storage keys are simply the prefixes used for the collections. This helps avoid data collision
            tokens_per_owner: LookupMap::new(StorageKey::TokensPerOwner.try_to_vec().unwrap()),
            tokens_by_id: LookupMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
            token_metadata_by_id: UnorderedMap::new(
                StorageKey::TokenMetadataById.try_to_vec().unwrap(),
            ),
            //set the owner_id field equal to the passed in owner_id. 
            owner_id,
            metadata: LazyOption::new(
                StorageKey::NFTContractMetadata.try_to_vec().unwrap(),
                Some(&metadata),
            ),
            whitelist: LookupMap::new(StorageKey::Whitelist.try_to_vec().unwrap()),
            oglist: LookupMap::new(StorageKey::Oglist.try_to_vec().unwrap()),
            presale_minted: LookupMap::new(StorageKey::PresaleMinted.try_to_vec().unwrap()),
            pubsale_minted: LookupMap::new(StorageKey::PubsaleMinted.try_to_vec().unwrap()),
            token_ids: ids,
        };

        //return the Contract object
        this
    }
}