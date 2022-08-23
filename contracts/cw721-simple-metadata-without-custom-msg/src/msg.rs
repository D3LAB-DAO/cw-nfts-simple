use cosmwasm_std::Empty;
use crate::contract::Extension;
use cw721_base::msg::{ExecuteMsg as Cw721ExecuteMsg, QueryMsg as Cw721QueryMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    DefaultCw721ExecuteMsg(Box<Cw721ExecuteMsg<Extension, Empty>>),
    ValidHello {},
    InvalidHello {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Cw721QueryMsg(Cw721QueryMsg<Empty>),
    HelloQuery { value: String },
}
