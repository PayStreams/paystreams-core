use cosmwasm_std::{Deps, Env, Response, StdResult, Timestamp, Uint128};
use wynd_utils::{Curve, CurveError};

use crate::{
    state::{payment_streams, PaymentStream, StreamType},
    ContractError,
};

pub fn calc_rate_per_second(duration: u64, deposit: Uint128) -> Result<Uint128, ContractError> {
    if duration == 0 {
        Err(ContractError::DivisionByZero {})
    } else {
        // Perform the calculation in a higher precision by scaling the deposit before division
        // This scales the deposit to a larger number to avoid losing the fraction when dividing by a large duration.
        let scaling_factor = 1_000_000_000_000_000; // Scaling to a billion for precision
        let scaled_deposit = deposit
            .u128()
            .checked_mul(scaling_factor)
            .ok_or(ContractError::DivisionByZero {})?;

        let rate_per_second = scaled_deposit
            .checked_div(duration as u128)
            .ok_or(ContractError::DivisionByZero {})?
            .checked_div(scaling_factor)
            .ok_or(ContractError::DivisionByZero {})?;
        println!("Rate per second: {:?}", rate_per_second);
        println!("Scaled deposit: {:?}", scaled_deposit);

        Ok(Uint128::new(rate_per_second))
    }
}

pub fn validate_curve(stream_type: StreamType, curve: &Curve) -> Result<(), CurveError> {
    match stream_type {
        StreamType::Basic => {
            // For a basic stream, we expect a constant curve.
            match curve {
                Curve::Constant { .. } => Ok(()),
                _ => Err(CurveError::NotMonotonic),
            }
        }
        StreamType::LinearCurveBased => {
            // For a linear curve, we expect a monotonically increasing curve.
            curve.validate_monotonic_increasing()
        }
        StreamType::CliffCurveBased => {
            // For a cliff curve, we expect a monotonically increasing curve after an initial period.
            // The PiecewiseLinear should start with a constant y (the cliff) followed by increasing y values.
            match curve {
                Curve::PiecewiseLinear(piecewise) => {
                    if piecewise.steps.len() < 2 {
                        return Err(CurveError::MissingSteps);
                    }
                    // Ensure the first part of the curve is flat (the cliff).
                    if piecewise.steps[0].1 != piecewise.steps[1].1 {
                        return Err(CurveError::NotMonotonic);
                    }
                    // The rest of the curve should be monotonically increasing.
                    piecewise.validate_monotonic_increasing()
                }
                _ => Err(CurveError::NotMonotonic),
            }
        }
        StreamType::DynamicCurveBased => {
            // The dynamic curve can be complex and is outside of the scope of this simple validation.
            // More context would be needed to validate.
            Ok(())
        }
        StreamType::ExponentialCurveBased | StreamType::ExponentialCurveBasedWithCliff => {
            // Exponential curves cannot be perfectly represented by linear pieces, but we can check
            // if the curve is monotonically increasing which is a basic expectation.
            curve.validate_monotonic_increasing()
        }
        StreamType::TraditionalUnlockStepCurve => {
            // A traditional unlock step curve should have steps, which means it should be PiecewiseLinear
            // with segments where y remains the same (steps), followed by increases.
            match curve {
                Curve::PiecewiseLinear(piecewise) => {
                    // Verify the steps are properly ordered and values either stay the same or increase.
                    let mut last_x = 0;
                    let mut last_y = Uint128::zero();
                    for (x, y) in &piecewise.steps {
                        if *x <= last_x {
                            return Err(CurveError::PointsOutOfOrder);
                        }
                        if *y < last_y {
                            return Err(CurveError::NotMonotonic);
                        }
                        last_x = *x;
                        last_y = *y;
                    }
                    Ok(())
                }
                _ => Err(CurveError::NotMonotonic),
            }
        }
    }
}

