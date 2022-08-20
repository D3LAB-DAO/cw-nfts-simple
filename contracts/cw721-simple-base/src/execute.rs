use crate::error::ContractError;
use crate::msg::MintMsg;
use crate::state::{get_minter, increment_tokens, tokens, Approval, TokenInfo, OPERATORS};
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response};
use cw721::{CustomMsg, Expiration};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn mint<T, C>(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MintMsg<T>,
) -> Result<Response<C>, ContractError>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
{
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

    tokens().update(deps.storage, &msg.token_id, |old| match old {
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

pub fn approve<T, C>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
    expires: Option<Expiration>,
) -> Result<Response<C>, ContractError>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
{
    _update_approvals::<T>(deps, &env, &info, &spender, &token_id, true, expires)?;

    Ok(Response::new()
        .add_attribute("action", "approve")
        .add_attribute("sender", info.sender)
        .add_attribute("spender", spender)
        .add_attribute("token_id", token_id))
}

#[allow(clippy::too_many_arguments)]
pub fn _update_approvals<T>(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    spender: &str,
    token_id: &str,
    // if add == false, remove. if add == true, remove then set with this expiration
    add: bool,
    expires: Option<Expiration>,
) -> Result<TokenInfo<T>, ContractError>
where
    T: Serialize + DeserializeOwned + Clone,
{
    let mut token = tokens().load(deps.storage, token_id)?;
    // ensure we have permissions
    check_can_approve(deps.as_ref(), env, info, &token)?;

    // update the approval list (remove any for the same spender before adding)
    let spender_addr = deps.api.addr_validate(spender)?;
    token.approvals = token
        .approvals
        .into_iter()
        .filter(|apr| apr.spender != spender_addr)
        .collect();

    // only difference between approve and revoke
    if add {
        // reject expired data as invalid
        let expires = expires.unwrap_or_default();
        if expires.is_expired(&env.block) {
            return Err(ContractError::Expired {});
        }
        let approval = Approval {
            spender: spender_addr,
            expires,
        };
        token.approvals.push(approval);
    }

    tokens().save(deps.storage, token_id, &token)?;

    Ok(token)
}

/// returns true iff the sender can execute approve or reject on the contract
pub fn check_can_approve<T>(
    deps: Deps,
    env: &Env,
    info: &MessageInfo,
    token: &TokenInfo<T>,
) -> Result<(), ContractError>
where
    T: Serialize + DeserializeOwned + Clone,
{
    // owner can approve
    if token.owner == info.sender {
        return Ok(());
    }
    // operator can approve
    let op = OPERATORS.may_load(deps.storage, (&token.owner, &info.sender))?;
    match op {
        Some(ex) => {
            if ex.is_expired(&env.block) {
                Err(ContractError::Unauthorized {})
            } else {
                Ok(())
            }
        }
        None => Err(ContractError::Unauthorized {}),
    }
}
