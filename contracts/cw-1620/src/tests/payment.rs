mod basic_use_cases {

    use cosmwasm_std::{Addr, Coin, Uint128};
    use wynd_utils::Curve;

    use crate::{state::StreamType, tests::suite::SuiteBuilder, ContractError};

    #[test]
    fn test_create_stream_errors() {
        let funder = Addr::unchecked("funder");
        let alice = Addr::unchecked("alice");
        let bob = Addr::unchecked("bob");
        let charlie = Addr::unchecked("charlie");
        let recipients = vec![alice, bob, charlie];
        let mut suite = SuiteBuilder::new()
            .with_funds(
                &funder.to_string(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(1000000000u128),
                }],
            )
            .build();

        // Attempt to create a stream
        let err = suite
            .create_stream(
                funder.clone(),
                recipients[0].clone(),
                100u128,
                "ibc/something/axlusdc",
                suite.get_time_as_timestamp().plus_seconds(100).seconds(),
                suite.get_time_as_timestamp().seconds(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(100u128),
                }],
                None,
                None,
            )
            .unwrap_err();
        assert_eq!(
            ContractError::DeltaIssue {
                start_time: 1571797519,
                stop_time: 1571797419
            },
            err.downcast().unwrap(),
            "Expected InvalidTime error"
        );
    }

    #[test]
    fn test_create_stream_with_curve() {
        let funder = Addr::unchecked("funder");
        let alice = Addr::unchecked("alice");
        let bob = Addr::unchecked("bob");
        let charlie = Addr::unchecked("charlie");
        let recipients = vec![alice, bob, charlie];
        let mut suite = SuiteBuilder::new()
            .with_funds(
                &funder.to_string(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(1000000000u128),
                }],
            )
            .build();

        // Attempt to create a stream
        suite
            .create_stream(
                funder.clone(),
                recipients[0].clone(),
                100u128,
                "ibc/something/axlusdc",
                suite.get_time_as_timestamp().seconds(),
                suite.get_time_as_timestamp().plus_seconds(100).seconds(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(100u128),
                }],
                Some(StreamType::LinearCurveBased),
                Some(Curve::SaturatingLinear(wynd_utils::SaturatingLinear {
                    min_x: suite.get_time_as_timestamp().seconds(),
                    min_y: Uint128::zero(),
                    max_x: suite.get_time_as_timestamp().seconds() + 100,
                    max_y: Uint128::from(100u128),
                })),
            )
            .unwrap();

        // Verify that the stream was created
        let created_stream = suite.query_stream_by_index(1u64).unwrap().streams.pop();

        match created_stream {
            Some(stream) => {
                assert_eq!(stream.recipient, recipients[0].to_string());
                assert_eq!(stream.sender, funder.to_string());
                assert_eq!(stream.deposit, Uint128::from(100u128));
                assert_eq!(
                    stream.token_addr.to_string(),
                    "native:ibc/something/axlusdc".to_string()
                );
                assert_eq!(
                    stream.start_time.seconds(),
                    suite.get_time_as_timestamp().seconds()
                );
                assert_eq!(
                    stream.stop_time.seconds(),
                    suite.get_time_as_timestamp().plus_seconds(100).seconds()
                );
            }
            None => {
                panic!("Stream was not created");
            }
        }

        // Verify again but query by recipient
        let created_stream = suite
            .query_streams_by_sender(funder.clone())
            .unwrap()
            .streams
            .pop();
        match created_stream {
            Some(stream) => {
                assert_eq!(stream.recipient, recipients[0].to_string());
                assert_eq!(stream.sender, funder.to_string());
                assert_eq!(stream.deposit, Uint128::from(100u128));
                assert_eq!(
                    stream.token_addr.to_string(),
                    "native:ibc/something/axlusdc".to_string()
                );
                assert_eq!(
                    stream.start_time.seconds(),
                    suite.get_time_as_timestamp().seconds()
                );
                assert_eq!(
                    stream.stop_time.seconds(),
                    suite.get_time_as_timestamp().plus_seconds(100).seconds()
                );
            }
            None => {
                panic!("Stream by sender was not created or not found");
            }
        }

        // Verify again but query by payee
        let created_stream = suite
            .query_streams_by_payee(recipients[0].clone())
            .unwrap()
            .streams
            .pop();

        match created_stream {
            Some(stream) => {
                assert_eq!(stream.recipient, recipients[0].to_string());
                assert_eq!(stream.sender, funder.to_string());
                assert_eq!(stream.deposit, Uint128::from(100u128));
                assert_eq!(
                    stream.token_addr.to_string(),
                    "native:ibc/something/axlusdc".to_string()
                );
                assert_eq!(
                    stream.start_time.seconds(),
                    suite.get_time_as_timestamp().seconds()
                );
                assert_eq!(
                    stream.stop_time.seconds(),
                    suite.get_time_as_timestamp().plus_seconds(100).seconds()
                );
            }
            None => {
                panic!("Stream by payee was not created or not found");
            }
        }

        // Pass some time so the recipient can withdraw something
        suite.update_time(41);

        // Verify we have some claimable amount
        let claimable_amount = suite.query_stream_claimable_amount(1u64).unwrap();
        assert_eq!(claimable_amount, 41u128);
        println!("Claimable amount: {}", suite.get_time_as_timestamp());
        // Attempt to withdraw with recipient

        suite
            .withdraw_from_stream(
                recipients[0].clone(),
                41u128,
                "ibc/something/axlusdc",
                Some(1u64),
            )
            .unwrap();

        // Verify that the recipient has withdrawn something
        let created_stream = suite.query_stream_by_index(1u64).unwrap().streams.pop();

        match created_stream {
            Some(stream) => {
                assert_eq!(stream.recipient, recipients[0].to_string());
                assert_eq!(stream.sender, funder.to_string());
                assert_eq!(stream.deposit, Uint128::from(100u128));
                assert_eq!(
                    stream.token_addr.to_string(),
                    "native:ibc/something/axlusdc".to_string()
                );
                assert_eq!(stream.remaining_balance, Uint128::from(59u128));
            }
            None => {
                panic!("Stream was not created");
            }
        }

        // Pass some time so the recipient can withdraw something
        suite.update_time(59);

        // Verify we have some claimable amount
        let claimable_amount = suite.query_stream_claimable_amount(1u64).unwrap();
        assert_eq!(claimable_amount, 59u128);
        println!("Claimable amount: {}", suite.get_time_as_timestamp());
        // Attempt to withdraw with recipient

        suite
            .withdraw_from_stream(
                recipients[0].clone(),
                59u128,
                "ibc/something/axlusdc",
                Some(1u64),
            )
            .unwrap();

        // Also verify recipients balance of this token has increased, its a native token so we can use Bank
        let balance = suite
            .query_balance(&recipients[0].clone().to_string(), "ibc/something/axlusdc")
            .unwrap();
        assert_eq!(balance, 100u128);
    }

    #[test]
    fn test_create_stream() {
        let funder = Addr::unchecked("funder");
        let alice = Addr::unchecked("alice");
        let bob = Addr::unchecked("bob");
        let charlie = Addr::unchecked("charlie");
        let recipients = vec![alice, bob, charlie];
        let mut suite = SuiteBuilder::new()
            .with_funds(
                &funder.to_string(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(1000000000u128),
                }],
            )
            .build();

        // Attempt to create a stream
        suite
            .create_stream(
                funder.clone(),
                recipients[0].clone(),
                100u128,
                "ibc/something/axlusdc",
                suite.get_time_as_timestamp().seconds(),
                suite.get_time_as_timestamp().plus_seconds(100).seconds(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(100u128),
                }],
                None,
                None,
            )
            .unwrap();

        // Verify that the stream was created
        let created_stream = suite.query_stream_by_index(1u64).unwrap().streams.pop();

        match created_stream {
            Some(stream) => {
                assert_eq!(stream.recipient, recipients[0].to_string());
                assert_eq!(stream.sender, funder.to_string());
                assert_eq!(stream.deposit, Uint128::from(100u128));
                assert_eq!(
                    stream.token_addr.to_string(),
                    "native:ibc/something/axlusdc".to_string()
                );
                assert_eq!(
                    stream.start_time.seconds(),
                    suite.get_time_as_timestamp().seconds()
                );
                assert_eq!(
                    stream.stop_time.seconds(),
                    suite.get_time_as_timestamp().plus_seconds(100).seconds()
                );
            }
            None => {
                panic!("Stream was not created");
            }
        }

        // Verify again but query by recipient
        let created_stream = suite
            .query_streams_by_sender(funder.clone())
            .unwrap()
            .streams
            .pop();
        match created_stream {
            Some(stream) => {
                assert_eq!(stream.recipient, recipients[0].to_string());
                assert_eq!(stream.sender, funder.to_string());
                assert_eq!(stream.deposit, Uint128::from(100u128));
                assert_eq!(
                    stream.token_addr.to_string(),
                    "native:ibc/something/axlusdc".to_string()
                );
                assert_eq!(
                    stream.start_time.seconds(),
                    suite.get_time_as_timestamp().seconds()
                );
                assert_eq!(
                    stream.stop_time.seconds(),
                    suite.get_time_as_timestamp().plus_seconds(100).seconds()
                );
            }
            None => {
                panic!("Stream by sender was not created or not found");
            }
        }

        // Verify again but query by payee
        let created_stream = suite
            .query_streams_by_payee(recipients[0].clone())
            .unwrap()
            .streams
            .pop();

        match created_stream {
            Some(stream) => {
                assert_eq!(stream.recipient, recipients[0].to_string());
                assert_eq!(stream.sender, funder.to_string());
                assert_eq!(stream.deposit, Uint128::from(100u128));
                assert_eq!(
                    stream.token_addr.to_string(),
                    "native:ibc/something/axlusdc".to_string()
                );
                assert_eq!(
                    stream.start_time.seconds(),
                    suite.get_time_as_timestamp().seconds()
                );
                assert_eq!(
                    stream.stop_time.seconds(),
                    suite.get_time_as_timestamp().plus_seconds(100).seconds()
                );
            }
            None => {
                panic!("Stream by payee was not created or not found");
            }
        }

        // Pass some time so the recipient can withdraw something
        suite.update_time(50);

        // Verify we have some claimable amount
        let claimable_amount = suite.query_stream_claimable_amount(1u64).unwrap();
        assert_eq!(claimable_amount, 50u128);

        // Attempt to withdraw with recipient

        suite
            .withdraw_from_stream(
                recipients[0].clone(),
                50u128,
                "ibc/something/axlusdc",
                Some(1u64),
            )
            .unwrap();

        // Verify that the recipient has withdrawn something
        let created_stream = suite.query_stream_by_index(1u64).unwrap().streams.pop();

        match created_stream {
            Some(stream) => {
                assert_eq!(stream.recipient, recipients[0].to_string());
                assert_eq!(stream.sender, funder.to_string());
                assert_eq!(stream.deposit, Uint128::from(100u128));
                assert_eq!(
                    stream.token_addr.to_string(),
                    "native:ibc/something/axlusdc".to_string()
                );
                assert_eq!(stream.remaining_balance, Uint128::from(50u128));
            }
            None => {
                panic!("Stream was not created");
            }
        }

        // Also verify recipients balance of this token has increased, its a native token so we can use Bank
        let balance = suite
            .query_balance(&recipients[0].clone().to_string(), "ibc/something/axlusdc")
            .unwrap();
        assert_eq!(balance, 50u128);
    }

    #[test]
    pub fn test_create_multiple_streams() {
        let funder = Addr::unchecked("funder");
        let alice = Addr::unchecked("alice");
        let bob = Addr::unchecked("bob");
        let charlie = Addr::unchecked("charlie");
        let recipients = vec![alice, bob, charlie];
        let mut suite = SuiteBuilder::new()
            .with_funds(
                &funder.to_string(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(1000000000u128),
                }],
            )
            .build();

        // For each recipient, create a stream
        for recipient in recipients.iter() {
            suite
                .create_stream(
                    funder.clone(),
                    recipient.clone(),
                    100u128,
                    "ibc/something/axlusdc",
                    suite.get_time_as_timestamp().seconds(),
                    suite.get_time_as_timestamp().plus_seconds(100).seconds(),
                    &[Coin {
                        denom: "ibc/something/axlusdc".to_string(),
                        amount: Uint128::from(100u128),
                    }],
                    None,
                    None,
                )
                .unwrap();
        }

        // Advance time 20% of the way through the streams
        suite.update_time(20);

        // For each recipient, withdraw from their stream
        for (idx, recipient) in recipients.iter().enumerate() {
            // Each recipient can only withdraw 20% and not for example 21%
            let err = suite
                .withdraw_from_stream(
                    recipient.clone(),
                    21u128,
                    "ibc/something/axlusdc",
                    Some(idx as u64 + 1u64),
                )
                .unwrap_err();
            assert_eq!(
                ContractError::NotEnoughAvailableBalance {},
                err.downcast().unwrap(),
                "Expected NotEnoughAvailableBalance error"
            );

            suite
                .withdraw_from_stream(
                    recipient.clone(),
                    20u128,
                    "ibc/something/axlusdc",
                    Some(idx as u64 + 1u64),
                )
                .unwrap();

            // Each recipient cannot withdraw more than their stream balance, so no more than already gotten by now
            let err = suite
                .withdraw_from_stream(
                    recipient.clone(),
                    100u128,
                    "ibc/something/axlusdc",
                    Some(idx as u64 + 1u64),
                )
                .unwrap_err();
            assert_eq!(
                ContractError::NotEnoughAvailableBalance {},
                err.downcast().unwrap(),
                "Expected NotEnoughAvailableBalance error"
            );
        }
    }
}

