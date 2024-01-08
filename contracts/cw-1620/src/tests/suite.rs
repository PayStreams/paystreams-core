use anyhow::Result as AnyResult;
use cosmwasm_std::{Addr, Coin, StdResult, Timestamp};
use cw_asset::{Asset, AssetInfo};
use cw_multi_test::{App, AppResponse, ContractWrapper, Executor};
use wynd_utils::Curve;

use crate::{msg::StreamsResponse, state::StreamType};
pub fn store_streaming_contract(app: &mut App) -> u64 {
    let contract = Box::new(
        ContractWrapper::new_with_empty(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ), // .with_reply(crate::contract::reply)
           // .with_migrate(crate::contract::migrate),
    );

    app.store_code(contract)
}

fn store_cw20(app: &mut App) -> u64 {
    let contract = Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));

    app.store_code(contract)
}

#[derive(Debug)]
pub struct SuiteBuilder {
    funds: Vec<(Addr, Vec<Coin>)>,
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self { funds: vec![] }
    }

    pub fn with_funds(mut self, addr: &str, funds: &[Coin]) -> Self {
        self.funds.push((Addr::unchecked(addr), funds.into()));
        self
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");
        let funder: Addr = Addr::unchecked("funder");

        let _cw20_id = store_cw20(&mut app);
        let _id = store_streaming_contract(&mut app);

        let msg = crate::msg::InstantiateMsg { count: 0 };

        let paystreams_addr = app
            .instantiate_contract(
                _id,
                owner.clone(),
                &msg,
                &[],
                "Paystreams",
                Some(owner.to_string()),
            )
            .unwrap();

        let funds = self.funds;
        app.init_modules(|router, _, storage| -> AnyResult<()> {
            for (addr, coin) in funds {
                router.bank.init_balance(storage, &addr, coin)?;
            }
            Ok(())
        })
        .unwrap();

        Suite {
            owner: owner.to_string(),
            funder: funder.to_string(),
            app,
            paystreams_addr: paystreams_addr,
        }
    }
}

pub struct Suite {
    pub owner: String,
    pub funder: String,
    app: App,
    pub paystreams_addr: Addr,
}

impl Suite {
    #[allow(unused)]
    pub fn next_block(&mut self, time: u64) {
        self.app.update_block(|block| {
            block.time = block.time.plus_seconds(time);
            block.height += 1
        });
    }

    /// update block's time to simulate passage of time
    pub fn update_time(&mut self, time_update: u64) {
        self.app
            .update_block(|block: &mut cosmwasm_std::BlockInfo| {
                block.time = block.time.plus_seconds(time_update);
                block.height += time_update / 5;
            })
    }

    pub fn query_balance(&self, user: &str, denom: &str) -> AnyResult<u128> {
        Ok(self.app.wrap().query_balance(user, denom)?.amount.u128())
    }
    pub fn get_time_as_timestamp(&self) -> Timestamp {
        self.app.block_info().time
    }
    pub fn create_stream(
        &mut self,
        sender: Addr,
        recipient: Addr,
        deposit: u128,
        token_addr: &str,
        start_time: u64,
        stop_time: u64,
        funds: &[Coin],
        stream_type: Option<StreamType>,
        curve: Option<Curve>,
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::CreateStream {
            recipient: recipient.to_string(),
            asset: Asset {
                amount: deposit.into(),
                info: AssetInfo::Native(token_addr.to_string()),
            },
            start_time: start_time,
            stop_time: stop_time,
            stream_type: stream_type,
            curve: curve,
        };

        self.app
            .execute_contract(sender, self.paystreams_addr.clone(), &msg, funds)
    }

    pub fn withdraw_from_stream(
        &mut self,
        recipient: Addr,
        amount: u128,
        denom: &str,
        stream_idx: Option<u64>,
    ) -> AnyResult<AppResponse> {
        let msg = crate::msg::ExecuteMsg::ClaimFromStream {
            recipient: recipient.to_string(),
            amount: amount.into(),
            denom: denom.to_string(),
            stream_idx: stream_idx,
        };

        self.app
            .execute_contract(recipient, self.paystreams_addr.clone(), &msg, &[])
    }

    pub fn query_stream_count(&mut self) -> u64 {
        let msg = crate::msg::QueryMsg::StreamCount {};
        let count: crate::msg::CountResponse = self
            .app
            .wrap()
            .query_wasm_smart(&self.paystreams_addr, &msg)
            .unwrap();
        count.count as u64
    }

    pub fn query_streams_by_payee(&mut self, payee: Addr) -> StdResult<StreamsResponse> {
        let msg = crate::msg::QueryMsg::StreamsByRecipient {
            payee: payee.to_string(),
            reverse: None,
            limit: None,
        };
        let streams: StreamsResponse = self
            .app
            .wrap()
            .query_wasm_smart(&self.paystreams_addr, &msg)?;

        Ok(streams)
    }

    pub fn query_streams_by_sender(&mut self, sender: Addr) -> StdResult<StreamsResponse> {
        let msg = crate::msg::QueryMsg::StreamsBySender {
            sender: sender.to_string(),
            reverse: None,
            limit: None,
        };
        let streams: StreamsResponse = self
            .app
            .wrap()
            .query_wasm_smart(&self.paystreams_addr, &msg)?;

        Ok(streams)
    }
    #[allow(unused)]
    pub fn query_stream_by_payer_and_payee(
        &mut self,
        payer: Addr,
        payee: Addr,
    ) -> StdResult<StreamsResponse> {
        let msg = crate::msg::QueryMsg::LookupStream {
            payer: payer.to_string(),
            payee: payee.to_string(),
        };
        let streams: StreamsResponse = self
            .app
            .wrap()
            .query_wasm_smart(&self.paystreams_addr, &msg)?;

        Ok(streams)
    }

    pub fn query_stream_by_index(&mut self, index: u64) -> StdResult<StreamsResponse> {
        let msg = crate::msg::QueryMsg::StreamsByIndex { index: index };
        let streams: StreamsResponse = self
            .app
            .wrap()
            .query_wasm_smart(&self.paystreams_addr, &msg)?;

        Ok(streams)
    }

    pub fn query_stream_claimable_amount(&mut self, index: u64) -> StdResult<u128> {
        let msg = crate::msg::QueryMsg::StreamClaimableAmount { index: index };
        let claimable_amt: crate::msg::StreamClaimableAmtResponse = self
            .app
            .wrap()
            .query_wasm_smart(&self.paystreams_addr, &msg)?;

        Ok(claimable_amt.amount_available.u128())
    }
}
