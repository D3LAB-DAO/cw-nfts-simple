use cw721_simple_base::error::ContractError as Cw721ContractError;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("HelloError: {msg}")]
    HelloError { msg: String },

    #[error("Cw721ContractError")]
    Cw721ContractError(Cw721ContractError),
}
