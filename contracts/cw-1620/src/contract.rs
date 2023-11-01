#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult, Timestamp, Uint128, Addr, from_binary, Empty,
};
use cw2::set_contract_version;
use cw_utils::may_pay;
use serde::de;
use wynd_utils::Curve;
use cw20::Cw20ReceiveMsg;

use crate::error::ContractError;
use crate::msg::{
    CountResponse, ExecuteMsg, InstantiateMsg, LookupStreamResponse, QueryMsg, StreamsResponse, Cw20HookMsg, StreamClaimableAmtResponse,
};
use crate::state::{
    payment_streams, ConfigState, PaymentStream, StreamData, StreamType, LAST_STREAM_IDX, STATE,
    STREAMS,
};
use cw_asset::{Asset, AssetInfo, AssetInfoBase, AssetInfoKey};


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-1620";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_LIMIT_FOR_QUERY: Uint128 = Uint128::new(10);
#[allow(unused)]
const DEFAULT_ORDER_FOR_QUERY: Order = Order::Ascending;
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = ConfigState {
        count: msg.count,
        owner: info.sender.clone(),
        fee_asset: None,
        fees: None,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    LAST_STREAM_IDX.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: Empty) -> Result<Response, ContractError> {
    

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::CreateStream {
            recipient,
            asset,
            start_time,
            stop_time,
            stream_type,
            curve,
        } => 
        
        {   
            match asset.info.clone() {
                AssetInfo::Native(denom) => {
                    let deposit_amount = may_pay(&info, &denom).unwrap();
                    if deposit_amount < asset.amount {
                        return Err(ContractError::NotEnoughAvailableFunds {});
                    }
                    try_create_stream(
                    deps,
                    info,
                    recipient,
                    asset.amount,
                    asset.info,
                    StreamData {
                        start_time: Timestamp::from_seconds(start_time),
                        stop_time: Timestamp::from_seconds(stop_time),
                        stream_type: stream_type,
                        curve: curve,
                    },
                )},
                _ => unimplemented!()
            }
                
            
        },
        ExecuteMsg::ClaimFromStream {
            recipient,
            amount,
            denom,
            stream_idx,
        } => claim_from_stream(deps, info, env, recipient, amount, denom, stream_idx),
        ExecuteMsg::CancelStream { stream_idx } => {
            // Load stream, verify sender is the sender of stream, and then delete stream
            let stream = payment_streams().load(deps.storage, &stream_idx.to_string())?;
            if stream.sender != info.sender {
                return Err(ContractError::Unauthorized {});
            }

            // Check it doesn't exceed available
            let available_bal_for_stream: Uint128 =
                avail_balance_of(stream.clone(), env).unwrap_or_else(|_| Uint128::zero());

            let mut messages: Vec<CosmosMsg> = vec![];
            let denom = match stream.token_addr {
                AssetInfo::Native(denom) => denom,
                _ => unimplemented!(),
            };
            if available_bal_for_stream > Uint128::zero() {
                // Pay the available to the receipient
                let payout_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
                    to_address: stream.recipient.to_string(),
                    amount: vec![Coin {
                        denom: denom.clone(),
                        amount: available_bal_for_stream,
                    }],
                });
                messages.push(payout_msg);
            }
            // Pay the remaining to the sender
            let payout_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
                to_address: stream.sender.to_string(),
                amount: vec![Coin { denom, amount: stream.remaining_balance.checked_sub(available_bal_for_stream)? }],
            });
            messages.push(payout_msg);

            // Return response with messages
            Ok(Response::new()
                .add_messages(messages)
                .add_attribute("method", "cancel_stream"))
            
        },
    }
}

// receive_cw20 routes a cw20 token to the proper handler in this case stake and unstake
fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let sender = deps.api.addr_validate(&cw20_msg.sender)?;

    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::CreateStream {recipient,
            start_time,
            stop_time,
            stream_type,
            curve} => {
            if cw20_msg.amount.is_zero() {
                return Err(ContractError::InvalidAmount {});
            }
            let asset = AssetInfo::Cw20(info.sender.clone());
            try_create_stream(
                deps,
                info,
                recipient,
                cw20_msg.amount,
                asset,
                StreamData {
                    start_time: Timestamp::from_seconds(start_time),
                    stop_time: Timestamp::from_seconds(stop_time),
                    stream_type: stream_type,
                    curve: curve,
                },
            )
        }
    }
}

