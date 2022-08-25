use crate::state::{
    get_minter, get_tokens, token_count, Approval, TokenInfo, CONTRACT_INFO, OPERATORS,
};
use cosmwasm_std::{to_binary, Addr, Binary, BlockInfo, Deps, Env, Order, StdError, StdResult};
use cw721::{
    AllNftInfoResponse, ApprovalResponse, ApprovalsResponse, Expiration, NftInfoResponse,
    NumTokensResponse, OperatorsResponse, OwnerOfResponse, TokensResponse,
};
use cw721_base::MinterResponse;
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;
use serde::de::DeserializeOwned;
use serde::Serialize;

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 100;

pub fn minter(deps: Deps) -> StdResult<Binary> {
    to_binary(&MinterResponse {
        minter: get_minter(deps.storage).to_string(),
    })
}

pub fn contract_info(deps: Deps) -> StdResult<Binary> {
    to_binary(&CONTRACT_INFO.load(deps.storage)?)
}

pub fn num_tokens(deps: Deps) -> StdResult<Binary> {
    to_binary(&NumTokensResponse {
        count: token_count(deps.storage)?,
    })
}

pub fn nft_info<T>(deps: Deps, token_id: String) -> StdResult<Binary>
where
    T: Serialize + DeserializeOwned + Clone,
{
    let info: TokenInfo<T> = get_tokens().load(deps.storage, &token_id)?;
    to_binary(&NftInfoResponse {
        token_uri: info.token_uri,
        extension: info.extension,
    })
}

pub fn owner_of<T>(
    deps: Deps,
    env: Env,
    token_id: String,
    include_expired: bool,
) -> StdResult<Binary>
where
    T: Serialize + DeserializeOwned + Clone,
{
    let info: TokenInfo<T> = get_tokens().load(deps.storage, &token_id)?;
    to_binary(&OwnerOfResponse {
        owner: info.owner.to_string(),
        approvals: humanize_approvals(&env.block, &info, include_expired),
    })
}

fn humanize_approvals<T>(
    block: &BlockInfo,
    info: &TokenInfo<T>,
    include_expired: bool,
) -> Vec<cw721::Approval>
where
    T: Serialize + DeserializeOwned + Clone,
{
    info.approvals
        .iter()
        .filter(|apr| include_expired || !apr.is_expired(block))
        .map(humanize_approval)
        .collect()
}

fn humanize_approval(approval: &Approval) -> cw721::Approval {
    cw721::Approval {
        spender: approval.spender.to_string(),
        expires: approval.expires,
    }
}

/// operators returns all operators owner given access to
pub fn operators(
    deps: Deps,
    env: Env,
    owner: String,
    include_expired: bool,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_addr.as_ref().map(Bound::exclusive);

    let owner_addr = deps.api.addr_validate(&owner)?;
    let res: StdResult<Vec<_>> = OPERATORS
        .prefix(&owner_addr)
        .range(deps.storage, start, None, Order::Ascending)
        .filter(|r| include_expired || r.is_err() || !r.as_ref().unwrap().1.is_expired(&env.block))
        .take(limit)
        .map(parse_approval)
        .collect();
    to_binary(&OperatorsResponse { operators: res? })
}

pub fn approval<T>(
    deps: Deps,
    env: Env,
    token_id: String,
    spender: String,
    include_expired: bool,
) -> StdResult<Binary>
where
    T: Serialize + DeserializeOwned + Clone,
{
    let token: TokenInfo<T> = get_tokens().load(deps.storage, &token_id)?;

    // token owner has absolute approval
    if token.owner == spender {
        let approval = cw721::Approval {
            spender: token.owner.to_string(),
            expires: Expiration::Never {},
        };
        return to_binary(&ApprovalResponse { approval });
    }

    let filtered: Vec<_> = token
        .approvals
        .into_iter()
        .filter(|t| t.spender == spender)
        .filter(|t| include_expired || !t.is_expired(&env.block))
        .map(|a| cw721::Approval {
            spender: a.spender.into_string(),
            expires: a.expires,
        })
        .collect();

    if filtered.is_empty() {
        return Err(StdError::not_found("Approval not found"));
    }
    // we expect only one item
    let approval = filtered[0].clone();

    to_binary(&ApprovalResponse { approval })
}

/// approvals returns all approvals owner given access to
pub fn approvals<T>(
    deps: Deps,
    env: Env,
    token_id: String,
    include_expired: bool,
) -> StdResult<Binary>
where
    T: Serialize + DeserializeOwned + Clone,
{
    let token: TokenInfo<T> = get_tokens().load(deps.storage, &token_id)?;
    let approvals: Vec<_> = token
        .approvals
        .into_iter()
        .filter(|t| include_expired || !t.is_expired(&env.block))
        .map(|a| cw721::Approval {
            spender: a.spender.into_string(),
            expires: a.expires,
        })
        .collect();

    to_binary(&ApprovalsResponse { approvals })
}

pub fn tokens<T>(
    deps: Deps,
    owner: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary>
where
    T: Serialize + DeserializeOwned + Clone,
{
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let owner_addr = deps.api.addr_validate(&owner)?;
    let tokens: Vec<String> = get_tokens::<T>()
        .idx
        .owner
        .prefix(owner_addr)
        .keys(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    to_binary(&TokensResponse { tokens })
}

pub fn all_tokens<T>(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary>
where
    T: Serialize + DeserializeOwned + Clone,
{
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let tokens: StdResult<Vec<String>> = get_tokens::<T>()
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(k, _)| k))
        .collect();

    to_binary(&TokensResponse { tokens: tokens? })
}

pub fn all_nft_info<T>(
    deps: Deps,
    env: Env,
    token_id: String,
    include_expired: bool,
) -> StdResult<Binary>
where
    T: Serialize + DeserializeOwned + Clone,
{
    let info: TokenInfo<T> = get_tokens().load(deps.storage, &token_id)?;
    to_binary(&AllNftInfoResponse {
        access: OwnerOfResponse {
            owner: info.owner.to_string(),
            approvals: humanize_approvals(&env.block, &info, include_expired),
        },
        info: NftInfoResponse {
            token_uri: info.token_uri,
            extension: info.extension,
        },
    })
}

fn parse_approval(item: StdResult<(Addr, Expiration)>) -> StdResult<cw721::Approval> {
    item.map(|(spender, expires)| cw721::Approval {
        spender: spender.to_string(),
        expires,
    })
}
