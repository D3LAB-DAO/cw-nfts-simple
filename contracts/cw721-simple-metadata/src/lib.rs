#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult};
use cw721_simple::contract::{
    execute as cw721_execute, instantiate as cw721_instantiate, query as cw721_query,
};
use cw721_simple::error::ContractError;
use cw721_simple::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MetaMessage {
    Hello {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

// see: https://docs.opensea.io/docs/metadata-standards
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
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
pub type MetaMsgExtension = MetaMessage;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw721_instantiate(deps, env, info, msg)
}

fn handle_custom_msg(msg: MetaMsgExtension) -> Result<Response, ContractError> {
    match msg {
        MetaMessage::Hello {} => Ok(Response::new().add_attribute("custom_msg", "hello")),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<Extension, MetaMsgExtension>,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Extension { msg } => handle_custom_msg(msg),
        _ => cw721_execute(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    cw721_query::<Extension, Empty>(deps, env, msg)
}

#[cfg(test)]
pub mod test_contract {
    use crate::execute;
    use crate::{instantiate, query, Extension, MetaMsgExtension, Metadata, Trait};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, from_binary, DepsMut, Response};
    use cw721::{NftInfoResponse, OwnerOfResponse};
    use cw721_simple::error::ContractError;
    use cw721_simple::msg::{ExecuteMsg, InstantiateMsg, MintMsg, QueryMsg};

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

    fn mint(deps: DepsMut, owner: &str, token_id: &str) -> Result<Response, ContractError> {
        let execute_mint_msg =
            ExecuteMsg::<Extension, MetaMsgExtension>::Mint(MintMsg::<Extension> {
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

        let custom_execute_msg = ExecuteMsg::Extension {
            msg: MetaMsgExtension::Hello {},
        };
        let custom_execute_res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR1, &[]),
            custom_execute_msg,
        )
        .unwrap();

        assert_eq!(custom_execute_res.attributes, [attr("custom_msg", "hello")]);
    }
}
