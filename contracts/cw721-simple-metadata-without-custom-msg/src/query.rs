use cosmwasm_std::{to_binary, Binary, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub fn handle_custom_query_msg(value: String) -> StdResult<Binary> {
    to_binary(&HelloResponse { msg: value })
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct HelloResponse {
    pub msg: String,
}
