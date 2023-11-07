use crate::state::{PaymentStream, StreamType};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Timestamp, Uint128};
use cw_asset::{Asset, AssetInfo};
use wynd_utils::Curve;
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub count: i32,
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    CreateStream {
        recipient: String,
        asset: Asset,
        start_time: u64,
        stop_time: u64,
        stream_type: Option<StreamType>,
        curve: Option<Curve>,
    },
    ClaimFromStream {
        recipient: String,
        amount: Uint128,
        denom: String,
        stream_idx: Option<u64>,
    },
    CancelStream {
        stream_idx: u64,
    },
}

#[cw_serde]
pub enum Cw20HookMsg {
    CreateStream {
        recipient: String,
        start_time: u64,
        stop_time: u64,
        stream_type: Option<StreamType>,
        curve: Option<Curve>,
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
    StreamsByRecipient {
        payee: String,
        reverse: Option<bool>,
        limit: Option<Uint128>,
    },
    #[returns(StreamsResponse)]
    StreamsBySender {
        sender: String,
        reverse: Option<bool>,
        limit: Option<Uint128>,
    },
    #[returns(StreamsResponse)]
    StreamsByIndex { index: u64 },

    #[returns(StreamClaimableAmtResponse)]
    StreamClaimableAmount { index: u64 },
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

#[cw_serde]
pub struct StreamClaimableAmtResponse {
    pub amount_available: Uint128,
    pub stream: PaymentStream,
}
