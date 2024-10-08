use super::*;

#[cfg(feature = "electrum")]
#[test]
#[parallel]
fn success() {
    initialize();

    // receiver wallet
    let rcv_wallet = get_test_wallet(true, None);

    // drain funded wallet with no allocation UTXOs
    let (wallet, online) = get_funded_noutxo_wallet!();
    let expected_balance = BtcBalance {
        vanilla: Balance {
            settled: 100000000,
            future: 100000000,
            spendable: 100000000,
        },
        colored: Balance {
            settled: 0,
            future: 0,
            spendable: 0,
        },
    };
    wait_for_btc_balance(&wallet, &online, &expected_balance);
    let address = test_get_address(&rcv_wallet); // also updates backup_info
    let bak_info_before = wallet.database.get_backup_info().unwrap().unwrap();
    test_drain_to_keep(&wallet, &online, &address);
    let bak_info_after = wallet.database.get_backup_info().unwrap().unwrap();
    assert!(bak_info_after.last_operation_timestamp > bak_info_before.last_operation_timestamp);
    mine(false);
    wait_for_unspents(&wallet, Some(&online), false, 0);

    // issue asset (to produce an RGB allocation)
    fund_wallet(test_get_address(&wallet));
    test_create_utxos_default(&wallet, &online);
    mine(false);
    test_issue_asset_nia(&wallet, &online, None);

    // drain funded wallet with RGB allocations
    let expected_balance = BtcBalance {
        vanilla: Balance {
            settled: 99994508,
            future: 99994508,
            spendable: 99994508,
        },
        colored: Balance {
            settled: 5000,
            future: 5000,
            spendable: 5000,
        },
    };
    wait_for_btc_balance(&wallet, &online, &expected_balance);
    test_drain_to_keep(&wallet, &online, &test_get_address(&rcv_wallet));
    mine(false);
    wait_for_unspents(&wallet, Some(&online), false, UTXO_NUM);
    test_drain_to_destroy(&wallet, &online, &test_get_address(&rcv_wallet));
    mine(false);
    wait_for_unspents(&wallet, Some(&online), false, 0);
}

#[cfg(feature = "electrum")]
#[test]
#[parallel]
fn pending_witness_receive() {
    initialize();

    let amount: u64 = 66;

    // wallets
    let (wallet, online) = get_funded_wallet!();
    let (rcv_wallet, rcv_online) = get_funded_wallet!();
    let (drain_wallet, _drain_online) = get_empty_wallet!();

    // issue
    let asset = test_issue_asset_nia(&wallet, &online, None);

    // send
    stop_mining();
    let receive_data = test_witness_receive(&rcv_wallet);
    let recipient_map = HashMap::from([(
        asset.asset_id.clone(),
        vec![Recipient {
            amount,
            recipient_id: receive_data.recipient_id.clone(),
            witness_data: Some(WitnessData {
                amount_sat: 1000,
                blinding: None,
            }),
            transport_endpoints: TRANSPORT_ENDPOINTS.clone(),
        }],
    )]);
    let txid = test_send(&wallet, &online, &recipient_map);
    assert!(!txid.is_empty());

    // refresh receiver (no UTXOs created) + sender (to broadcast) + mine
    wait_for_refresh(&rcv_wallet, &rcv_online, None, None);
    wait_for_refresh(&wallet, &online, Some(&asset.asset_id), None);
    mine(true);

    // receiver still doesn't see the new UTXO (not refreshed a 2nd time yet)
    let unspents = list_test_unspents(&rcv_wallet, "before draining");
    assert_eq!(unspents.len(), 6);

    // drain receiver, which syncs the wallet, detecting (and draining) the new UTXO as well
    let address = test_get_address(&drain_wallet);
    test_drain_to_destroy(&rcv_wallet, &rcv_online, &address);
    let unspents = list_test_unspents(&rcv_wallet, "after draining");
    assert_eq!(unspents.len(), 0);

    // refresh receiver, if draining hadn't synced (before draining) a new UTXO would appear
    wait_for_refresh(&rcv_wallet, &rcv_online, None, None);
    let unspents = list_test_unspents(&rcv_wallet, "after receiver refresh 2");
    assert_eq!(unspents.len(), 0);
}

#[cfg(feature = "electrum")]
#[test]
#[parallel]
fn fail() {
    initialize();

    // wallets
    let (wallet, online) = get_empty_wallet!();
    let (rcv_wallet, rcv_online) = get_empty_wallet!();

    // drain empty wallet
    let result = test_drain_to_result(&wallet, &online, &test_get_address(&rcv_wallet), true);
    assert!(matches!(
        result,
        Err(Error::InsufficientBitcoins {
            needed: _,
            available: _
        })
    ));

    // bad online object
    fund_wallet(test_get_address(&wallet));
    let result = test_drain_to_result(&wallet, &rcv_online, &test_get_address(&rcv_wallet), false);
    assert!(matches!(result, Err(Error::CannotChangeOnline)));

    // bad address
    let result = test_drain_to_result(&wallet, &online, "invalid address", false);
    assert!(matches!(result, Err(Error::InvalidAddress { details: _ })));

    // fee min/max
    fund_wallet(test_get_address(&wallet));
    let result =
        test_drain_to_begin_result(&wallet, &online, &test_get_address(&rcv_wallet), true, 0.9);
    assert!(matches!(result, Err(Error::InvalidFeeRate { details: m }) if m == FEE_MSG_LOW));
    let result = test_drain_to_begin_result(
        &wallet,
        &online,
        &test_get_address(&rcv_wallet),
        true,
        1000.1,
    );
    assert!(matches!(result, Err(Error::InvalidFeeRate { details: m }) if m == FEE_MSG_HIGH));

    // no private keys
    let (wallet, online) = get_funded_noutxo_wallet(false, None);
    let result = test_drain_to_result(&wallet, &online, &test_get_address(&rcv_wallet), false);
    assert!(matches!(result, Err(Error::WatchOnly)));
}
