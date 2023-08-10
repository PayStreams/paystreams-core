use cw_storage_plus::Index;
use cw_storage_plus::IndexList;
use cw_storage_plus::IndexedMap;
use cw_storage_plus::MultiIndex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use cw_storage_plus::Map;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// A PaymentStream is a State Object which contains the details for a Payment Stream between two parties
// Parties in this case being recipient and sender addresses
// The Stream stores information which can be payer-defined such as the rate of payment per second.
pub struct PaymentStream {
    pub deposit: Uint128,
    pub rate_per_second: Uint128,
    pub remaining_balance: Uint128,
    pub stop_time: u64,
    pub start_time: u64,
    pub recipient: Addr,
    pub sender: Addr,
    pub token_addr: Addr,
    pub is_entity: bool,
}



pub const STATE: Item<State> = Item::new("state");
pub const STREAMS: Map<(&Addr, &Addr), PaymentStream> = Map::new("streams");
// Secondary Indexes for our STREAMS map. 
// This will enable us to query the map by sender or recipient address instead of needing both 
pub struct TokenIndexes<'a> {
    pub sender: MultiIndex<'a, String, PaymentStream, String>,
    pub recipient: MultiIndex<'a, String, PaymentStream, String>,
}
// Setup indexes
impl<'a> IndexList<PaymentStream> for TokenIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<PaymentStream>> + '_> {
        let v: Vec<&dyn Index<PaymentStream>> = vec![&self.sender, &self.recipient];
        Box::new(v.into_iter())
    }
}

pub fn payment_streams<'a>() -> IndexedMap<'a, &'a str, PaymentStream, TokenIndexes<'a>> {
    let indexes = TokenIndexes {
        sender: MultiIndex::new(
            |_pk: &[u8], d: &PaymentStream| d.sender.clone().to_string(),
            "vesting_contracts",
            "vesting_contracts__instantiator",
        ),
        recipient: MultiIndex::new(
            |_pk: &[u8], d: &PaymentStream| d.recipient.clone().to_string(),
            "vesting_contracts",
            "vesting_contracts__recipient",
        ),
    };
    IndexedMap::new("vesting_contracts", indexes)
}