mod views_with_mock_data {

    // For this module we want to test the query data we will be consuming on the frontend
    // We will create a few streams and then query them using the views we have defined
    // We will then assert that the data returned is what we expect

    use cosmwasm_std::{Addr, Coin, Uint128};

    use crate::{state::PaymentStream, tests::suite::SuiteBuilder};

    #[test]
    fn test_query_stream_count() {
        let funder = Addr::unchecked("funder");
        let alice = Addr::unchecked("alice");
        let bob = Addr::unchecked("bob");
        let charlie = Addr::unchecked("charlie");
        let recipients = vec![alice, bob, charlie];
        let mut suite = SuiteBuilder::new()
            .with_funds(
                &funder.to_string(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(1000000000u128),
                }],
            )
            .build();

        // For each recipient, create a stream
        for recipient in recipients.iter() {
            suite
                .create_stream(
                    funder.clone(),
                    recipient.clone(),
                    100u128,
                    "ibc/something/axlusdc",
                    suite.get_time_as_timestamp().seconds(),
                    suite.get_time_as_timestamp().plus_seconds(100).seconds(),
                    &[Coin {
                        denom: "ibc/something/axlusdc".to_string(),
                        amount: Uint128::from(100u128),
                    }],
                    None,
                    None,
                )
                .unwrap();
        }

