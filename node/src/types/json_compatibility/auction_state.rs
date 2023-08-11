// TODO - remove once schemars stops causing warning.
#![allow(clippy::field_reassign_with_default)]

use std::collections::{btree_map::Entry, BTreeMap};

use num_traits::Zero;
use once_cell::sync::Lazy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use casper_types::{
    system::auction::{
        BidKind, DelegationRate, Delegator, EraValidators, Staking, ValidatorBid, ValidatorBids,
    },
    AccessRights, Digest, EraId, PublicKey, SecretKey, URef, U512,
};

use crate::rpcs::docs::DocExample;

static ERA_VALIDATORS: Lazy<EraValidators> = Lazy::new(|| {
    let secret_key_1 = SecretKey::ed25519_from_bytes([42; SecretKey::ED25519_LENGTH]).unwrap();
    let public_key_1 = PublicKey::from(&secret_key_1);

    let mut validator_weights = BTreeMap::new();
    validator_weights.insert(public_key_1, U512::from(10));

    let mut era_validators = BTreeMap::new();
    era_validators.insert(EraId::from(10u64), validator_weights);

    era_validators
});
static BIDS: Lazy<ValidatorBids> = Lazy::new(|| {
    let bonding_purse = URef::new([250; 32], AccessRights::READ_ADD_WRITE);
    let staked_amount = U512::from(10);
    let release_era: u64 = 42;

    let validator_secret_key =
        SecretKey::ed25519_from_bytes([42; SecretKey::ED25519_LENGTH]).unwrap();
    let validator_public_key = PublicKey::from(&validator_secret_key);

    let validator_bid = ValidatorBid::locked(
        validator_public_key.clone(),
        bonding_purse,
        staked_amount,
        DelegationRate::zero(),
        release_era,
    );
    let mut bids = BTreeMap::new();
    bids.insert(validator_public_key, Box::new(validator_bid));

    bids
});
static AUCTION_INFO: Lazy<AuctionState> = Lazy::new(|| {
    let state_root_hash = Digest::from([11; Digest::LENGTH]);
    let validator_secret_key =
        SecretKey::ed25519_from_bytes([42; SecretKey::ED25519_LENGTH]).unwrap();
    let validator_public_key = PublicKey::from(&validator_secret_key);

    let mut bids = vec![];
    let validator_bid = ValidatorBid::unlocked(
        validator_public_key.clone(),
        URef::new([250; 32], AccessRights::READ_ADD_WRITE),
        U512::from(20),
        DelegationRate::zero(),
    );
    bids.push(BidKind::Validator(Box::new(validator_bid)));

    let delegator_secret_key =
        SecretKey::ed25519_from_bytes([43; SecretKey::ED25519_LENGTH]).unwrap();
    let delegator_public_key = PublicKey::from(&delegator_secret_key);
    let delegator_bid = Delegator::unlocked(
        delegator_public_key,
        U512::from(10),
        URef::new([251; 32], AccessRights::READ_ADD_WRITE),
        validator_public_key,
    );
    bids.push(BidKind::Delegator(Box::new(delegator_bid)));

    let height: u64 = 10;
    let era_validators = EraValidators::doc_example().clone();
    AuctionState::new(state_root_hash, height, era_validators, bids)
});

/// A validator's weight.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JsonValidatorWeights {
    public_key: PublicKey,
    weight: U512,
}

/// The validators for the given era.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JsonEraValidators {
    era_id: EraId,
    validator_weights: Vec<JsonValidatorWeights>,
}

/// A delegator associated with the given validator.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JsonDelegator {
    public_key: PublicKey,
    staked_amount: U512,
    bonding_purse: URef,
    delegatee: PublicKey,
}

/// An entry in a founding validator map representing a bid.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JsonBid {
    /// The purse that was used for bonding.
    bonding_purse: URef,
    /// The amount of tokens staked by a validator (not including delegators).
    staked_amount: U512,
    /// The delegation rate.
    delegation_rate: DelegationRate,
    /// The delegators.
    delegators: Vec<JsonDelegator>,
    /// Is this an inactive validator.
    inactive: bool,
}

