#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult};
use cw721_base::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cw721_simple::contract::{
    execute as cw721_execute, instantiate as cw721_instantiate, query as cw721_query,
};
use cw721_simple::error::ContractError;

type Extension = Option<Empty>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw721_instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<Extension, Empty>,
) -> Result<Response, ContractError> {
    cw721_execute(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg<Empty>) -> StdResult<Binary> {
    cw721_query::<Extension, Empty>(deps, env, msg)
}
