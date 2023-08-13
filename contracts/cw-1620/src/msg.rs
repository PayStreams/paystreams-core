use crate::state::PaymentStream;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Timestamp, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub count: i32,
}

#[cw_serde]

pub enum ExecuteMsg {
    CreateStream {
        recipient: String,
        deposit: Uint128,
        token_addr: String,
        start_time: Timestamp,
        stop_time: Timestamp,
    },
    WithdrawFromStream {
        recipient: String,
        amount: Uint128,
        denom: String,
        stream_idx: Option<u64>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(LookupStreamResponse)]
    LookupStream { payer: String, payee: String },
    #[returns(CountResponse)]
    StreamCount {},
    #[returns(StreamsResponse)]
    StreamsByPayee {
        payee: String,
        reverse: Option<bool>,
        limit: Option<usize>,
    },
    #[returns(StreamsResponse)]
    StreamsBySender {
        sender: String,
        reverse: Option<bool>,
        limit: Option<usize>,
    },
    #[returns(StreamsResponse)]
    StreamsByIndex { index: u64 },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct CountResponse {
    pub count: i32,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct LookupStreamResponse {
    pub stream: PaymentStream,
}

// A generic enough response which returns a Vec of PaymentStreams, may be for a payer or a payee
#[cw_serde]
pub struct StreamsResponse {
    pub streams: Vec<PaymentStream>,
}
