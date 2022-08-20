use cosmwasm_std::{Binary, Deps, StdResult, to_binary};
use cw721::{NftInfoResponse, NumTokensResponse};
use crate::msg::MinterResponse;
use crate::state::{CONTRACT_INFO, get_minter, token_count};

pub fn minter(deps: Deps) -> StdResult<Binary> {
    to_binary(&MinterResponse {
        minter: get_minter(deps.storage).to_string()
    })
}

pub fn contract_info(deps: Deps) -> StdResult<Binary> {
    to_binary(&CONTRACT_INFO.load(deps.storage)?)
}

pub fn num_tokens(deps: Deps) -> StdResult<Binary> {
    to_binary(&NumTokensResponse {
        count: token_count(deps.storage)?
    })
}

pub fn nft_info(
    deps: Deps,
    token_id: String,
) -> StdResult<Binary> {
    let info =
}