impl JsonBid {
    pub(crate) fn new(
        validator_bid: ValidatorBid,
        delegators: BTreeMap<PublicKey, Delegator>,
    ) -> Self {
        let mut json_delegators: Vec<JsonDelegator> = Vec::with_capacity(delegators.len());
        for (public_key, delegator) in delegators.iter() {
            json_delegators.push(JsonDelegator {
                public_key: public_key.clone(),
                staked_amount: delegator.staked_amount(),
                bonding_purse: *delegator.bonding_purse(),
                delegatee: delegator.validator_public_key().clone(),
            });
        }
        JsonBid {
            bonding_purse: *validator_bid.bonding_purse(),
            staked_amount: validator_bid.staked_amount(),
            delegation_rate: *validator_bid.delegation_rate(),
            delegators: json_delegators,
            inactive: validator_bid.inactive(),
        }
    }
}

/// A Json representation of a single bid.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JsonBids {
    public_key: PublicKey,
    bid: JsonBid,
}

/// Data structure summarizing auction contract data.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuctionState {
    /// Global state hash.
    pub state_root_hash: Digest,
    /// Block height.
    pub block_height: u64,
    /// Era validators.
    pub era_validators: Vec<JsonEraValidators>,
    /// All bids contained within a vector.
    bids: Vec<JsonBids>,
}

impl AuctionState {
    /// Create new instance of `AuctionState`
    pub fn new(
        state_root_hash: Digest,
        block_height: u64,
        era_validators: EraValidators,
        bids: Vec<BidKind>,
    ) -> Self {
        let mut json_era_validators: Vec<JsonEraValidators> = Vec::new();
        for (era_id, validator_weights) in era_validators.iter() {
            let mut json_validator_weights: Vec<JsonValidatorWeights> = Vec::new();
            for (public_key, weight) in validator_weights.iter() {
                json_validator_weights.push(JsonValidatorWeights {
                    public_key: public_key.clone(),
                    weight: *weight,
                });
            }
            json_era_validators.push(JsonEraValidators {
                era_id: *era_id,
                validator_weights: json_validator_weights,
            });
        }

        let staking = {
            let mut staking: Staking = BTreeMap::new();
            for bid_kind in bids.iter().filter(|x| x.is_unified()) {
                if let BidKind::Unified(bid) = bid_kind {
                    let public_key = bid.validator_public_key().clone();
                    let validator_bid = ValidatorBid::unlocked(
                        bid.validator_public_key().clone(),
                        *bid.bonding_purse(),
                        *bid.staked_amount(),
                        *bid.delegation_rate(),
                    );
                    staking.insert(public_key, (validator_bid, bid.delegators().clone()));
                }
            }

            for bid_kind in bids.iter().filter(|x| x.is_validator()) {
                if let BidKind::Validator(validator_bid) = bid_kind {
                    let public_key = validator_bid.validator_public_key().clone();
                    staking.insert(public_key, (*validator_bid.clone(), BTreeMap::new()));
                }
            }

            for bid_kind in bids.iter().filter(|x| x.is_delegator()) {
                if let BidKind::Delegator(delegator_bid) = bid_kind {
                    let validator_public_key = delegator_bid.validator_public_key().clone();
                    if let Entry::Occupied(mut occupant) =
                        staking.entry(validator_public_key.clone())
                    {
                        let (_, delegators) = occupant.get_mut();
                        delegators.insert(
                            delegator_bid.delegator_public_key().clone(),
                            *delegator_bid.clone(),
                        );
                    }
                }
            }
            staking
        };

        let mut json_bids: Vec<JsonBids> = Vec::new();
        for (public_key, (validator_bid, delegators)) in staking {
            let json_bid = JsonBid::new(validator_bid, delegators);
            json_bids.push(JsonBids {
                public_key: public_key.clone(),
                bid: json_bid,
            });
        }

        AuctionState {
            state_root_hash,
            block_height,
            era_validators: json_era_validators,
            bids: json_bids,
        }
    }
}

impl DocExample for AuctionState {
    fn doc_example() -> &'static Self {
        &AUCTION_INFO
    }
}

impl DocExample for EraValidators {
    fn doc_example() -> &'static Self {
        &ERA_VALIDATORS
    }
}

impl DocExample for ValidatorBids {
    fn doc_example() -> &'static Self {
        &BIDS
    }
}
