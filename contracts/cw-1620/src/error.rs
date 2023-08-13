use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid Amount")]
    InvalidAmount {},

    #[error("Not Enough Balance Available to Withdraw")]
    NotEnoughAvailableBalance {},

    #[error("Not Enough Funds Available to Withdraw")]
    NotEnoughAvailableFunds {},

    #[error("Could not find a stream with provided index or address")]
    StreamNotFound {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