// To create a stream we want verify a number of things before saving the stream info and starting some accrual process
// 1. The recipient is a valid address
// 2. The deposit is a valid amount
// 3. The first entry in funds is a native token and the amount is non zero
// 4. The start time is before the stop time
pub fn try_create_stream(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    deposit: Uint128,
    token_addr: AssetInfoBase<Addr>,
    stream_data: StreamData,
) -> Result<Response, ContractError> {
    let recipient = deps.api.addr_validate(&recipient)?;
    let start_time = stream_data.start_time.seconds();
    let stop_time = stream_data.stop_time.seconds();

    if stop_time <= start_time {
        return Err(ContractError::DeltaIssue { start_time: start_time, stop_time: stop_time });
    }

    // Get the time delta
    let duration = stop_time - start_time;    

    // Unless stream_type is provided we will assume it is StreamType::Basic
    let stream_type = stream_data.stream_type.unwrap_or(StreamType::Basic);

    let stream_idx = LAST_STREAM_IDX.load(deps.storage)? + 1;

    // Only stream type is supported right now
    match stream_type {
        StreamType::Basic => {
            // Calculate rate_per_second with error handling
            let rate_per_second = deposit.checked_div(Uint128::from(duration)).map_err(|_| ContractError::DivisionByZero {})?;
            let stream_data = PaymentStream {
                stream_idx,
                recipient: recipient.clone(),
                deposit,
                token_addr: token_addr,
                start_time: stream_data.start_time,
                stop_time: stream_data.stop_time,
                is_entity: false,
                rate_per_second,
                remaining_balance: deposit,
                sender: info.sender.clone(),
                curve: None,
            };
            STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
                state.count += 1;
                Ok(state)
            })?;
            payment_streams().save(
                deps.storage,
                stream_data.stream_idx.to_string().as_ref(),
                &stream_data,
            )?;
            payment_streams().load(deps.storage, &stream_data.stream_idx.to_string())?;
            STREAMS.save(deps.storage, (&recipient, &info.sender), &stream_data)?;
            LAST_STREAM_IDX.save(deps.storage, &stream_idx)?;
            println!("Stream created: {:?}", stream_data);
        }
        StreamType::LinearCurveBased => {
            // Verify the provided curve is valid, in this case we want to make sure its the right curve type and its monotonically increasing
            let curve = stream_data.curve.unwrap();
            curve.validate_monotonic_increasing()?;

            match curve.clone() {
                Curve::Constant { y } => {
                    // We can get rate per second in the case of a constant curve by dividing the deposit by the duration
                    let rate_per_second = y;
                    let stream_data = PaymentStream {
                        stream_idx,
                        recipient: recipient.clone(),
                        deposit,
                        token_addr,
                        start_time: stream_data.start_time,
                        stop_time: stream_data.stop_time,
                        is_entity: false,
                        rate_per_second,
                        remaining_balance: deposit,
                        sender: info.sender.clone(),
                        curve: Some(curve),
                    };
                    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
                        state.count += 1;
                        Ok(state)
                    })?;
                    payment_streams().save(
                        deps.storage,
                        stream_data.stream_idx.to_string().as_ref(),
                        &stream_data,
                    )?;
                    payment_streams().load(deps.storage, &stream_data.stream_idx.to_string())?;
                    STREAMS.save(deps.storage, (&recipient, &info.sender), &stream_data)?;
                    LAST_STREAM_IDX.save(deps.storage, &stream_idx)?;
                    println!("Stream created: {:?}", stream_data);
                }
                Curve::SaturatingLinear(s) => {
                    // We can get rate per second in the case of a constant curve by dividing the deposit by the duration
                    let rate_per_second = s.max_y - s.min_y;
                    let stream_data = PaymentStream {
                        stream_idx,
                        recipient: recipient.clone(),
                        deposit,
                        token_addr,
                        start_time: stream_data.start_time,
                        stop_time: stream_data.stop_time,
                        is_entity: false,
                        rate_per_second,
                        remaining_balance: deposit,
                        sender: info.sender.clone(),
                        curve: Some(curve),
                    };
                    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
                        state.count += 1;
                        Ok(state)
                    })?;
                    payment_streams().save(
                        deps.storage,
                        stream_data.stream_idx.to_string().as_ref(),
                        &stream_data,
                    )?;
                    payment_streams().load(deps.storage, &stream_data.stream_idx.to_string())?;
                    STREAMS.save(deps.storage, (&recipient, &info.sender), &stream_data)?;
                    LAST_STREAM_IDX.save(deps.storage, &stream_idx)?;
                    println!("Stream created: {:?}", stream_data);
                }
                _ => {
                    return Err(ContractError::Unauthorized {});
                }
            }
        }
        _ => return Err(ContractError::Unauthorized {}),
    }

    Ok(Response::new().add_attribute("method", "try_create_stream"))
}

