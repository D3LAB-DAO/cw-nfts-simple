use std::fmt::Debug;
use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response};
use schemars::JsonSchema;
use crate::error::ContractError;
use crate::execute::mint;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{set_contract_info, set_minter};
use cw2::set_contract_version;
use cw721::ContractInfoResponse;
use serde::de::DeserializeOwned;
use serde::Serialize;

const CONTRACT_NAME: &str = "crates.io:cw721-simple-base";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let contract_info = ContractInfoResponse {
        name: msg.name.to_string(),
        symbol: msg.symbol.to_string(),
    };
    let minter = deps.api.addr_validate(&msg.minter)?;

    set_contract_info(deps.storage, contract_info)?;
    set_minter(deps.storage, minter)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute<T>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<T>,
) -> Result<Response, ContractError>
    where T: Serialize + DeserializeOwned + Clone {
    match msg {
        ExecuteMsg::Mint(msg) => mint(deps, env, info, msg),
        _ => Ok(Response::new()),
    }
}

#[cfg(test)]
pub mod contract_tests {
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{DepsMut, Env, MessageInfo};
    use crate::contract::{instantiate, execute};
    use crate::msg::{ExecuteMsg, InstantiateMsg, MintMsg};
    use crate::state::{TokenInfo, tokens};

    const ADDR1: &str = "juno18zfp9u7zxg3gel4r3txa2jqxme7jkw7d972flm";

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    struct CustomInfo {
        name: String,
        url: String,
    }


    fn init(deps: DepsMut, env: Env, info: MessageInfo) {
        instantiate(
            deps,
            env,
            info,
            InstantiateMsg {
                name: "cw721".to_string(),
                symbol: "cw721".to_string(),
                minter: ADDR1.to_string(),
            },
        ).unwrap();
    }

    fn mint(deps: DepsMut, env: Env, info: MessageInfo) {
        let execute_mint_msg = ExecuteMsg::Mint(MintMsg {
            token_id: "1".to_string(),
            owner: ADDR1.to_string(),
            token_uri: None,
            extension: CustomInfo {
                name: "token_1".to_string(),
                url: "https://token1.test".to_string(),
            },
        });

        execute(deps, env, info, execute_mint_msg).unwrap();
    }

    #[test]
    fn test_execute_mint() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[]);

        init(deps.as_mut(), env.clone(), info.clone());
        mint(deps.as_mut(), env, info);

        let token_1: TokenInfo<CustomInfo> = tokens().load(&deps.storage, "1").unwrap();

        assert_eq!(token_1.extension.name, "token_1");
        assert_eq!(token_1.extension.url, "https://token1.test");
    }
}
