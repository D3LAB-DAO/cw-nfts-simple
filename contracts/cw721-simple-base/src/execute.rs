use crate::error::ContractError;
use crate::msg::MintMsg;
use crate::state::{get_minter, increment_tokens, TokenInfo, tokens};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

pub fn mint<T>(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MintMsg<T>,
) -> Result<Response, ContractError>
    where T: Serialize + DeserializeOwned + Clone {
    let minter = get_minter(deps.storage);

    if info.sender != minter {
        return Err(ContractError::Unauthorized {});
    }

    // create the token
    let token = TokenInfo {
        owner: deps.api.addr_validate(&msg.owner)?,
        approvals: vec![],
        token_uri: msg.token_uri,
        extension: msg.extension,
    };

    tokens()
        .update(deps.storage, &msg.token_id, |old| match old {
            Some(_) => Err(ContractError::Claimed {}),
            None => Ok(token),
        })?;

    increment_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("minter", info.sender)
        .add_attribute("owner", msg.owner)
        .add_attribute("token_id", msg.token_id))
}