pub fn claim_from_stream(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    recipient: String,
    amount: Uint128,
    denom: String,
    stream_idx: Option<u64>,
) -> Result<Response, ContractError> {
    let recipient = deps.api.addr_validate(&recipient)?;

    // Check amount is valid
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidAmount {});
    }
    // let stream = payment_streams().load(deps.storage, &stream_idx.to_string())?;
    let mut paystream: PaymentStream = if let Some(stream_idx) = stream_idx {
        payment_streams().load(deps.storage, &stream_idx.to_string())?
    } else {
        STREAMS.load(deps.storage, (&recipient, &info.sender))?
    };

    println!("Stream {:?}", paystream);

    // Check it doesn't exceed available
    let available_bal_for_stream: Uint128 =
        avail_balance_of(paystream.clone(), env).unwrap_or_else(|_| Uint128::zero());
    println!(
        "Amount {:?} Balance: {:?} Remaining from stream {:?}",
        amount, available_bal_for_stream, paystream.remaining_balance
    );

    if amount > available_bal_for_stream {
        return Err(ContractError::NotEnoughAvailableBalance {});
    }
    if amount > paystream.remaining_balance {
        return Err(ContractError::NotEnoughAvailableFunds {});
    }

    // Make the payout happen
    let payout_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![Coin { denom, amount }],
    });
    println!("Remaining balance {:?}", paystream.remaining_balance);
    paystream.remaining_balance = paystream.remaining_balance.checked_sub(amount)?;
    println!("Remaining balance {:?}", paystream.remaining_balance);

    STREAMS.save(deps.storage, (&recipient, &info.sender), &paystream)?;
    payment_streams().save(
        deps.storage,
        paystream.stream_idx.to_string().as_ref(),
        &paystream,
    )?;

    Ok(Response::new()
        .add_attribute("method", "try_withdraw_from_stream")
        .add_message(payout_msg))
}

pub fn avail_balance_of(stream: PaymentStream, env: Env) -> Result<Uint128, ContractError> {
    // Get the time delta
    let delta = delta(stream.clone(), env.clone())?;
    println!("Stream {:?}", stream);
    match stream.curve {
        Some(curve) => {
            match curve.clone() {
                Curve::SaturatingLinear(s) => {
                    let rec_amt =
                        stream.deposit.u128() - curve.value(env.block.time.seconds()).u128();
                    let rec_bal = Uint128::from(rec_amt);
                    let amount_available = stream.deposit.checked_sub(stream.remaining_balance)?;
                    let new_balance = rec_bal.checked_sub(amount_available)?;

                    // Calc this better

                    return Ok(new_balance);
                }
                _ => {
                    return Err(ContractError::Unauthorized {});
                }
            };
        }
        None => {
            // Use delta to get the balance that should be available
            let rec_bal = Uint128::from(delta).checked_mul(stream.rate_per_second)?;
            // println!("Delta: {:?}, Recipient Balance: {:?}", delta, rec_bal);

            if stream.deposit >= stream.remaining_balance {
                // println!("Deposit: {:?}, Remaining: {:?}", stream.deposit , stream.remaining_balance);
                let amount_available = stream.deposit.checked_sub(stream.remaining_balance)?;
                let new_balance = rec_bal.checked_sub(amount_available)?;
                // println!("New balance {:?}", new_balance);
                return Ok(new_balance);
            }
            Ok(Uint128::from(0u128))
        }
    }
}

