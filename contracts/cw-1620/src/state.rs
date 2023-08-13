use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal;
use cosmwasm_std::Timestamp;
// TODO: Consider using our own impl of Asset derived from WW and other implementations
use cw_storage_plus::Index;
use cw_storage_plus::IndexList;
use cw_storage_plus::IndexedMap;
use cw_storage_plus::MultiIndex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use white_whale::pool_network::asset::Asset;

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use cw_storage_plus::Map;
use wynd_utils::Curve;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigState {
    pub count: i32,
    pub owner: Addr,
    pub fee_asset: Option<Asset>,
    // TODO: Move this out into a fee system
    pub fees: Option<Vec<Decimal>>,
}
impl Default for ConfigState {
    fn default() -> Self {
        Self {
            count: 0,
            owner: Addr::unchecked(""),
            fee_asset: None,
            fees: None,
        }
    }
}

#[cw_serde]
/// All the different types of payment streams we can create
/// Not all types are supported as of yet but these are all the possible types
/// Note DynamicCurveBased can be used to create almost any type of payment curve and is for very advanced use cases
pub enum StreamType {
    Basic,
    LinearCurveBased,
    CliffCurveBased,
    DynamicCurveBased,
    ExponentialCurveBased,
    ExponentialCurveBasedWithCliff,
    TraditionalUnlockStepCurve,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// A PaymentStream is a State Object which contains the details for a Payment Stream between two parties
// Parties in this case being recipient and sender addresses
// The Stream stores information which can be payer-defined such as the rate of payment per second.
pub struct PaymentStream {
    pub stream_idx: u64,
    pub deposit: Uint128,
    pub rate_per_second: Uint128,
    pub remaining_balance: Uint128,
    pub stop_time: Timestamp,
    pub start_time: Timestamp,
    pub recipient: Addr,
    pub sender: Addr,
    pub token_addr: Addr,
    pub is_entity: bool,
    pub curve: Option<Curve>,
}

#[cw_serde]
pub struct StreamData {
    pub start_time: Timestamp,
    pub stop_time: Timestamp,
    pub stream_type: Option<StreamType>,
    pub curve: Option<Curve>,
}

pub const STATE: Item<ConfigState> = Item::new("state");
// TODO: Make this a Vec of streams and update the logic on create to simply push a new stream to the vec, and on withdraw, unless an index is provided, attempt to withdraw from all. If it is, search the vec and use idx to find it
pub const STREAMS: Map<(&Addr, &Addr), PaymentStream> = Map::new("streams");
// Extra State Item to store the index we will use to base a new stream's index off of
pub const LAST_STREAM_IDX: Item<u64> = Item::new("last_stream_idx");
// Secondary Indexes for our STREAMS map.
// This will enable us to query the map by sender or recipient address instead of needing both
pub struct StreamSecondaryIndexes<'a> {
    pub sender: MultiIndex<'a, String, PaymentStream, String>,
    pub recipient: MultiIndex<'a, String, PaymentStream, String>,
    pub by_index: MultiIndex<'a, String, PaymentStream, u64>,
}
// Setup indexes
impl<'a> IndexList<PaymentStream> for StreamSecondaryIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<PaymentStream>> + '_> {
        let v: Vec<&dyn Index<PaymentStream>> = vec![&self.sender, &self.recipient, &self.by_index];
        Box::new(v.into_iter())
    }
}

pub fn payment_streams<'a>() -> IndexedMap<'a, &'a str, PaymentStream, StreamSecondaryIndexes<'a>> {
    let indexes = StreamSecondaryIndexes {
        sender: MultiIndex::new(
            |_pk: &[u8], d: &PaymentStream| d.sender.clone().to_string(),
            "paystream",
            "paystream__sender",
        ),
        recipient: MultiIndex::new(
            |_pk: &[u8], d: &PaymentStream| d.recipient.clone().to_string(),
            "paystream",
            "paystream__recipient",
        ),
        by_index: MultiIndex::new(
            |_pk: &[u8], d: &PaymentStream| d.stream_idx.clone().to_string(),
            "paystream",
            "paystream__index",
        ),
    };
    IndexedMap::new("paystream", indexes)
}