        // Verify that the stream count is 3
        let count = suite.query_stream_count();
        assert_eq!(count, 3u64);
    }

    #[test]
    fn test_query_streams_by_payee_one_stream() {
        let funder = Addr::unchecked("funder");
        let alice = Addr::unchecked("alice");
        let bob = Addr::unchecked("bob");
        let charlie = Addr::unchecked("charlie");
        let recipients = vec![alice, bob, charlie];
        let mut suite = SuiteBuilder::new()
            .with_funds(
                &funder.to_string(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(1000000000u128),
                }],
            )
            .build();

        // For each recipient, create a stream
        for recipient in recipients.iter() {
            suite
                .create_stream(
                    funder.clone(),
                    recipient.clone(),
                    100u128,
                    "ibc/something/axlusdc",
                    suite.get_time_as_timestamp().seconds(),
                    suite.get_time_as_timestamp().plus_seconds(100).seconds(),
                    &[Coin {
                        denom: "ibc/something/axlusdc".to_string(),
                        amount: Uint128::from(100u128),
                    }],
                    None,
                    None,
                )
                .unwrap();
        }

        // Verify that the stream count is 3
        let count = suite.query_stream_count();
        assert_eq!(count, 3u64);

        // Verify that the streams by payee are what we expect
        let streams = suite
            .query_streams_by_payee(recipients[0].clone())
            .unwrap()
            .streams;
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].recipient, recipients[0].to_string());
        assert_eq!(streams[0].sender, funder.to_string());
        assert_eq!(streams[0].deposit, Uint128::from(100u128));
        assert_eq!(
            streams[0].token_addr.to_string(),
            "native:ibc/something/axlusdc".to_string()
        );
        assert_eq!(
            streams[0].start_time.seconds(),
            suite.get_time_as_timestamp().seconds()
        );
        assert_eq!(
            streams[0].stop_time.seconds(),
            suite.get_time_as_timestamp().plus_seconds(100).seconds()
        );
    }

    #[test]
    fn test_query_streams_by_payee_multiple_streams() {
        let funder = Addr::unchecked("funder");
        let alice = Addr::unchecked("alice");
        let bob = Addr::unchecked("bob");
        let charlie = Addr::unchecked("charlie");
        let recipients = vec![alice, bob, charlie];
        let mut suite = SuiteBuilder::new()
            .with_funds(
                &funder.to_string(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(1000000000u128),
                }],
            )
            .build();

        // For each recipient, create a stream
        for recipient in recipients.iter() {
            suite
                .create_stream(
                    funder.clone(),
                    recipient.clone(),
                    100u128,
                    "ibc/something/axlusdc",
                    suite.get_time_as_timestamp().seconds(),
                    suite.get_time_as_timestamp().plus_seconds(100).seconds(),
                    &[Coin {
                        denom: "ibc/something/axlusdc".to_string(),
                        amount: Uint128::from(100u128),
                    }],
                    None,
                    None,
                )
                .unwrap();
        }

        // Create an extra stream for recipients[1]
        suite
            .create_stream(
                funder.clone(),
                recipients[1].clone(),
                100u128,
                "ibc/something/axlusdc",
                suite.get_time_as_timestamp().seconds(),
                suite.get_time_as_timestamp().plus_seconds(100).seconds(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(100u128),
                }],
                None,
                None,
            )
            .unwrap();

        // Verify that the stream count is 3
        let count = suite.query_stream_count();
        assert_eq!(count, 4u64);

        // Verify that the streams by payee are what we expect
        let streams = suite
            .query_streams_by_payee(recipients[1].clone())
            .unwrap()
            .streams;
        assert_eq!(streams.len(), 2);
        assert_eq!(streams[0].recipient, recipients[1].to_string());
        assert_eq!(streams[0].sender, funder.to_string());
        assert_eq!(streams[0].deposit, Uint128::from(100u128));
        assert_eq!(
            streams[0].token_addr.to_string(),
            "native:ibc/something/axlusdc".to_string()
        );
        assert_eq!(
            streams[0].start_time.seconds(),
            suite.get_time_as_timestamp().seconds()
        );
        assert_eq!(
            streams[0].stop_time.seconds(),
            suite.get_time_as_timestamp().plus_seconds(100).seconds()
        );
    }

    // Test that we can query streams by sender, setup multiple streams and query them
    // Then setup 1 extra stream and do 1 more query to verify it was added
    // We want to verify the full stream response and all its data
    #[test]
    fn test_query_streams_by_sender_multiple_streams() {
        let funder = Addr::unchecked("funder");
        let alice = Addr::unchecked("alice");
        let bob = Addr::unchecked("bob");
        let charlie = Addr::unchecked("charlie");
        let recipients = vec![alice, bob, charlie];
        let mut suite = SuiteBuilder::new()
            .with_funds(
                &funder.to_string(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(1000000000u128),
                }],
            )
            .build();

        // all_streams will eventually hold all streams
        let mut all_streams: Vec<PaymentStream> = vec![];

        // For each recipient, create a stream
        for (idx, recipient) in recipients.iter().enumerate() {
            suite
                .create_stream(
                    funder.clone(),
                    recipient.clone(),
                    100u128,
                    "ibc/something/axlusdc",
                    suite.get_time_as_timestamp().seconds(),
                    suite.get_time_as_timestamp().plus_seconds(100).seconds(),
                    &[Coin {
                        denom: "ibc/something/axlusdc".to_string(),
                        amount: Uint128::from(100u128),
                    }],
                    None,
                    None,
                )
                .unwrap();
            // Now query the stream by index and save it to all_streams vec for later
            let stream = suite
                .query_stream_by_index(idx as u64 + 1)
                .unwrap()
                .streams
                .pop();
            match stream {
                Some(stream) => {
                    all_streams.push(stream);
                }
                None => {
                    panic!("Stream was not created");
                }
            }
        }

        // Create an extra stream for recipients[1]
        suite
            .create_stream(
                funder.clone(),
                recipients[1].clone(),
                100u128,
                "ibc/something/axlusdc",
                suite.get_time_as_timestamp().seconds(),
                suite.get_time_as_timestamp().plus_seconds(100).seconds(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(100u128),
                }],
                None,
                None,
            )
            .unwrap();

        // Now query the stream by index and save it to all_streams vec for later
        let stream = suite.query_stream_by_index(4u64).unwrap().streams.pop();
        match stream {
            Some(stream) => {
                all_streams.push(stream);
            }
            None => {
                panic!("Stream was not created");
            }
        }

        // Verify that the stream count is 3
        let count = suite.query_stream_count();
        assert_eq!(count, 4u64);

        // Verify that the streams by payee are what we expect
        let streams = suite
            .query_streams_by_sender(funder.clone())
            .unwrap()
            .streams;
        assert_eq!(streams.len(), 4);
        assert_eq!(streams, all_streams);
    }
}