pub fn delta(stream: PaymentStream, env: Env) -> StdResult<u64> {
    if env.block.time <= stream.start_time {
        return Ok(0);
    }
    if env.block.time < stream.stop_time {
        return Ok(env
            .block
            .time
            .minus_seconds(stream.start_time.seconds())
            .seconds());
    }
    let duration = stream
        .stop_time
        .seconds()
        .checked_sub(stream.start_time.seconds())
        .ok_or(ContractError::Unauthorized {})
        .unwrap();
    Ok(Timestamp::from_seconds(duration).seconds())
}

pub fn deltaOf(deps:Deps, env: Env,stream_id: u64) -> StdResult<u64> {
    // Get the stream from storage
    let stream = payment_streams().load(deps.storage, &stream_id.to_string())?;
    // If the stream hasn't started yet, return 0
    if env.block.time <= stream.start_time {
        return Ok(0);
    }
    // If the stream has started but not ended, return the delta
    if env.block.time < stream.stop_time {
        return Ok(env
            .block
            .time
            .minus_seconds(stream.start_time.seconds())
            .seconds());
    }
    // If the stream has ended, return the duration
    let duration = stream
        .stop_time
        .seconds()
        .checked_sub(stream.start_time.seconds())
        .unwrap();
    Ok(Timestamp::from_seconds(duration).seconds())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::LookupStream { payer, payee } => to_binary(&query_stream(deps, payer, payee)?),
        QueryMsg::StreamCount {} => to_binary(&query_stream_count(deps)?),
        QueryMsg::StreamsByPayee {
            payee,
            reverse,
            limit,
        } => {
            let order = match reverse {
                Some(false) | None => Order::Ascending,
                Some(true) => Order::Descending,
            };

            to_binary(&query_streams_by_payee(deps, payee, order, limit)?)
        }
        QueryMsg::StreamsBySender {
            sender,
            reverse,
            limit,
        } => {
            let order = match reverse {
                Some(false) | None => Order::Ascending,
                Some(true) => Order::Descending,
            };

            to_binary(&query_streams_by_sender(deps, sender, order, limit)?)
        }
        QueryMsg::StreamsByIndex { index } => to_binary(&query_stream_by_index(deps, index)?),
        QueryMsg::StreamClaimableAmount { index } => to_binary(&query_stream_amount_claimable(deps, env, index)?),
    }
}

fn query_stream_count(deps: Deps) -> StdResult<CountResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(CountResponse { count: state.count })
}

fn query_stream(deps: Deps, payer: String, payee: String) -> StdResult<LookupStreamResponse> {
    // For now, just validate what we got was a valid address
    let _vld_payer = deps.api.addr_validate(&payer)?;
    let _vld_payee = deps.api.addr_validate(&payee)?;
    let state = STREAMS.load(deps.storage, (&_vld_payer, &_vld_payee))?;
    Ok(LookupStreamResponse { stream: state })
}

pub fn query_streams_by_payee(
    deps: Deps,
    payee: String,
    order: Order,
    limit: Option<Uint128>,
) -> StdResult<StreamsResponse> {
    let _vld_payee = deps.api.addr_validate(&payee)?;

    let streams: Vec<PaymentStream> = payment_streams()
        .idx
        .recipient
        .prefix(payee)
        .range(deps.storage, None, None, order)
        .take(limit.unwrap_or(DEFAULT_LIMIT_FOR_QUERY).u128() as usize)
        .flat_map(|vc| Ok::<PaymentStream, ContractError>(vc?.1))
        .collect();

    Ok(StreamsResponse { streams })
}

pub fn query_streams_by_sender(
    deps: Deps,
    sender: String,
    order: Order,
    limit: Option<Uint128>,
) -> StdResult<StreamsResponse> {
    let _vld_sender = deps.api.addr_validate(&sender)?;

    let streams: Vec<PaymentStream> = payment_streams()
        .idx
        .sender
        .prefix(sender)
        .range(deps.storage, None, None, order)
        .take(limit.unwrap_or(DEFAULT_LIMIT_FOR_QUERY).u128() as usize)
        .flat_map(|vc| Ok::<PaymentStream, ContractError>(vc?.1))
        .collect();

    println!("Querying by sender {:?}", streams);

    Ok(StreamsResponse { streams })
}

