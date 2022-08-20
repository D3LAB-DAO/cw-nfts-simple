use crate::error::ContractError;
use crate::execute;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{set_contract_info, set_minter};
use cosmwasm_std::{entry_point, DepsMut, Empty, Env, MessageInfo, Response};
use cw2::set_contract_version;
use cw721::ContractInfoResponse;

const CONTRACT_NAME: &str = "crates.io:cw721-simple-base";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

type Extension = Option<Empty>;

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
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<Extension, Empty>,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint(msg) => execute::mint(deps, env, info, msg),
        ExecuteMsg::Approve {
            spender,
            token_id,
            expires,
        } => execute::approve::<Extension, Empty>(deps, env, info, spender, token_id, expires),
        ExecuteMsg::Revoke { spender, token_id } => {
            execute::revoke::<Extension, Empty>(deps, env, info, spender, token_id)
        }
        ExecuteMsg::ApproveAll { operator, expires } => {
            execute::approve_all(deps, env, info, operator, expires)
        }
        _ => Ok(Response::new()),
    }
}

#[cfg(test)]
pub mod contract_tests {
    use crate::contract::{execute, instantiate};
    use crate::error::ContractError;
    use crate::msg::{ExecuteMsg, InstantiateMsg, MintMsg};
    use crate::state::{tokens, TokenInfo};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, DepsMut, Empty, Response};
    use cw721::Expiration;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    const ADDR1: &str = "juno18zfp9u7zxg3gel4r3txa2jqxme7jkw7d972flm";
    const ADDR2: &str = "osmo18zfp9u7zxg3gel4r3txa2jqxme7jkw7dmh6zw4";

    type Extension = Option<Empty>;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    struct CustomInfo {
        name: String,
        url: String,
    }

    fn init(deps: DepsMut) {
        instantiate(
            deps,
            mock_env(),
            mock_info(ADDR1, &[]),
            InstantiateMsg {
                name: "cw721".to_string(),
                symbol: "cw721".to_string(),
                minter: ADDR1.to_string(),
            },
        )
        .unwrap();
    }

    fn mint(deps: DepsMut) -> Result<Response, ContractError> {
        let execute_mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
            token_id: "1".to_string(),
            owner: ADDR1.to_string(),
            token_uri: None,
            extension: None,
        });

        execute(deps, mock_env(), mock_info(ADDR1, &[]), execute_mint_msg)
    }

    fn approve(deps: DepsMut) -> Result<Response, ContractError> {
        let valid_approve_msg = ExecuteMsg::Approve {
            spender: ADDR2.to_string(),
            token_id: "1".to_string(),
            expires: Some(Expiration::AtHeight(50000)),
        };

        execute(deps, mock_env(), mock_info(ADDR1, &[]), valid_approve_msg)
    }

    #[test]
    fn test_execute_mint() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());
        let res = mint(deps.as_mut()).unwrap();

        let token_1: TokenInfo<Extension> = tokens().load(&deps.storage, "1").unwrap();

        assert_eq!(token_1.owner, ADDR1);
        assert_eq!(token_1.extension, None);
        assert_eq!(
            res.attributes,
            [
                attr("action", "mint"),
                attr("minter", ADDR1),
                attr("owner", ADDR1),
                attr("token_id", "1")
            ]
        )
    }

    #[test]
    fn test_approve() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());
        mint(deps.as_mut()).unwrap();
        let valid_res = approve(deps.as_mut()).unwrap();

        assert_eq!(
            valid_res.attributes,
            [
                attr("action", "approve"),
                attr("sender", ADDR1),
                attr("spender", ADDR2),
                attr("token_id", "1")
            ]
        );

        let expired_approve_msg = ExecuteMsg::Approve {
            spender: ADDR2.to_string(),
            token_id: "1".to_string(),
            expires: Some(Expiration::AtHeight(100)),
        };

        let invalid_res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            expired_approve_msg,
        )
        .unwrap_err();

        assert_eq!(invalid_res, ContractError::Expired {});
    }

    #[test]
    fn test_revoke() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());
        mint(deps.as_mut()).unwrap();
        approve(deps.as_mut()).unwrap();

        let revoke_msg = ExecuteMsg::Revoke {
            spender: ADDR1.to_string(),
            token_id: "1".to_string(),
        };

        let revoke_res =
            execute(deps.as_mut(), mock_env(), mock_info(ADDR1, &[]), revoke_msg).unwrap();
        assert_eq!(
            revoke_res.attributes,
            [
                attr("action", "revoke"),
                attr("sender", ADDR1),
                attr("spender", ADDR1),
                attr("token_id", "1")
            ]
        );
    }

    #[test]
    fn test_approve_all() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());
        mint(deps.as_mut()).unwrap();

        let approve_all_msg = ExecuteMsg::ApproveAll {
            operator: ADDR2.to_string(),
            expires: Some(Expiration::AtHeight(50000)),
        };

        let approve_all_res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            approve_all_msg,
        )
        .unwrap();
        assert_eq!(
            approve_all_res.attributes,
            [
                attr("action", "approve_all"),
                attr("sender", "juno18zfp9u7zxg3gel4r3txa2jqxme7jkw7d972flm"),
                attr("operator", "osmo18zfp9u7zxg3gel4r3txa2jqxme7jkw7dmh6zw4")
            ]
        );

        let expired_approve_all_msg = ExecuteMsg::ApproveAll {
            operator: ADDR2.to_string(),
            expires: Some(Expiration::AtHeight(10)),
        };

        let expired_approve_all_res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            expired_approve_all_msg,
        )
        .unwrap_err();
        assert_eq!(expired_approve_all_res, ContractError::Expired {});
    }
}
