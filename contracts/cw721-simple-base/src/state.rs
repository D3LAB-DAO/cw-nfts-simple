use cosmwasm_std::{Addr, BlockInfo};
use cw_storage_plus::{Index, IndexedMap, IndexList, Item, Map, MultiIndex};
use schemars::{JsonSchema};
use serde::{Deserialize, Serialize};
use cw721::ContractInfoResponse;
use cw_utils::Expiration;
use serde::de::DeserializeOwned;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TokenCount {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo<T> {
    /// The owner of the newly minted NFT
    pub owner: Addr,
    /// Approvals are stored here, as we clear them all upon transfer and cannot accumulate much
    pub approvals: Vec<Approval>,

    /// Universal resource identifier for this NFT
    /// Should point to a JSON file that conforms to the ERC721
    /// Metadata JSON Schema
    pub token_uri: Option<String>,

    /// You can add any custom metadata here when you extend cw721-base
    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Approval {
    /// Account that can transfer/send the token
    pub spender: Addr,
    /// When the Approval expires (maybe Expiration::never)
    pub expires: Expiration,
}

impl Approval {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires.is_expired(block)
    }
}

pub struct TokenIndexes<'a, T>
    where T: Serialize + DeserializeOwned + Clone {
    pub owner: MultiIndex<'a, Addr, TokenInfo<T>, String>,
}

impl<'a, T> IndexList<TokenInfo<T>> for TokenIndexes<'a, T>
    where T: Serialize + DeserializeOwned + Clone {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item=&'_ dyn Index<TokenInfo<T>>> + '_> {
        let v: Vec<&dyn Index<TokenInfo<T>>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}


pub const CONTRACT_INFO: Item<ContractInfoResponse> = Item::new("contract_info");
pub const MINTER: Item<Addr> = Item::new("minter");
pub const OPERATORS: Map<(&Addr, &Addr), Expiration> = Map::new("operators");