mod curve_tests {
    use cosmwasm_std::{Addr, Coin, Uint128};
    use wynd_utils::Curve;

    use crate::{state::StreamType, tests::suite::SuiteBuilder};

    #[test]
    fn test_linear_curve() {
        let funder = Addr::unchecked("funder");
        let alice = Addr::unchecked("alice");
        let bob = Addr::unchecked("bob");
        let charlie = Addr::unchecked("charlie");
        let recipients = vec![alice, bob, charlie];
        let mut suite = SuiteBuilder::new()
            .with_funds(
                &funder.to_string(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(1000000000u128),
                }],
            )
            .build();

        //                Some(Curve::saturating_linear((suite.get_time_as_timestamp().seconds(), 0u128), (suite.get_time_as_timestamp().seconds()+100, 100u128))),

        // Create a stream with a linear curve
        suite
            .create_stream(
                funder.clone(),
                recipients[0].clone(),
                100u128,
                "ibc/something/axlusdc",
                suite.get_time_as_timestamp().seconds(),
                suite.get_time_as_timestamp().plus_seconds(100).seconds(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(100u128),
                }],
                Some(StreamType::LinearCurveBased),
                Some(Curve::saturating_linear(
                    (suite.get_time_as_timestamp().seconds(), 0u128),
                    (suite.get_time_as_timestamp().seconds() + 100, 100u128),
                )),
            )
            .unwrap();

        // Advance time 20% of the way through the streams
        suite.update_time(20);

        // Verify that the recipient has withdrawn something
        let created_stream = suite.query_stream_by_index(1u64).unwrap().streams.pop();

        match created_stream {
            Some(stream) => {
                assert_eq!(stream.recipient, recipients[0].to_string());
                assert_eq!(stream.sender, funder.to_string());
                assert_eq!(stream.deposit, Uint128::from(100u128));
                assert_eq!(
                    stream.token_addr.to_string(),
                    "native:ibc/something/axlusdc".to_string()
                );
                assert_eq!(stream.remaining_balance, Uint128::from(100u128));
            }
            None => {
                panic!("Stream was not created");
            }
        }

        // Advance time 20% of the way through the streams
        suite.update_time(20);

        // Attempt to withdraw
        suite
            .withdraw_from_stream(
                recipients[0].clone(),
                20u128,
                "ibc/something/axlusdc",
                Some(1u64),
            )
            .unwrap();

        // Verify that the recipient has withdrawn something

        let created_stream = suite.query_stream_by_index(1u64).unwrap().streams.pop();

        match created_stream {
            Some(stream) => {
                assert_eq!(stream.recipient, recipients[0].to_string());
                assert_eq!(stream.sender, funder.to_string());
                assert_eq!(stream.deposit, Uint128::from(100u128));
                assert_eq!(
                    stream.token_addr.to_string(),
                    "native:ibc/something/axlusdc".to_string()
                );
                assert_eq!(stream.remaining_balance, Uint128::from(80u128));
            }
            None => {
                panic!("Stream was not created");
            }
        }

        // Advance time 20% of the way through the streams

        suite.update_time(20);

        // Attempt to withdraw

        suite
            .withdraw_from_stream(
                recipients[0].clone(),
                20u128,
                "ibc/something/axlusdc",
                Some(1u64),
            )
            .unwrap();

        // Verify the balance of recipient is 40
        let balance = suite
            .query_balance(&recipients[0].clone().to_string(), "ibc/something/axlusdc")
            .unwrap();
        assert_eq!(balance, 40u128);
    }
}