pub fn avail_balance_of(stream: PaymentStream, env: Env) -> Result<Uint128, ContractError> {
    // Get the time delta
    let delta = delta(stream.clone(), env.clone())?;
    println!("Delta: {:?}", delta);
    match stream.curve {
        Some(curve) => {
            match curve.clone() {
                // The calculation for each curve is the same only the curve changes so we can use the same logic for each
                // Any of these types are the same Curve::Constant { _ } | Curve::SaturatingLinear(s) | Curve::PiecewiseLinear(s)
                Curve::Constant { y } => {
                    let rec_amt =
                        stream.deposit.u128() - curve.value(env.block.time.seconds()).u128();
                    let rec_bal = Uint128::from(rec_amt);
                    let amount_available = stream.deposit.checked_sub(stream.remaining_balance)?;
                    let new_balance = rec_bal.checked_sub(amount_available)?;

                    // Calc this better

                    return Ok(new_balance);
                }
                Curve::SaturatingLinear(s) => {
                    let avail_amt = curve.value(env.block.time.seconds());
                    let amount_available = stream.deposit.checked_sub(stream.remaining_balance)?;
                    let new_balance = avail_amt.checked_sub(amount_available)?;
                    println!("New balance {:?}", new_balance);
                    println!(
                        "New balance {:?}",
                        curve.value(env.block.time.seconds()).u128()
                    );

                    // Calc this better

                    return Ok(new_balance);
                }
                Curve::PiecewiseLinear(s) => {
                    let avail_amt = curve.value(env.block.time.seconds());
                    let amount_available = stream.deposit.checked_sub(stream.remaining_balance)?;
                    let new_balance = avail_amt.checked_sub(amount_available)?;
                    println!("New balance {:?}", new_balance);
                    println!(
                        "New balance {:?}",
                        curve.value(env.block.time.seconds()).u128()
                    );

                    // Calc this better

                    return Ok(new_balance);
                }
                _ => {
                    return Err(ContractError::Unauthorized {});
                }
            };
        }
        None => {
            // TO get the available balance off a stream we need to calculate the delta and multiply it by the rate per second
            // This requires the rate_per_second to be set and to be able to handle
            // Use delta to get the balance that should be available
            let rate_per_second: Uint128;
            let due_balance: Uint128;
            // Calculate the expected amount for the given duration to find a fallback rate
            let fallback_rate_per_second = stream.deposit.multiply_ratio(
                Uint128::from(delta), // For rate per second, the numerator should be 1 second
                Uint128::from(stream.stop_time.seconds() - stream.start_time.seconds()),
            );
            println!("Fallback rate per second: {:?}", fallback_rate_per_second);
            rate_per_second = fallback_rate_per_second;
            due_balance = fallback_rate_per_second;
            // println!("Delta: {:?}, Recipient Balance: {:?}", delta, rec_bal);

            // Calculate the rate per second on the fly and use that to calculate the available balance
            // This is a fallback in case the rate_per_second is not set
            // let rate_per_second = calc_rate_per_second(delta, stream.deposit)?;
            println!("Rate per second: {:?}", rate_per_second);

            if stream.deposit >= stream.remaining_balance {
                println!(
                    "Deposit: {:?}, Remaining: {:?}",
                    stream.deposit, stream.remaining_balance
                );
                // If the stream is ended, just give the remaining balance
                if stream.stop_time <= env.block.time {
                    println!("{:?} {:?}", stream.stop_time, env.block.time);
                    return Ok(stream.remaining_balance);
                }

                let amount_available = stream.deposit.checked_sub(stream.remaining_balance)?;
                println!("Amount available: {:?}", amount_available);
                let new_balance = due_balance.checked_sub(amount_available)?;
                // println!("New balance {:?}", new_balance);
                println!("New balance {:?}", new_balance);

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

pub fn deltaOf(deps: Deps, env: Env, stream_id: u64) -> StdResult<u64> {
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
