use crate::state::PaymentStream;
use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateStream {
        recipient: String,
        deposit: Uint128,
        token_addr: String,
        start_time: u64,
        stop_time: u64,
    },
    WithdrawFromStream {
        recipient: String,
        amount: Uint128,
        denom: String,
        stream_idx: Option<u64>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    LookupStream {
        payer: String,
        payee: String,
    },
    StreamCount {},
    StreamsByPayee {
        payee: String,
        reverse: Option<bool>,
        limit: Option<usize>,
    },
    StreamsBySender {
        sender: String,
        reverse: Option<bool>,
        limit: Option<usize>,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i32,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LookupStreamResponse {
    pub stream: PaymentStream,
}

// A generic enough response which returns a Vec of PaymentStreams, may be for a payer or a payee
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StreamsResponse {
    pub streams: Vec<PaymentStream>,
}