mod rounding_tests {
    use crate::{curve_helpers, tests::suite::SuiteBuilder, ContractError};
    use cosmwasm_std::{Addr, Coin, Uint128};

    #[test]
    fn test_rounding_small_deposit_over_long_duration() {
        let funder = Addr::unchecked("funder");
        let recipient = Addr::unchecked("recipient");
        let mut suite = SuiteBuilder::new()
            .with_funds(
                &funder.to_string(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(1000000000u128),
                }],
            )
            .build();

        // Small deposit, long duration (1 uwhale over 1 year)
        suite
            .create_stream(
                funder.clone(),
                recipient.clone(),
                Uint128::from(1u128).u128(), // 1 uwhale (which is 1_000_000 in smallest denomination)
                "ibc/something/axlusdc",
                suite.get_time_as_timestamp().seconds(),
                suite
                    .get_time_as_timestamp()
                    .plus_seconds(31536000)
                    .seconds(), // 1 year in seconds
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(1u128),
                }],
                None,
                None,
            )
            .unwrap();

        // Simulate half the duration passing
        suite.update_time(15768000); // Half a year

        // Query the claimable amount after half a year
        let claimable_amount_halfway = suite.query_stream_claimable_amount(1u64).unwrap();
        assert_eq!(
            claimable_amount_halfway,
            Uint128::from(0u128).u128(),
            "Should be 0 due to rounding down"
        );

        // End of the stream duration
        suite.update_time(31536000); // End of the year

        // Full amount should be claimable at the end
        let claimable_amount_end = suite.query_stream_claimable_amount(1u64).unwrap();
        assert_eq!(
            claimable_amount_end,
            Uint128::from(1u128).u128(),
            "Full amount should be claimable"
        );
    }

    #[test]
    fn test_rounding_large_deposit_short_duration() {
        let funder = Addr::unchecked("funder");

        let mut suite = SuiteBuilder::new()
            .with_funds(
                &funder.to_string(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(1000000000u128),
                }],
            )
            .build();
        let funder = Addr::unchecked("funder");
        let recipient = Addr::unchecked("recipient");

        let large_deposit = Uint128::new(1_000_000_000);
        let short_duration = 10; // 10 seconds

        // Create the stream
        suite
            .create_stream(
                funder.clone(),
                recipient.clone(),
                large_deposit.into(),
                "ibc/something/axlusdc",
                suite.get_time_as_timestamp().seconds(),
                suite
                    .get_time_as_timestamp()
                    .plus_seconds(short_duration)
                    .seconds(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: large_deposit,
                }],
                None,
                None,
            )
            .unwrap();

        // Move the blockchain forward by 5 seconds
        suite.update_time(5);

        // Check the claimable amount
        let claimable_amount = suite.query_stream_claimable_amount(1u64).unwrap();
        let expected_claimable_halfway = large_deposit.u128() / 2;
        assert_eq!(
            claimable_amount,
            Uint128::from(expected_claimable_halfway).into(),
            "Claimable amount should be half of the deposit after half duration"
        );

        // Move to the end of the stream
        suite.update_time(short_duration);

        // Full amount should be claimable at the end
        let claimable_amount_end = suite.query_stream_claimable_amount(1u64).unwrap();
        assert_eq!(
            claimable_amount_end,
            large_deposit.into(),
            "Full amount should be claimable at the end of the duration"
        );
    }

    #[test]
    fn test_rounding_edge_case_for_one_second() {
        let funder = Addr::unchecked("funder");

        let mut suite = SuiteBuilder::new()
            .with_funds(
                &funder.to_string(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(1000000000u128),
                }],
            )
            .build();
        let funder = Addr::unchecked("funder");
        let recipient = Addr::unchecked("recipient");

        let deposit_for_one_second = Uint128::new(10); // 10 uwhale

        suite
            .create_stream(
                funder.clone(),
                recipient.clone(),
                deposit_for_one_second.into(),
                "ibc/something/axlusdc",
                suite.get_time_as_timestamp().seconds(),
                suite.get_time_as_timestamp().plus_seconds(1).seconds(), // 1 second stream
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: deposit_for_one_second,
                }],
                None,
                None,
            )
            .unwrap();

        // Move the blockchain forward by 1 second
        suite.update_time(1);

        // Query the claimable amount
        let claimable_amount = suite.query_stream_claimable_amount(1u64).unwrap();
        assert_eq!(
            claimable_amount,
            deposit_for_one_second.into(),
            "Claimable amount should match deposit after one second"
        );
    }

    #[test]
    fn test_rounding_with_intermittent_claims() {
        let funder = Addr::unchecked("funder");

        let mut suite = SuiteBuilder::new()
            .with_funds(
                &funder.to_string(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(1000000000u128),
                }],
            )
            .build();
        let funder = Addr::unchecked("funder");
        let recipient = Addr::unchecked("recipient");

        let deposit = Uint128::new(1_000_000); // 1 million uwhale
        let duration = 100; // 100 seconds

        suite
            .create_stream(
                funder.clone(),
                recipient.clone(),
                deposit.into(),
                "ibc/something/axlusdc",
                suite.get_time_as_timestamp().seconds(),
                suite
                    .get_time_as_timestamp()
                    .plus_seconds(duration)
                    .seconds(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: deposit,
                }],
                None,
                None,
            )
            .unwrap();

        let intervals = vec![1, 5, 10, 20, 64]; // Claim intervals in seconds
        let mut total_claimed = 0;

        for &interval in &intervals {
            // Move forward in time
            suite.update_time(interval);

            // Query and claim the available amount
            let claimable_amount = suite.query_stream_claimable_amount(1u64).unwrap();
            suite
                .withdraw_from_stream(
                    recipient.clone(),
                    Uint128::from(claimable_amount).u128(),
                    "ibc/something/axlusdc",
                    Some(1u64),
                )
                .unwrap();

            // Keep track of the total claimed amount
            total_claimed += claimable_amount;

            // Verify that the total claimed does not exceed what should have been paid out
            let expected_total_claimed = total_claimed.min(deposit.u128());
            assert_eq!(
                total_claimed, expected_total_claimed,
                "Total claimed should not exceed expected amount"
            );
        }

        // At the end of the stream, the remaining balance should be zero
        suite.update_time(duration);
        let remaining_balance = suite
            .query_stream_by_index(1u64)
            .unwrap()
            .streams
            .pop()
            .unwrap()
            .remaining_balance;
        assert_eq!(
            remaining_balance,
            Uint128::zero(),
            "Remaining balance should be zero at the end of the stream"
        );
    }

    #[test]
    fn test_stream_claimable_amount_over_time() {
        let funder = Addr::unchecked("funder");

        let mut suite = SuiteBuilder::new()
            .with_funds(
                &funder.to_string(),
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: Uint128::from(1000000000u128),
                }],
            )
            .build();
        let recipient = Addr::unchecked("recipient");

        // Total deposit for the stream
        let total_deposit = Uint128::new(1_000_000);

        // Timestamps for the start and end of the stream (1 year duration)
        let start_timestamp = 1571797419;
        let end_timestamp = start_timestamp + (12 * 30 * 24 * 60 * 60); // Adding 12 months worth of seconds

        // Creating the stream
        suite
            .create_stream(
                funder.clone(),
                recipient.clone(),
                total_deposit.u128(),
                "ibc/something/axlusdc",
                start_timestamp,
                end_timestamp,
                &[Coin {
                    denom: "ibc/something/axlusdc".to_string(),
                    amount: total_deposit,
                }],
                None,
                None,
            )
            .unwrap();

        // Define the month durations to test
        let month_durations = [1, 3, 6, 12];
        let reduction = [0, 1, 3, 6];
        let seconds_in_a_month = 30 * 24 * 60 * 60; // Approximate seconds in a month
        for &month_duration in &month_durations {
            // Calculate the timestamp for the end of the duration
            let duration_timestamp = start_timestamp + (month_duration * seconds_in_a_month);
            // Calculate the expected amount for the given duration
            let expected_amount = total_deposit.multiply_ratio(
                Uint128::from(month_duration * seconds_in_a_month),
                Uint128::from(end_timestamp - start_timestamp),
            );
            println!(
                "Expected amount after {} months is {}",
                month_duration,
                expected_amount.u128()
            );
            // This could be better
            let reduction = match month_duration {
                1 => reduction[0],
                3 => reduction[1],
                6 => reduction[2],
                12 => reduction[3],
                _ => {
                    panic!("Unexpected month duration");
                }
            };

            // Move the blockchain time forward, each time we need to subtract the amount of time we have already advanced
            suite.update_time((seconds_in_a_month * (month_duration - reduction)));

            // Query the claimable amount
            let claimable_amount = suite.query_stream_claimable_amount(1u64).unwrap();

            // Assert that the claimable amount is as expected
            println!(
                "Claimable amount after {} months is {}",
                month_duration, claimable_amount
            );
            assert_eq!(
                claimable_amount,
                expected_amount.u128(),
                "Claimable amount after {} months is not as expected",
                month_duration
            );
        }
    }
}
