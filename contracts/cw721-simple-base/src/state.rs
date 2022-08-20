use cosmwasm_std::{Addr, BlockInfo};
use cw721::ContractInfoResponse;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

const TOKENS_KEY: &str = "tokens";
const TOKENS_OWNER_KEY: &str = "tokens__owner";

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
    where
        T: Serialize + DeserializeOwned + Clone,
{
    // An owner can have multiple tokens, which has string type TokenPK
    pub owner: MultiIndex<'a, Addr, TokenInfo<T>, String>,
}

impl<'a, T> IndexList<TokenInfo<T>> for TokenIndexes<'a, T>
    where
        T: Serialize + DeserializeOwned + Clone,
{
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item=&'_ dyn Index<TokenInfo<T>>> + '_> {
        let v: Vec<&dyn Index<TokenInfo<T>>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn tokens<'a, T>() -> IndexedMap<'a, &'a str, TokenInfo<T>, TokenIndexes<'a, T>>
    where
        T: Serialize + DeserializeOwned + Clone,
{
    let indexes = TokenIndexes {
        owner: MultiIndex::new(
            |d: &TokenInfo<T>| d.owner.clone(),
            TOKENS_KEY,
            TOKENS_OWNER_KEY,
        ),
    };
    IndexedMap::new(TOKENS_KEY, indexes)
}

pub const CONTRACT_INFO: Item<ContractInfoResponse> = Item::new("contract_info");
pub const MINTER: Item<Addr> = Item::new("minter");
pub const OPERATORS: Map<(&Addr, &Addr), Expiration> = Map::new("operators");

#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;
    use crate::error::ContractError;
    use crate::state::TokenInfo;
    use crate::state::tokens;
    use serde::{Deserialize, Serialize};

    #[test]
    pub fn test_multi_index() {
        use cosmwasm_std::testing::mock_dependencies;
        let mut deps = mock_dependencies();

        let owner: &str = "ADDR1";
        let owner_addr = Addr::unchecked(owner);
        let token_id: &str = "1";

        #[derive(Serialize, Deserialize, Clone)]
        struct CustomInfo {
            name: String,
            url: String,
        }

        let new_token = TokenInfo::<CustomInfo> {
            owner: owner_addr.clone(),
            approvals: vec![],
            token_uri: None,
            extension: CustomInfo {
                name: "test_nft".to_string(),
                url: "https://test_nft.com".to_string(),
            },
        };

        tokens()
            .update(&mut deps.storage, token_id, |old| match old {
                Some(_) => Err(ContractError::Claimed {}),
                None => Ok(new_token.clone()),
            })
            .unwrap();

        let token: TokenInfo<CustomInfo> = tokens().load(&deps.storage, token_id).unwrap();

        assert_eq!(token.owner, owner_addr);
        assert_eq!(token.extension.name, "test_nft");
    }
}
