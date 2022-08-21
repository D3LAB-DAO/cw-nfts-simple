use std::error::Error;
use std::fmt::{Debug};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{set_contract_info, set_minter};
use crate::{execute, query};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw721::{ContractInfoResponse, CustomMsg};
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
pub fn execute<T, M, C, E>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<T, M>,
) -> Result<Response<C>, ContractError<E>>
    where
        T: Serialize + DeserializeOwned + Clone,
    // Custom message for extension E have to implement traits same as T
        M: Serialize + DeserializeOwned + Clone,
        E: Debug + PartialEq + Error,
        C: CustomMsg,
{
    match msg {
        ExecuteMsg::Mint(msg) => execute::mint::<T, C, E>(deps, env, info, msg),
        ExecuteMsg::Approve {
            spender,
            token_id,
            expires,
        } => execute::approve::<T, C, E>(deps, env, info, spender, token_id, expires),
        ExecuteMsg::Revoke { spender, token_id } => {
            execute::revoke::<T, C, E>(deps, env, info, spender, token_id)
        }
        ExecuteMsg::ApproveAll { operator, expires } => {
            execute::approve_all::<C, E>(deps, env, info, operator, expires)
        }
        ExecuteMsg::RevokeAll { operator } => execute::revoke_all::<C, E>(deps, env, info, operator),
        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => execute::transfer_nft::<T, C, E>(deps, env, info, recipient, token_id),
        ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => execute::send_nft::<T, C, E>(deps, env, info, contract, token_id, msg),
        ExecuteMsg::Burn { token_id } => execute::burn::<T, C, E>(deps, env, info, token_id),
        ExecuteMsg::Extension { msg: _ } => Ok(Response::new()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query<T, Q>(deps: Deps, env: Env, msg: QueryMsg<Q>) -> StdResult<Binary>
    where
        T: Serialize + DeserializeOwned + Clone,
        Q: Serialize + DeserializeOwned + Clone,
{
    match msg {
        QueryMsg::Minter {} => query::minter(deps),
        QueryMsg::ContractInfo {} => query::contract_info(deps),
        QueryMsg::NftInfo { token_id } => query::nft_info::<T>(deps, token_id),
        QueryMsg::OwnerOf {
            token_id,
            include_expired,
        } => query::owner_of::<T>(deps, env, token_id, include_expired.unwrap_or(false)),
        QueryMsg::AllNftInfo {
            token_id,
            include_expired,
        } => query::all_nft_info::<T>(deps, env, token_id, include_expired.unwrap_or(false)),
        QueryMsg::AllOperators {
            owner,
            include_expired,
            start_after,
            limit,
        } => query::operators(
            deps,
            env,
            owner,
            include_expired.unwrap_or(false),
            start_after,
            limit,
        ),
        QueryMsg::NumTokens {} => query::num_tokens(deps),
        QueryMsg::AllTokens { start_after, limit } => {
            query::all_tokens::<T>(deps, start_after, limit)
        }
        QueryMsg::Approval {
            token_id,
            spender,
            include_expired,
        } => query::approval::<T>(
            deps,
            env,
            token_id,
            spender,
            include_expired.unwrap_or(false),
        ),
        QueryMsg::Approvals {
            token_id,
            include_expired,
        } => query::approvals::<T>(deps, env, token_id, include_expired.unwrap_or(false)),
        QueryMsg::Tokens {
            owner,
            start_after,
            limit,
        } => query::tokens::<T>(deps, owner, start_after, limit),
        QueryMsg::Extension { msg: _ } => Ok(Binary::default()),
    }
}

#[cfg(test)]
pub mod contract_tests {
    use crate::contract::{execute, instantiate, query};
    use crate::error::{ContractError, CustomError};
    use crate::msg::{ExecuteMsg, InstantiateMsg, MintMsg, MinterResponse, QueryMsg};
    use crate::state::{get_tokens, TokenInfo};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, from_binary, to_binary, DepsMut, Empty, Response};
    use cw721::{
        AllNftInfoResponse, Approval, ApprovalResponse, ApprovalsResponse, ContractInfoResponse,
        Expiration, NftInfoResponse, NumTokensResponse, OperatorsResponse, OwnerOfResponse,
        TokensResponse,
    };

    const ADDR1: &str = "juno18zfp9u7zxg3gel4r3txa2jqxme7jkw7d972flm";
    const ADDR2: &str = "osmo18zfp9u7zxg3gel4r3txa2jqxme7jkw7dmh6zw4";

    type Extension = Option<Empty>;

    fn init(deps: DepsMut) {
        instantiate(
            deps,
            mock_env(),
            mock_info(ADDR1, &[]),
            InstantiateMsg {
                name: "cw721-contract".to_string(),
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

        execute::<Extension, Empty, Empty, CustomError>(
            deps,
            mock_env(),
            mock_info(ADDR1, &[]),
            execute_mint_msg,
        )
    }

    fn approve(deps: DepsMut, sender: &str, spender: &str) -> Result<Response, ContractError> {
        let valid_approve_msg = ExecuteMsg::Approve {
            spender: spender.to_string(),
            token_id: "1".to_string(),
            expires: Some(Expiration::AtHeight(50000)),
        };

        execute::<Extension, Empty, Empty, CustomError>(
            deps,
            mock_env(),
            mock_info(sender, &[]),
            valid_approve_msg,
        )
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

        execute::<Extension, Empty, Empty, CustomError>(
            deps,
            mock_env(),
            mock_info(sender, &[]),
            transfer_nft_msg,
        )
    }

    #[test]
    fn test_execute_mint() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());
        let res = mint(deps.as_mut(), ADDR1, "1").unwrap();
        mint(deps.as_mut(), ADDR1, "2").unwrap();

        let token_1: TokenInfo<Extension> = get_tokens().load(&deps.storage, "1").unwrap();

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
        );

        let num_tokens_query_msg = QueryMsg::<Empty>::NumTokens {};
        let num_tokens_query_res: NumTokensResponse = from_binary(
            &query::<Extension, Empty>(deps.as_ref(), mock_env(), num_tokens_query_msg).unwrap(),
        )
            .unwrap();
        assert_eq!(num_tokens_query_res, NumTokensResponse { count: 2 });
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

        let query_approval_msg = QueryMsg::<Empty>::Approval {
            token_id: "1".to_string(),
            spender: ADDR2.to_string(),
            include_expired: None,
        };

        let query_approval_res: ApprovalResponse = from_binary(
            &query::<Extension, Empty>(deps.as_ref(), mock_env(), query_approval_msg).unwrap(),
        )
            .unwrap();
        assert_eq!(
            query_approval_res,
            ApprovalResponse {
                approval: Approval {
                    spender: ADDR2.to_string(),
                    expires: Expiration::AtHeight(50000),
                }
            }
        );

        let query_approvals_msg = QueryMsg::<Empty>::Approvals {
            token_id: "1".to_string(),
            include_expired: Some(true),
        };

        let approvals_res: ApprovalsResponse = from_binary(
            &query::<Extension, Empty>(deps.as_ref(), mock_env(), query_approvals_msg).unwrap(),
        )
            .unwrap();
        assert_eq!(
            approvals_res,
            ApprovalsResponse {
                approvals: vec![Approval {
                    spender: ADDR2.to_string(),
                    expires: Expiration::AtHeight(50000),
                }]
            }
        );

        let expired_approve_msg = ExecuteMsg::<Extension, Empty>::Approve {
            spender: ADDR2.to_string(),
            token_id: "1".to_string(),
            expires: Some(Expiration::AtHeight(100)),
        };

        let invalid_res = execute::<Extension, Empty, Empty, CustomError>(
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

        let revoke_res = execute::<Extension, Empty, Empty, CustomError>(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            revoke_msg,
        )
            .unwrap();
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

        let approve_all_res = execute::<Extension, Empty, Empty, CustomError>(
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

        let all_operators_query_msg = QueryMsg::<Empty>::AllOperators {
            owner: ADDR1.to_string(),
            include_expired: None,
            start_after: None,
            limit: None,
        };

        let owner_of_res: OperatorsResponse = from_binary(
            &query::<Extension, Empty>(deps.as_ref(), mock_env(), all_operators_query_msg).unwrap(),
        )
            .unwrap();
        assert_eq!(
            owner_of_res,
            OperatorsResponse {
                operators: vec![Approval {
                    spender: ADDR2.to_string(),
                    expires: Expiration::AtHeight(50000),
                }]
            }
        );

        let expired_approve_all_msg = ExecuteMsg::<Extension, Empty>::ApproveAll {
            operator: ADDR2.to_string(),
            expires: Some(Expiration::AtHeight(10)),
        };

        let expired_approve_all_res = execute::<Extension, Empty, Empty, CustomError>(
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

        let revoke_all_res = execute::<Extension, Empty, Empty, CustomError>(
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

        let owner_of_query_msg = QueryMsg::<Empty>::OwnerOf {
            token_id: "1".to_string(),
            include_expired: None,
        };

        let owner_of_res: OwnerOfResponse = from_binary(
            &query::<Extension, Empty>(deps.as_ref(), mock_env(), owner_of_query_msg).unwrap(),
        )
            .unwrap();
        assert_eq!(
            owner_of_res,
            OwnerOfResponse {
                owner: ADDR2.to_string(),
                approvals: vec![],
            }
        );

        // ADDR1 is unauthorized
        let transfer_err = transfer_nft(deps.as_mut(), ADDR1, ADDR2).unwrap_err();
        assert_eq!(transfer_err, ContractError::Unauthorized {});

        mint(deps.as_mut(), ADDR2, "2").unwrap();

        let tokens_query_msg = QueryMsg::Tokens {
            owner: ADDR2.to_string(),
            start_after: None,
            limit: None,
        };

        let tokens_query_res: TokensResponse = from_binary(
            &query::<Extension, Empty>(deps.as_ref(), mock_env(), tokens_query_msg).unwrap(),
        )
            .unwrap();

        assert_eq!(
            tokens_query_res,
            TokensResponse {
                tokens: vec!["1".to_string(), "2".to_string()]
            }
        );
    }

    #[test]
    fn test_approve_and_transfer() {
        let mut deps = mock_dependencies();

        init(deps.as_mut());
        mint(deps.as_mut(), ADDR1, "1").unwrap();
        approve(deps.as_mut(), ADDR1, ADDR2).unwrap();
        transfer_nft(deps.as_mut(), ADDR2, ADDR2).unwrap();

        let all_tokens_query_msg = QueryMsg::AllTokens {
            start_after: None,
            limit: None,
        };

        let all_tokens_query_res: TokensResponse = from_binary(
            &query::<Extension, Empty>(deps.as_ref(), mock_env(), all_tokens_query_msg).unwrap(),
        )
            .unwrap();
        assert_eq!(
            all_tokens_query_res,
            TokensResponse {
                tokens: vec!["1".to_string()]
            }
        )
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

        execute::<Extension, Empty, Empty, CustomError>(
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

        execute::<Extension, Empty, Empty, CustomError>(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            burn_msg.clone(),
        )
            .unwrap();
        // Cannot burn same nft again
        execute::<Extension, Empty, Empty, CustomError>(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            burn_msg,
        )
            .unwrap_err();
    }

    #[test]
    fn query_contract_info() {
        let mut deps = mock_dependencies();
        init(deps.as_mut());
        mint(deps.as_mut(), ADDR1, "1").unwrap();
        mint(deps.as_mut(), ADDR1, "2").unwrap();

        let contract_info_query_msg = QueryMsg::<Empty>::ContractInfo {};
        let contract_info_query_res: ContractInfoResponse = from_binary(
            &query::<Extension, Empty>(deps.as_ref(), mock_env(), contract_info_query_msg).unwrap(),
        )
            .unwrap();
        assert_eq!(
            contract_info_query_res,
            ContractInfoResponse {
                name: "cw721-contract".to_string(),
                symbol: "cw721".to_string(),
            }
        );

        let nft_info_query_msg = QueryMsg::<Empty>::NftInfo {
            token_id: "1".to_string(),
        };

        let nft_info_query_res: NftInfoResponse<Extension> = from_binary(
            &query::<Extension, Empty>(deps.as_ref(), mock_env(), nft_info_query_msg).unwrap(),
        )
            .unwrap();
        assert_eq!(
            nft_info_query_res,
            NftInfoResponse {
                token_uri: None,
                extension: None,
            }
        );

        approve(deps.as_mut(), ADDR1, ADDR2).unwrap();

        let all_nft_info_query_msg = QueryMsg::<Empty>::AllNftInfo {
            token_id: "1".to_string(),
            include_expired: Some(true),
        };

        let all_nft_info_query_res: AllNftInfoResponse<Extension> = from_binary(
            &query::<Extension, Empty>(deps.as_ref(), mock_env(), all_nft_info_query_msg).unwrap(),
        )
            .unwrap();

        assert_eq!(
            all_nft_info_query_res,
            AllNftInfoResponse {
                access: OwnerOfResponse {
                    owner: ADDR1.to_string(),
                    approvals: vec![Approval {
                        spender: ADDR2.to_string(),
                        expires: Expiration::AtHeight(50000),
                    }],
                },
                info: NftInfoResponse {
                    token_uri: None,
                    extension: None,
                },
            }
        )
    }

    #[test]
    fn test_query_minter() {
        let mut deps = mock_dependencies();
        init(deps.as_mut());

        let minter_query_msg = QueryMsg::<Empty>::Minter {};

        let minter_query_res: MinterResponse = from_binary(
            &query::<Extension, Empty>(deps.as_ref(), mock_env(), minter_query_msg).unwrap(),
        )
            .unwrap();
        assert_eq!(
            minter_query_res,
            MinterResponse {
                minter: ADDR1.to_string()
            }
        );
    }
}
