use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;
use wynd_utils::CurveError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("{0}")]
    Curve(#[from] CurveError),

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

    #[error("Attempted to divide by zero")]
    DivisionByZero {},

    #[error("Time delta is not set up properly {start_time}, {stop_time}")]
    DeltaIssue { start_time: u64, stop_time: u64 },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
