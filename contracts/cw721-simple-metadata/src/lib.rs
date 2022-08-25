#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw721_base::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cw721_simple_base::contract::{
    execute as cw721_execute, instantiate as cw721_instantiate, query as cw721_query,
};
use cw721_simple_base::error::ContractError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CustomExtensionMsg {
    ValidHello {},
    InvalidHello {},
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum CustomError {
    #[error("HelloError: {msg}")]
    HelloError { msg: String },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub enum CustomQuery {
    HelloQuery {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct HelloResponse {
    msg: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

// see: https://docs.opensea.io/docs/metadata-standards
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
}

pub type Extension = Option<Metadata>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw721_instantiate(deps, env, info, msg)
}

fn handle_custom_msg(msg: CustomExtensionMsg) -> Result<Response, ContractError<CustomError>> {
    match msg {
        CustomExtensionMsg::ValidHello {} => {
            Ok(Response::new().add_attribute("custom_msg", "hello"))
        }
        CustomExtensionMsg::InvalidHello {} => {
            Err(ContractError::CustomError(CustomError::HelloError {
                msg: "no_hello".to_string(),
            }))
        }
    }
}

fn handle_custom_query_msg(msg: CustomQuery) -> StdResult<Binary> {
    match msg {
        CustomQuery::HelloQuery {} => to_binary(&HelloResponse {
            msg: "custom_hello_query_response".to_string(),
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<Extension, CustomExtensionMsg>,
) -> Result<Response, ContractError<CustomError>> {
    match msg {
        ExecuteMsg::Extension { msg } => handle_custom_msg(msg),
        _ => cw721_execute(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg<CustomQuery>) -> StdResult<Binary> {
    match msg {
        QueryMsg::Extension { msg } => handle_custom_query_msg(msg),
        _ => cw721_query::<Extension, CustomQuery>(deps, env, msg),
    }
}

#[cfg(test)]
pub mod test_contract {
    use crate::{
        execute, instantiate, query, CustomError, CustomExtensionMsg, Extension, Metadata, Trait,
    };
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, DepsMut, Response};
    use cw721::{NftInfoResponse, OwnerOfResponse};
    use cw721_base::msg::{ExecuteMsg, InstantiateMsg, MintMsg, QueryMsg};
    use cw721_simple_base::error::ContractError;

    const ADDR1: &str = "juno18zfp9u7zxg3gel4r3txa2jqxme7jkw7d972flm";

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

    fn mint(
        deps: DepsMut,
        owner: &str,
        token_id: &str,
    ) -> Result<Response, ContractError<CustomError>> {
        let execute_mint_msg =
            ExecuteMsg::<Extension, CustomExtensionMsg>::Mint(MintMsg::<Extension> {
                token_id: token_id.to_string(),
                owner: owner.to_string(),
                token_uri: None,
                extension: Some(Metadata {
                    image: Some("image".to_string()),
                    image_data: Some("image_data".to_string()),
                    external_url: Some("external_url".to_string()),
                    description: Some("description".to_string()),
                    name: Some("name".to_string()),
                    attributes: Some(vec![Trait {
                        display_type: Some("display_type".to_string()),
                        trait_type: "trait_type".to_string(),
                        value: "value".to_string(),
                    }]),
                    background_color: Some("background_color".to_string()),
                    animation_url: Some("animation_url".to_string()),
                    youtube_url: Some("youtube_url".to_string()),
                }),
            });

        execute(deps, mock_env(), mock_info(ADDR1, &[]), execute_mint_msg)
    }

    #[test]
    fn test_mint() {
        let mut deps = mock_dependencies();
        init(deps.as_mut());
        mint(deps.as_mut(), ADDR1, "1").unwrap();

        let owner_of_query_msg = QueryMsg::OwnerOf {
            token_id: "1".to_string(),
            include_expired: Some(false),
        };

        let owner_of_res: OwnerOfResponse =
            from_binary(&query(deps.as_ref(), mock_env(), owner_of_query_msg).unwrap()).unwrap();
        assert_eq!(
            owner_of_res,
            OwnerOfResponse {
                owner: ADDR1.to_string(),
                approvals: vec![],
            }
        );

        let nft_info_query_msg = QueryMsg::NftInfo {
            token_id: "1".to_string(),
        };

        let nft_info_res: NftInfoResponse<Extension> =
            from_binary(&query(deps.as_ref(), mock_env(), nft_info_query_msg).unwrap()).unwrap();

        assert_eq!(
            nft_info_res,
            NftInfoResponse {
                token_uri: None,
                extension: Some(Metadata {
                    image: Some("image".to_string()),
                    image_data: Some("image_data".to_string()),
                    external_url: Some("external_url".to_string()),
                    description: Some("description".to_string()),
                    name: Some("name".to_string()),
                    attributes: Some(vec![Trait {
                        display_type: Some("display_type".to_string()),
                        trait_type: "trait_type".to_string(),
                        value: "value".to_string(),
                    }]),
                    background_color: Some("background_color".to_string()),
                    animation_url: Some("animation_url".to_string()),
                    youtube_url: Some("youtube_url".to_string()),
                }),
            }
        );
    }
}
