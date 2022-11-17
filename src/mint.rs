use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: Option<TokenId>,
        // metadata: TokenMetadata,
        receiver_id: AccountId,
        //we add an optional parameter for perpetual royalties
        _perpetual_royalties: Option<HashMap<AccountId, u32>>,
    ) -> String{
        let total_supply:u16 = self.token_metadata_by_id.len().try_into().unwrap();
        assert!(
            total_supply < 666,
            "Exceeds max nfts"
        );
        let length:u64 = self.token_ids.len().try_into().unwrap();
        let index:usize = (env::block_timestamp() % length).try_into().unwrap();
        let my_token_id = self.token_ids[index];
        self.token_ids.remove(index.try_into().unwrap());
        let token_type = get_type_by_id(my_token_id);
        let caller = env::predecessor_account_id();
        let deposit = env::attached_deposit();
        let curr_time = env::block_timestamp() / 1_000_000;

        const PRESALE_TIME: u64 = 0; // 2nd July 2022 04:00PM UTC
        const PUBSALE_TIME: u64 = 1656788400000; // 2nd July 2022 07:00PM UTC
        const OG_PRICE: u128 = 7_000_000_000_000_000_000_000_000; // 7 $NEAR
        const WL_PRICE: u128 = 8_000_000_000_000_000_000_000_000; // 8 $NEAR
        const PUB_PRICE: u128 = 8_000_000_000_000_000_000_000_000; // 8 $NEAR
        assert!(
            curr_time >= PRESALE_TIME,
            "Presale not started"
        );
        if curr_time < PUBSALE_TIME {
            assert!(self.oglist.contains_key(&caller) || self.whitelist.contains_key(&caller), "You are not whitelisted");
        }
        if self.oglist.contains_key(&caller) {
            assert!(
                deposit >= OG_PRICE,
                "Insufficient fund"
            );
        } else if self.whitelist.contains_key(&caller) {
            assert!(
                deposit >= WL_PRICE,
                "Insufficient fund"
            );
        } else {
            assert!(deposit >= PUB_PRICE, "Insufficient fund")
        }

        let metadata = TokenMetadata {
            title: Some(format!("Mr Giggles #{}", my_token_id)), // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
            description: Some(String::from("Mr Giggles is an art NFT project (on the NEAR Protocol blockchain) with lofty aspirations of becoming a thriving DAO. This project is the brainchild of father and son duo: Mr Giggles and Jamma (Mr Giggles Jr.). Mr Giggles is built on community, inviting holders in to make decisions and benefit from being involved in the project.")), // free-form description
            media: Some(format!("https://ipfs.io/ipfs/QmXXsDoynSQpinPRSqu6RGJwSHkkSFA1k2xX7r8kco557P/{}.gif", my_token_id)), // URL to associated media, preferably to decentralized, content-addressed storage
            media_hash: Some(Base64VecU8(b"VGhpcyBpcyBtZWRpYSBoYXNoLg==".to_vec())), // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
            copies: Some(1), // number of copies of this set of metadata in existence when token was minted.
            issued_at: Some(curr_time), // When token was issued or minted, Unix epoch in milliseconds
            expires_at: None, // When token expires, Unix epoch in milliseconds
            starts_at: None, // When token starts being valid, Unix epoch in milliseconds
            updated_at: None, // When token was last updated, Unix epoch in milliseconds
            extra: Some(format!("{{\"attributes\": [{{\"trait_type\": \"Class\", \"value\": \"{}\" }}]}}", token_type)), // anything extra the NFT wants to store on-chain. Can be stringified JSON.
            reference: Some(format!("https://ipfs.io/ipfs/QmeUL6QkHZKMPdWRwb8kWmQMbgdTbMNpZUegLkpd2Wx4fY/{}.json", my_token_id)), // URL to an off-chain JSON file with more info.
            reference_hash: Some(Base64VecU8(b"QmFzZTY0LWVuY29kZWQgc2hhMjU2IGhhc2ggb2YgSlNPTiBmcm9tIHJlZmVyZW5jZSBmaWVsZC4=".to_vec())), // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
        };

        let mut final_token_id = format!("{}", my_token_id);
        if let Some(token_id) = token_id {
            final_token_id = token_id;
        }
        //measure the initial storage being used on the contract
        // let initial_storage_usage = env::storage_usage();

        // create a royalty map to store in the token
        let mut royalty = HashMap::new();
        let royal: AccountId = "mrgiggles.near".parse().unwrap();
        /*
        // if perpetual royalties were passed into the function: 
        if let Some(perpetual_royalties) = perpetual_royalties {
            //make sure that the length of the perpetual royalties is below 7 since we won't have enough GAS to pay out that many people
            assert!(perpetual_royalties.len() < 7, "Cannot add more than 6 perpetual royalty amounts");

            //iterate through the perpetual royalties and insert the account and amount in the royalty map
            for (account, amount) in perpetual_royalties {
                royalty.insert(account, amount);
            }
        }
        */
        royalty.insert(royal, 700);

        //specify the token struct that contains the owner ID 
        let token = Token {
            //set the owner ID equal to the receiver ID passed into the function
            owner_id: receiver_id,
            //we set the approved account IDs to the default value (an empty map)
            approved_account_ids: Default::default(),
            //the next approval ID is set to 0
            next_approval_id: 0,
            //the map of perpetual royalties for the token (The owner will get 100% - total perpetual royalties)
            royalty,
        };

        //insert the token ID and token struct and make sure that the token doesn't exist
        assert!(
            self.tokens_by_id.insert(&final_token_id, &token).is_none(),
            "Token already exists"
        );

        //insert the token ID and metadata
        self.token_metadata_by_id.insert(&final_token_id, &metadata);

        //call the internal method for adding the token to the owner
        self.internal_add_token_to_owner(&token.owner_id, &final_token_id);

        // Construct the mint log as per the events standard.
        let nft_mint_log: EventLog = EventLog {
            // Standard name ("nep171").
            standard: NFT_STANDARD_NAME.to_string(),
            // Version of the standard ("nft-1.0.0").
            version: NFT_METADATA_SPEC.to_string(),
            // The data related with the event stored in a vector.
            event: EventLogVariant::NftMint(vec![NftMintLog {
                // Owner of the token.
                owner_id: token.owner_id.to_string(),
                // Vector of token IDs that were minted.
                token_ids: vec![final_token_id.to_string()],
                // An optional memo to include.
                memo: None,
            }]),
        };

        // Log the serialized json.
        env::log_str(&nft_mint_log.to_string());
        if curr_time > PRESALE_TIME && curr_time < PUBSALE_TIME {
            if self.presale_minted.contains_key(&caller) {
                let minted = self.presale_minted.get(&caller).unwrap();
                self.presale_minted.remove(&caller.clone().into());
                self.presale_minted.insert(&caller.clone().into(), &(minted + 1));
            } else {
                self.presale_minted.insert(&caller.clone().into(), &(1));
            }
        } else {
            if self.pubsale_minted.contains_key(&caller) {
                let minted = self.pubsale_minted.get(&caller).unwrap();
                self.pubsale_minted.remove(&caller.clone().into());
                self.pubsale_minted.insert(&caller.clone().into(), &(minted + 1));
            } else {
                self.pubsale_minted.insert(&caller.clone().into(), &(1));
            }
        }
        nft_mint_log.to_string()
    }

    pub fn add_whitelist(&mut self, account_id: AccountId) {
        self.assert_owner();
        self.whitelist.insert(&account_id, &(true));
    }

    pub fn remove_whitelist(&mut self, account_id: AccountId) {
        self.assert_owner();
        self.whitelist.remove(&account_id);
    }

    pub fn add_whitelists(&mut self, account_ids: Vec<AccountId>) {
        self.assert_owner();
        for account_id in account_ids {
            if !self.whitelist.contains_key(&account_id) {
                self.whitelist.insert(&account_id, &(true));
            }
        }
    }

    pub fn remove_whitelists(&mut self, account_ids: Vec<AccountId>) {
        self.assert_owner();
        for account_id in account_ids {
            if self.whitelist.contains_key(&account_id) {
                self.whitelist.remove(&account_id);
            }
        }
    }

    pub fn add_oglist(&mut self, account_id: AccountId) {
        self.assert_owner();
        self.oglist.insert(&account_id, &(true));
    }

    pub fn remove_oglist(&mut self, account_id: AccountId) {
        self.assert_owner();
        self.oglist.remove(&account_id);
    }

    pub fn add_oglists(&mut self, account_ids: Vec<AccountId>) {
        self.assert_owner();
        for account_id in account_ids {
            if !self.oglist.contains_key(&account_id) {
                self.oglist.insert(&account_id, &(true));
            }
        }
    }

    pub fn remove_oglists(&mut self, account_ids: Vec<AccountId>) {
        self.assert_owner();
        for account_id in account_ids {
            if self.oglist.contains_key(&account_id) {
                self.oglist.remove(&account_id);
            }
        }
    }

    pub fn is_whitelist(&self, account_id: AccountId) -> bool {
        return self.whitelist.contains_key(&account_id);
    }

    pub fn is_oglist(&self, account_id: AccountId) -> bool {
        return self.oglist.contains_key(&account_id);
    }

    pub fn get_sale_state(&self) -> u16 {
        let curr_time = env::block_timestamp() / 1_000_000;
        const PRESALE_TIME: u64 = 0; // 2nd July 2022 04:00PM UTC
        const PUBSALE_TIME: u64 = 1656788400000; // 2nd July 2022 07:00PM UTC
        if curr_time < PRESALE_TIME {
            return 0;
        } else if curr_time > PRESALE_TIME && curr_time < PUBSALE_TIME {
            return 1;
        } else {
            return 2;
        }
    }

    pub fn get_curr_time(&self) -> u64 {
        return env::block_timestamp() / 1_000_000;
    }

    pub fn get_remaining_ids(&self) -> Vec<u16> {
        return self.token_ids.clone();
    }

    pub fn get_total_supply(&self) -> u16 {
        return self.token_metadata_by_id.len().try_into().unwrap();
    }

    pub fn get_presale_amount(&self, account_id: AccountId) -> u128 {
        if self.presale_minted.contains_key(&account_id) {
            return self.presale_minted.get(&account_id).unwrap();
        }
        return 0;
    }

    pub fn get_pubsale_amount(&self, account_id: AccountId) -> u128 {
        if self.pubsale_minted.contains_key(&account_id) {
            return self.pubsale_minted.get(&account_id).unwrap();
        }
        return 0;
    }

    pub fn get_metadatas(&self) -> Vec<TokenMetadata> {
        let mut result:Vec<TokenMetadata> = Vec::new();
        let total_supply:u16 = self.token_metadata_by_id.len().try_into().unwrap();
        let mut token_id:TokenId;
        let mut metadata:TokenMetadata;
        for i in 1..total_supply + 1 {
            token_id = format!("{}", i);
            metadata = self.token_metadata_by_id.get(&token_id).unwrap();
            result.push(metadata);
        }
        return result;
    }
}