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
}

pub const STATE: Item<ConfigState> = Item::new("state");
// TODO: Make this a Vec of streams and update the logic on create to simply push a new stream to the vec, and on withdraw, unless an index is provided, attempt to withdraw from all. If it is, search the vec and use idx to find it
pub const STREAMS: Map<(&Addr, &Addr), PaymentStream> = Map::new("streams");
// Extra State Item to store the index we will use to base a new stream's index off of
pub const LAST_STREAM_IDX: Item<u64> = Item::new("last_stream_idx");
// Secondary Indexes for our STREAMS map.
// This will enable us to query the map by sender or recipient address instead of needing both
pub struct TokenIndexes<'a> {
    pub sender: MultiIndex<'a, String, PaymentStream, String>,
    pub recipient: MultiIndex<'a, String, PaymentStream, String>,
    pub by_index: MultiIndex<'a, String, PaymentStream, u64>,
}
// Setup indexes
impl<'a> IndexList<PaymentStream> for TokenIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<PaymentStream>> + '_> {
        let v: Vec<&dyn Index<PaymentStream>> = vec![&self.sender, &self.recipient, &self.by_index];
        Box::new(v.into_iter())
    }
}

pub fn payment_streams<'a>() -> IndexedMap<'a, &'a str, PaymentStream, TokenIndexes<'a>> {
    let indexes = TokenIndexes {
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
            "paystreams__index",
        ),
    };
    IndexedMap::new("paystream", indexes)
}
