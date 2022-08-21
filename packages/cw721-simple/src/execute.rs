use crate::error::ContractError;
use crate::msg::MintMsg;
use crate::state::{
    decrement_tokens, get_minter, get_tokens, increment_tokens, Approval, TokenInfo, OPERATORS,
};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw721::{CustomMsg, Cw721ReceiveMsg, Expiration};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use std::fmt::Debug;

pub fn mint<T, C, E>(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MintMsg<T>,
) -> Result<Response<C>, ContractError<E>>
where
    T: Serialize + DeserializeOwned + Clone,
    E: Debug + PartialEq + Error,
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

    get_tokens().update(deps.storage, &msg.token_id, |old| match old {
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

pub fn approve<T, C, E>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
    expires: Option<Expiration>,
) -> Result<Response<C>, ContractError<E>>
where
    T: Serialize + DeserializeOwned + Clone,
    E: Debug + PartialEq + Error,
    C: CustomMsg,
{
    _update_approvals::<T, E>(deps, &env, &info, &spender, &token_id, true, expires)?;

    Ok(Response::new()
        .add_attribute("action", "approve")
        .add_attribute("sender", info.sender)
        .add_attribute("spender", spender)
        .add_attribute("token_id", token_id))
}

pub fn revoke<T, C, E>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
) -> Result<Response<C>, ContractError<E>>
where
    T: Serialize + DeserializeOwned + Clone,
    E: Debug + PartialEq + Error,
    C: CustomMsg,
{
    _update_approvals::<T, E>(deps, &env, &info, &spender, &token_id, false, None)?;

    Ok(Response::new()
        .add_attribute("action", "revoke")
        .add_attribute("sender", info.sender)
        .add_attribute("spender", spender)
        .add_attribute("token_id", token_id))
}

pub fn approve_all<C, E>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: String,
    expires: Option<Expiration>,
) -> Result<Response<C>, ContractError<E>>
where
    E: Debug + PartialEq + Error,
    C: CustomMsg,
{
    // reject expired data as invalid
    let expires = expires.unwrap_or_default();
    if expires.is_expired(&env.block) {
        return Err(ContractError::Expired {});
    }

    // set the operator for us
    let operator_addr = deps.api.addr_validate(&operator)?;
    OPERATORS.save(deps.storage, (&info.sender, &operator_addr), &expires)?;

    Ok(Response::new()
        .add_attribute("action", "approve_all")
        .add_attribute("sender", info.sender)
        .add_attribute("operator", operator))
}

pub fn revoke_all<C, E>(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: String,
) -> Result<Response<C>, ContractError<E>>
where
    E: Debug + PartialEq + Error,
    C: CustomMsg,
{
    let operator_addr = deps.api.addr_validate(&operator)?;
    OPERATORS.remove(deps.storage, (&info.sender, &operator_addr));

    Ok(Response::new()
        .add_attribute("action", "revoke_all")
        .add_attribute("sender", info.sender)
        .add_attribute("operator", operator))
}

pub fn burn<T, C, E>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response<C>, ContractError<E>>
where
    T: Serialize + DeserializeOwned + Clone,
    E: Debug + PartialEq + Error,
    C: CustomMsg,
{
    let token = get_tokens().load(deps.storage, &token_id)?;
    check_can_send::<T, E>(deps.as_ref(), &env, &info, &token)?;

    get_tokens::<T>().remove(deps.storage, &token_id)?;
    decrement_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "burn")
        .add_attribute("sender", info.sender)
        .add_attribute("token_id", token_id))
}

pub fn transfer_nft<T, C, E>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> Result<Response<C>, ContractError<E>>
where
    T: Serialize + DeserializeOwned + Clone,
    E: Debug + PartialEq + Error,
    C: CustomMsg,
{
    _transfer_nft::<T, E>(deps, &env, &info, &recipient, &token_id)?;

    Ok(Response::new()
        .add_attribute("action", "transfer_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("token_id", token_id))
}

pub fn send_nft<T, C, E>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    token_id: String,
    msg: Binary,
) -> Result<Response<C>, ContractError<E>>
where
    T: Serialize + DeserializeOwned + Clone,
    E: Debug + PartialEq + Error,
    C: CustomMsg,
{
    // Transfer token
    _transfer_nft::<T, E>(deps, &env, &info, &contract, &token_id)?;

    let send = Cw721ReceiveMsg {
        sender: info.sender.to_string(),
        token_id: token_id.clone(),
        msg,
    };

    // Send message
    Ok(Response::new()
        .add_message(send.into_cosmos_msg(contract.clone())?)
        .add_attribute("action", "send_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", contract)
        .add_attribute("token_id", token_id))
}

fn _transfer_nft<T, E>(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    recipient: &str,
    token_id: &str,
) -> Result<TokenInfo<T>, ContractError<E>>
where
    T: Serialize + DeserializeOwned + Clone,
    E: Debug + PartialEq + Error,
{
    let mut token = get_tokens().load(deps.storage, token_id)?;
    // ensure we have permissions
    check_can_send(deps.as_ref(), env, info, &token)?;
    // set owner and remove existing approvals
    token.owner = deps.api.addr_validate(recipient)?;
    token.approvals = vec![];
    get_tokens().save(deps.storage, token_id, &token)?;
    Ok(token)
}

#[allow(clippy::too_many_arguments)]
fn _update_approvals<T, E>(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    spender: &str,
    token_id: &str,
    // if add == false, remove. if add == true, remove then set with this expiration
    add: bool,
    expires: Option<Expiration>,
) -> Result<TokenInfo<T>, ContractError<E>>
where
    T: Serialize + DeserializeOwned + Clone,
    E: Debug + PartialEq + Error,
{
    let mut token = get_tokens().load(deps.storage, token_id)?;
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

    get_tokens().save(deps.storage, token_id, &token)?;

    Ok(token)
}

/// returns true iff the sender can execute approve or reject on the contract
pub fn check_can_approve<T, E>(
    deps: Deps,
    env: &Env,
    info: &MessageInfo,
    token: &TokenInfo<T>,
) -> Result<(), ContractError<E>>
where
    T: Serialize + DeserializeOwned + Clone,
    E: Debug + PartialEq + Error,
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

/// returns true iff the sender can transfer ownership of the token
pub fn check_can_send<T, E>(
    deps: Deps,
    env: &Env,
    info: &MessageInfo,
    token: &TokenInfo<T>,
) -> Result<(), ContractError<E>>
where
    T: Serialize + DeserializeOwned + Clone,
    E: Debug + PartialEq + Error,
{
    // owner can send
    if token.owner == info.sender {
        return Ok(());
    }

    // any non-expired token approval can send
    if token
        .approvals
        .iter()
        .any(|apr| apr.spender == info.sender && !apr.is_expired(&env.block))
    {
        return Ok(());
    }

    // operator can send
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
