use cosmwasm_std::StdError;
use std::error::Error;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError<E = CustomError>
where
    E: Error + Debug + PartialEq,
{
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("token_id already claimed")]
    Claimed {},

    #[error("Cannot set approval that is already expired")]
    Expired {},

    #[error("Approval not found for: {spender}")]
    ApprovalNotFound { spender: String },

    #[error("Saving contract info failed")]
    ContractInfoSaveError {},

    #[error("Saving minter failed")]
    MinterSaveError {},

    #[error("CustomError")]
    CustomError(E),
}

#[derive(Error, Debug, PartialEq)]
pub enum CustomError {
    #[error("CustomError")]
    CustomError {},
}
