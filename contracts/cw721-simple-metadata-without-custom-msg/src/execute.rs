use crate::error::ContractError;
use cosmwasm_std::Response;

pub fn valid_hello() -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("custom_msg", "valid_hello"))
}

pub fn invalid_hello() -> Result<Response, ContractError> {
    Err(ContractError::HelloError {
        msg: "invalid_hello".to_string(),
    })
}
