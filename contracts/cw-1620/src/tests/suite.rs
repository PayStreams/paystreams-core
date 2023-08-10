use anyhow::Result as AnyResult;
use cosmwasm_std::{Addr, Coin};
use cw_multi_test::{App, ContractWrapper};
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

        let cw20_code_id = store_cw20(&mut app);
        let paystreams_code_id = store_streaming_contract(&mut app);

        let init_msg = crate::msg::InstantiateMsg { count: 1 };

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
            paystreams_addr: Addr::unchecked("paystreams"),
        }
    }
}

pub struct Suite {
    pub owner: String,
    pub funder: String,
    app: App,
    pub paystreams_addr: Addr,
}
