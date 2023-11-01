mod basic_use_cases {

    use cosmwasm_std::{Addr, Coin, Uint128};

    use crate::{tests::suite::SuiteBuilder, ContractError};


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
            ContractError::DeltaIssue { start_time: 1571797519, stop_time: 1571797419 },
            err.downcast().unwrap(),
            "Expected InvalidTime error"
        );

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
                assert_eq!(stream.token_addr.to_string(), "native:ibc/something/axlusdc".to_string());
                assert_eq!(stream.start_time.seconds(), suite.get_time_as_timestamp().seconds());
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
                assert_eq!(stream.token_addr.to_string(), "native:ibc/something/axlusdc".to_string());
                assert_eq!(stream.start_time.seconds(), suite.get_time_as_timestamp().seconds());
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
                assert_eq!(stream.token_addr.to_string(), "native:ibc/something/axlusdc".to_string());
                assert_eq!(stream.start_time.seconds(), suite.get_time_as_timestamp().seconds());
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
        let claimable_amount = suite
            .query_stream_claimable_amount(1u64)
            .unwrap();
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
                assert_eq!(stream.token_addr.to_string(), "native:ibc/something/axlusdc".to_string());
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
        assert_eq!(streams[0].token_addr.to_string(),"native:ibc/something/axlusdc".to_string());
        assert_eq!(streams[0].start_time.seconds(), suite.get_time_as_timestamp().seconds());
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
        assert_eq!(streams[0].token_addr.to_string(), "native:ibc/something/axlusdc".to_string());
        assert_eq!(streams[0].start_time.seconds(), suite.get_time_as_timestamp().seconds());
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
                assert_eq!(stream.token_addr.to_string(), "native:ibc/something/axlusdc".to_string());
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
                assert_eq!(stream.token_addr.to_string(),  "native:ibc/something/axlusdc".to_string());
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