pub fn query_stream_by_index(deps: Deps, stream_idx: u64) -> StdResult<StreamsResponse> {
    // Get 1 from payment_streams
    let stream = payment_streams().load(deps.storage, &stream_idx.to_string())?;
    Ok(StreamsResponse {
        streams: vec![stream],
    })
}

pub fn query_stream_amount_claimable(deps: Deps, env: Env,stream_idx: u64) -> StdResult<StreamClaimableAmtResponse> {
    // Get 1 from payment_streams
    let stream = payment_streams().load(deps.storage, &stream_idx.to_string())?;
    // Get the time delta
    let delta = delta(stream.clone(), env.clone())?;
    // Use delta to get the balance that should be available
    let rec_bal = Uint128::from(delta).checked_mul(stream.rate_per_second)?;
    // println!("Delta: {:?}, Recipient Balance: {:?}", delta, rec_bal);

    Ok(StreamClaimableAmtResponse {
        stream,
        amount_available: rec_bal,
    })
}

// TODO: Add more queries for streams information such as
// + how much is left to be received (all unclaimed funds ready to claim+all due funds until end of stream)
// + how much will be paid/available at a given time (all earnings from start->given time less any claimed funds
// + how much has been paid out (all claimed funds)

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};
    use cosmwasm_std::{BankMsg, Coin, CosmosMsg, SubMsg, Timestamp, Uint128};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 0 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::StreamCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(0, value.count);
    }

    #[test]
    fn can_create_stream() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 0 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::StreamCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(0, value.count);

        // Create a Stream between mock users
        let payer = mock_info("payer", &coins(1000, "axlusdc"));

        let payee = mock_info("payee", &[]);

        let env = mock_env();

        let stream_msg = ExecuteMsg::CreateStream {
            asset: Asset {
                amount: Uint128::new(100),
                info: AssetInfo::Native("axlusdc".to_string()),
            },            recipient: payee.sender.to_string(),
            start_time: env.block.time.seconds(),
            stop_time: env.block.time.plus_seconds(100).seconds(),
            stream_type: None,
            curve: None,
        };

        let _ = execute(deps.as_mut(), env.clone(), payer.clone(), stream_msg).unwrap();

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::StreamCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.count);
    }

    #[test]

    // Idea of this test is too ensure that when a payer creates a stream that their specified deposit amount is not more than the funds they provide to fund the stream
    fn can_not_create_with_deposit_more_than_provided_funds() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 0 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::StreamCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(0, value.count);

        // Create 'payer' user with 1000 USD in balance to act as the payer of the stream
        let payer = mock_info("payer", &coins(1000, "axlusdc"));

        // Create 'payee' user with 0 USD in balance to act as the receiver of funds from the stream
        let payee = mock_info("payee", &coins(0, "axlusdc"));

        let env = mock_env();

        // Attempt to create a stream with 1 more dollar than provided. Cheating the system out of a dollar.
        // Basically "Hey lil dude from across the street, lemme hold a dollar"
        let stream_msg = ExecuteMsg::CreateStream {
            asset: Asset {
                amount: Uint128::new(1001),
                info: AssetInfo::Native("axlusdc".to_string()),
            },
            recipient: payee.sender.to_string(),
            start_time: env.block.time.seconds(),
            stop_time: env.block.time.plus_seconds(100).seconds(),
            stream_type: None,
            curve: None,
        };

        // We need this to unwrap as an error
        let _ = execute(deps.as_mut(), env.clone(), payer.clone(), stream_msg).unwrap_err();

        // Attempt to create a stream with the correct deposit and provided funds
        let stream_msg = ExecuteMsg::CreateStream {
            asset: Asset {
                amount: Uint128::new(1000),
                info: AssetInfo::Native("axlusdc".to_string()),
            },
            recipient: payee.sender.to_string(),
            start_time: env.block.time.seconds(),
            stop_time: env.block.time.plus_seconds(100).seconds(),
            stream_type: None,
            curve: None,
        };
        // No issue
        let _ = execute(deps.as_mut(), env.clone(), payer.clone(), stream_msg).unwrap();

        // TODO: Probably a test on its own, what if user provides too much monies but their specified deposit is not enough?
        // Maybe reject? Or have a refund option?

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::StreamCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.count);
    }

    #[test]
    fn can_withdraw_from_a_created_stream_happy_and_sad_path() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 0 };
        // Creator user will create the stream
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::StreamCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(0, value.count);

        // Create 'payer' user with 1000 USD in balance to act as the payer of the stream
        let payer = mock_info("payer", &coins(100, "axlusdc"));

        // Create 'payee' user with 0 USD in balance to act as the receiver of funds from the stream
        let payee = mock_info("payee", &coins(0, "axlusdc"));

        let mut env = mock_env();

        let stream_msg = ExecuteMsg::CreateStream {
            asset: Asset {
                amount: Uint128::new(100),
                info: AssetInfo::Native("axlusdc".to_string()),
            },
            recipient: payee.sender.to_string(),
            start_time: env.block.time.seconds(),
            stop_time: env.block.time.plus_seconds(100).seconds(),
            stream_type: None,
            curve: None,
        };

        let execute_res = execute(deps.as_mut(), env.clone(), payer.clone(), stream_msg).unwrap();

        assert_eq!(execute_res.events.len(), 0);

        // Verify the payee cant get all right away
        let withdraw_msg = ExecuteMsg::ClaimFromStream {
            amount: Uint128::new(90),
            denom: String::from("axlusdc"),
            recipient: payee.sender.to_string(),
            stream_idx: None,
        };
        let execute_res =
            execute(deps.as_mut(), env.clone(), payer.clone(), withdraw_msg).unwrap_err();

        match execute_res {
            ContractError::NotEnoughAvailableBalance {} => {}
            _ => panic!("DO NOT ENTER HERE"),
        }

        env.block.time = Timestamp::from_seconds(env.block.time.seconds() + 10);
        let withdraw_msg = ExecuteMsg::ClaimFromStream {
            amount: Uint128::new(10),
            denom: String::from("axlusdc"),
            recipient: payee.sender.to_string(),
            stream_idx: None,
        };
        let execute_res = execute(deps.as_mut(), env.clone(), payer.clone(), withdraw_msg).unwrap();
        assert_eq!(1, execute_res.messages.len());

        // Verify the payee has indeed been paid by verifying the attached bank submessage
        assert_eq!(
            execute_res.messages,
            vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "payee".to_string(),
                amount: vec![Coin {
                    denom: "axlusdc".to_string(),
                    amount: Uint128::from(10u128),
                }],
            }))]
        );

        // Verify the payee cant get all right away
        let withdraw_msg = ExecuteMsg::ClaimFromStream {
            amount: Uint128::new(90),
            denom: String::from("axlusdc"),
            recipient: payee.sender.to_string(),
            stream_idx: None,
        };
        let execute_res =
            execute(deps.as_mut(), env.clone(), payer.clone(), withdraw_msg).unwrap_err();

        match execute_res {
            ContractError::NotEnoughAvailableBalance {} => {}
            _ => panic!("DO NOT ENTER HERE"),
        }

        // Simulate the rest of the time, payee is able to get more now
        // env.block.height += 70;
        env.block.time = Timestamp::from_seconds(env.block.time.seconds() + 90);
        let withdraw_msg = ExecuteMsg::ClaimFromStream {
            amount: Uint128::new(90),
            denom: String::from("axlusdc"),
            recipient: payee.sender.to_string(),
            stream_idx: None,
        };
        let execute_res = execute(deps.as_mut(), env.clone(), payer.clone(), withdraw_msg).unwrap();

        assert_eq!(1, execute_res.messages.len());

        // Verify the payee has indeed been paid by verifying the attached bank submessage
        assert_eq!(
            execute_res.messages,
            vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "payee".to_string(),
                amount: vec![Coin {
                    denom: "axlusdc".to_string(),
                    amount: Uint128::from(90u128),
                }],
            }))]
        );

        // Now this time, having drained the funds. No more should be sent right ?
        // env.block.height += 51;
        env.block.time = Timestamp::from_seconds(env.block.time.seconds() + 51);
        let withdraw_msg = ExecuteMsg::ClaimFromStream {
            amount: Uint128::new(10),
            denom: String::from("axlusdc"),
            recipient: payee.sender.to_string(),
            stream_idx: None,
        };
        let execute_res = execute(deps.as_mut(), env.clone(), payer, withdraw_msg).unwrap_err();

        match execute_res {
            ContractError::NotEnoughAvailableBalance {} => {}
            _ => panic!("DO NOT ENTER HERE"),
        }
    }
}
