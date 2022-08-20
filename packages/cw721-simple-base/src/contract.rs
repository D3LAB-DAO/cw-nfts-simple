use crate::error::ContractError;
use crate::execute;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{set_contract_info, set_minter};
use cosmwasm_std::{entry_point, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw721::{ContractInfoResponse, CustomMsg};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Binary;

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
pub fn execute<T, E, C>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<T, E>,
) -> Result<Response<C>, ContractError>
    where
        T: Serialize + DeserializeOwned + Clone,
        E: CustomMsg,
        C: CustomMsg,
{
    match msg {
        ExecuteMsg::Mint(msg) => execute::mint::<T, C>(deps, env, info, msg),
        ExecuteMsg::Approve {
            spender,
            token_id,
            expires,
        } => execute::approve::<T, C>(deps, env, info, spender, token_id, expires),
        ExecuteMsg::Revoke { spender, token_id } => {
            execute::revoke::<T, C>(deps, env, info, spender, token_id)
        }
        ExecuteMsg::ApproveAll { operator, expires } => {
            execute::approve_all::<C>(deps, env, info, operator, expires)
        }
        ExecuteMsg::RevokeAll { operator } => execute::revoke_all::<C>(deps, env, info, operator),
        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => execute::transfer_nft::<T, C>(deps, env, info, recipient, token_id),
        ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => execute::send_nft::<T, C>(deps, env, info, contract, token_id, msg),
        ExecuteMsg::Burn { token_id } => execute::burn::<T, C>(deps, env, info, token_id),
        ExecuteMsg::Extension { msg: _ } => Ok(Response::new()),
    }
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn query(
//     deps: Deps, env: Env, msg: QueryMsg<Empty>) -> StdResult<Binary> {
//     match msg {
//         QueryMsg::
//     }
// }

#[cfg(test)]
pub mod contract_tests {
    use crate::contract::{execute, instantiate};
    use crate::error::ContractError;
    use crate::msg::{ExecuteMsg, InstantiateMsg, MintMsg};
    use crate::state::{tokens, TokenInfo};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, to_binary, DepsMut, Empty, Response};
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

    fn mint(deps: DepsMut, owner: &str, token_id: &str) -> Result<Response, ContractError> {
        let execute_mint_msg = ExecuteMsg::<Extension, Empty>::Mint(MintMsg::<Extension> {
            token_id: token_id.to_string(),
            owner: owner.to_string(),
            token_uri: None,
            extension: None,
        });

        execute::<Extension, Empty, Empty>(deps, mock_env(), mock_info(ADDR1, &[]), execute_mint_msg)
    }

    fn approve(deps: DepsMut, sender: &str, spender: &str) -> Result<Response, ContractError> {
        let valid_approve_msg = ExecuteMsg::Approve {
            spender: spender.to_string(),
            token_id: "1".to_string(),
            expires: Some(Expiration::AtHeight(50000)),
        };

        execute::<Extension, Empty, Empty>(deps, mock_env(), mock_info(sender, &[]), valid_approve_msg)
    }

    fn transfer_nft(
        deps: DepsMut,
        sender: &str,
        recipient: &str,
    ) -> Result<Response, ContractError> {
        let transfer_nft_msg = ExecuteMsg::<Extension, Empty>::TransferNft {
            recipient: recipient.to_string(),
            token_id: "1".to_string(),
        };

        execute::<Extension, Empty, Empty>(deps, mock_env(), mock_info(sender, &[]), transfer_nft_msg)
    }

    #[test]
    fn test_execute_mint() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());
        let res = mint(deps.as_mut(), ADDR1, "1").unwrap();

        let token_1: TokenInfo<Extension> = tokens().load(&deps.storage, "1").unwrap();

        assert_eq!(token_1.owner, ADDR1);
        assert_eq!(token_1.extension, None);
        assert_eq!(
            res.attributes,
            [
                attr("action", "mint"),
                attr("minter", ADDR1),
                attr("owner", ADDR1),
                attr("token_id", "1"),
            ]
        )
    }

    #[test]
    fn test_approve() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());
        mint(deps.as_mut(), ADDR1, "1").unwrap();
        let valid_res = approve(deps.as_mut(), ADDR1, ADDR2).unwrap();

        assert_eq!(
            valid_res.attributes,
            [
                attr("action", "approve"),
                attr("sender", ADDR1),
                attr("spender", ADDR2),
                attr("token_id", "1"),
            ]
        );

        let expired_approve_msg = ExecuteMsg::<Extension, Empty>::Approve {
            spender: ADDR2.to_string(),
            token_id: "1".to_string(),
            expires: Some(Expiration::AtHeight(100)),
        };

        let invalid_res = execute::<Extension, Empty, Empty>(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            expired_approve_msg,
        )
            .unwrap_err();
        assert_eq!(invalid_res, ContractError::Expired {});

        // Unauthorized approve
        let unauthorized_res = approve(deps.as_mut(), ADDR2, ADDR2).unwrap_err();
        assert_eq!(unauthorized_res, ContractError::Unauthorized {});
    }

    #[test]
    fn test_revoke() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());
        mint(deps.as_mut(), ADDR1, "1").unwrap();
        approve(deps.as_mut(), ADDR1, ADDR2).unwrap();

        let revoke_msg = ExecuteMsg::<Extension, Empty>::Revoke {
            spender: ADDR1.to_string(),
            token_id: "1".to_string(),
        };

        let revoke_res =
            execute::<Extension, Empty, Empty>(deps.as_mut(), mock_env(), mock_info(ADDR1, &[]), revoke_msg).unwrap();
        assert_eq!(
            revoke_res.attributes,
            [
                attr("action", "revoke"),
                attr("sender", ADDR1),
                attr("spender", ADDR1),
                attr("token_id", "1"),
            ]
        );
    }

    #[test]
    fn test_approve_all() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());
        mint(deps.as_mut(), ADDR1, "1").unwrap();

        let approve_all_msg = ExecuteMsg::<Extension, Empty>::ApproveAll {
            operator: ADDR2.to_string(),
            expires: Some(Expiration::AtHeight(50000)),
        };

        let approve_all_res = execute::<Extension, Empty, Empty>(
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
                attr("sender", ADDR1),
                attr("operator", ADDR2),
            ]
        );

        let expired_approve_all_msg = ExecuteMsg::<Extension, Empty>::ApproveAll {
            operator: ADDR2.to_string(),
            expires: Some(Expiration::AtHeight(10)),
        };

        let expired_approve_all_res = execute::<Extension, Empty, Empty>(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            expired_approve_all_msg,
        )
            .unwrap_err();
        assert_eq!(expired_approve_all_res, ContractError::Expired {});
    }

    #[test]
    fn test_revoke_all() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());
        mint(deps.as_mut(), ADDR1, "1").unwrap();
        approve(deps.as_mut(), ADDR1, ADDR2).unwrap();

        let revoke_all_msg = ExecuteMsg::<Extension, Empty>::RevokeAll {
            operator: ADDR2.to_string(),
        };

        let revoke_all_res = execute::<Extension, Empty, Empty>(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            revoke_all_msg,
        )
            .unwrap();

        assert_eq!(
            revoke_all_res.attributes,
            [
                attr("action", "revoke_all"),
                attr("sender", ADDR1),
                attr("operator", ADDR2),
            ]
        );
    }

    #[test]
    fn test_transfer_nft() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());
        mint(deps.as_mut(), ADDR1, "1").unwrap();
        let transfer_nft_res = transfer_nft(deps.as_mut(), ADDR1, ADDR2).unwrap();

        assert_eq!(
            transfer_nft_res.attributes,
            [
                attr("action", "transfer_nft"),
                attr("sender", ADDR1),
                attr("recipient", ADDR2),
                attr("token_id", "1"),
            ]
        );

        // ADDR1 is unauthorized
        let transfer_err = transfer_nft(deps.as_mut(), ADDR1, ADDR2).unwrap_err();
        assert_eq!(transfer_err, ContractError::Unauthorized {})
    }

    #[test]
    fn test_approve_and_transfer() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());
        mint(deps.as_mut(), ADDR1, "1").unwrap();
        approve(deps.as_mut(), ADDR1, ADDR2).unwrap();
        transfer_nft(deps.as_mut(), ADDR2, ADDR2).unwrap();
    }

    #[test]
    fn test_send_nft() {
        let mut deps = mock_dependencies();
        let contract_addr = mock_env().contract.address;

        init(deps.as_mut());
        mint(deps.as_mut(), ADDR1, "1").unwrap();

        approve(deps.as_mut(), ADDR1, contract_addr.as_ref()).unwrap();

        let send_nft_msg = ExecuteMsg::<Extension, Empty>::SendNft {
            contract: contract_addr.to_string(),
            token_id: "1".to_string(),
            msg: to_binary(&ExecuteMsg::TransferNft::<Extension, Empty> {
                recipient: ADDR2.to_string(),
                token_id: "1".to_string(),
            })
                .unwrap(),
        };

        execute::<Extension, Empty, Empty>(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            send_nft_msg,
        )
            .unwrap();
    }

    #[test]
    fn test_burn() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());
        mint(deps.as_mut(), ADDR1, "1").unwrap();
        approve(deps.as_mut(), ADDR1, ADDR2).unwrap();
        let burn_msg = ExecuteMsg::Burn {
            token_id: "1".to_string(),
        };

        execute::<Extension, Empty, Empty>(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            burn_msg.clone(),
        )
            .unwrap();
        // Cannot burn same nft again
        execute::<Extension, Empty, Empty>(deps.as_mut(), mock_env(), mock_info(ADDR1, &[]), burn_msg).unwrap_err();
    }
}
