use crate::error::ContractError;
use cosmwasm_std::{Addr, BlockInfo, Response, StdResult, Storage};
use cw721::{ContractInfoResponse, Expiration};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

const CONTRACT_KEY: &str = "nft_info";
const MINTER_KEY: &str = "minter";
const TOKEN_COUNT_KEY: &str = "num_tokens";
const OPERATORS_KEY: &str = "operators";
const TOKENS_KEY: &str = "tokens";
const TOKENS_OWNER_KEY: &str = "tokens__owner";

pub const CONTRACT_INFO: Item<ContractInfoResponse> = Item::new(CONTRACT_KEY);
pub const MINTER: Item<Addr> = Item::new(MINTER_KEY);
pub const TOKENS_COUNT: Item<u64> = Item::new(TOKEN_COUNT_KEY);
pub const OPERATORS: Map<(&Addr, &Addr), Expiration> = Map::new(OPERATORS_KEY);

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

    /// You can add any custom metadata here when you extend cw721-simple-base
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
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<TokenInfo<T>>> + '_> {
        let v: Vec<&dyn Index<TokenInfo<T>>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn get_tokens<'a, T>() -> IndexedMap<'a, &'a str, TokenInfo<T>, TokenIndexes<'a, T>>
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

pub fn token_count(storage: &dyn Storage) -> StdResult<u64> {
    Ok(TOKENS_COUNT.may_load(storage)?.unwrap_or_default())
}

pub fn increment_tokens(storage: &mut dyn Storage) -> StdResult<u64> {
    let val = token_count(storage)? + 1;
    TOKENS_COUNT.save(storage, &val)?;
    Ok(val)
}

pub fn decrement_tokens(storage: &mut dyn Storage) -> StdResult<u64> {
    let val = token_count(storage)? - 1;
    TOKENS_COUNT.save(storage, &val)?;
    Ok(val)
}

pub fn set_contract_info(
    storage: &mut dyn Storage,
    info: ContractInfoResponse,
) -> Result<Response, ContractError> {
    let res = CONTRACT_INFO.save(storage, &info);
    match res {
        Ok(_) => Ok(Response::new()),
        Err(_) => Err(ContractError::ContractInfoSaveError {}),
    }
}

pub fn set_minter(storage: &mut dyn Storage, minter: Addr) -> Result<Response, ContractError> {
    let res = MINTER.save(storage, &minter);
    match res {
        Ok(_) => Ok(Response::new()),
        Err(_) => Err(ContractError::MinterSaveError {}),
    }
}

pub fn get_minter(storage: &dyn Storage) -> Addr {
    MINTER.load(storage).unwrap_or_else(|_| Addr::unchecked(""))
}

#[cfg(test)]
mod state_tests {
    use crate::error::ContractError;
    use crate::state::token_count;
    use crate::state::tokens;
    use crate::state::{decrement_tokens, increment_tokens, TokenInfo};
    use cosmwasm_std::{Addr, Empty};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[test]
    pub fn state_test() {
        use cosmwasm_std::testing::mock_dependencies;
        let mut deps = mock_dependencies();

        let owner: &str = "ADDR1";
        let owner_addr = Addr::unchecked(owner);

        let token_1_id: &str = "1";
        let token_2_id: &str = "2";

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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

        let empty_custom_token = TokenInfo::<Empty> {
            owner: owner_addr.clone(),
            approvals: vec![],
            token_uri: None,
            extension: Empty {},
        };

        // Mint token if not claimed before else ContractError::Claimed
        get_tokens()
            .update(&mut deps.storage, token_1_id, |old| match old {
                Some(_) => Err(ContractError::Claimed {}),
                None => Ok(new_token.clone()),
            })
            .unwrap();

        let token_count_after_increment_1 = increment_tokens(&mut deps.storage).unwrap_or_default();
        assert_eq!(token_count_after_increment_1, 1);

        get_tokens()
            .update(&mut deps.storage, token_2_id, |old| match old {
                Some(_) => Err(ContractError::Claimed {}),
                None => Ok(empty_custom_token),
            })
            .unwrap();

        // Minting nft with same id will fail
        let wrong_token_update =
            get_tokens().update(&mut deps.storage, token_1_id, |old| match old {
                Some(_) => Err(ContractError::Claimed {}),
                None => Ok(new_token.clone()),
            });

        assert!(wrong_token_update.is_err());

        let token_count_after_increment_2 = increment_tokens(&mut deps.storage).unwrap_or_default();
        assert_eq!(token_count_after_increment_2, 2);

        let token_1: TokenInfo<CustomInfo> = get_tokens().load(&deps.storage, token_1_id).unwrap();
        let token_2: TokenInfo<Empty> = get_tokens().load(&deps.storage, token_2_id).unwrap();

        assert_eq!(token_1.owner, owner_addr);
        assert_eq!(token_1.extension.name, "test_nft");
        assert_eq!(token_2.owner, owner_addr);
        assert_eq!(token_2.extension, Empty {});

        let count = token_count(&deps.storage).unwrap_or_default();
        assert_eq!(count, 2);

        let token_count_after_decrement = decrement_tokens(&mut deps.storage).unwrap_or_default();
        assert_eq!(token_count_after_decrement, 1);
    }
}
