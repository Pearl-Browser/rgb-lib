use super::*;
use crate::wallet::backup::{restore_backup, ScryptParams};
use scrypt::Params;
use serial_test::parallel;

#[test]
#[parallel]
fn success() {
    initialize();

    let amount: u64 = 66;
    let backup_file = format!("{TEST_DATA_DIR}/test_backup_success.rgb-lib_backup");
    let password = "password";

    // wallets
    let (mut wallet, online) = get_funded_wallet!();
    let (mut rcv_wallet, rcv_online) = get_funded_wallet!();
    let mut wallet_data = wallet.wallet_data.clone();
    let wallet_dir = wallet.wallet_dir.clone();

    // issue
    let asset = test_issue_asset_nia(&mut wallet, &online, None);

    // send
    let receive_data = test_blind_receive(&mut rcv_wallet);
    let recipient_map = HashMap::from([(
        asset.asset_id.clone(),
        vec![Recipient {
            amount,
            recipient_data: RecipientData::BlindedUTXO(
                SecretSeal::from_str(&receive_data.recipient_id).unwrap(),
            ),
            transport_endpoints: TRANSPORT_ENDPOINTS.clone(),
        }],
    )]);
    let txid = test_send(&mut wallet, &online, &recipient_map);
    assert!(!txid.is_empty());
    // take transfers from WaitingCounterparty to Settled
    stop_mining();
    test_refresh_all(&mut rcv_wallet, &rcv_online);
    test_refresh_asset(&mut wallet, &online, &asset.asset_id);
    mine(true);
    test_refresh_asset(&mut rcv_wallet, &rcv_online, &asset.asset_id);
    test_refresh_asset(&mut wallet, &online, &asset.asset_id);

    // pre-backup wallet data
    check_test_wallet_data(&mut wallet, &asset, None, 1, amount);

    // backup
    println!("\nbacking up...");
    wallet.backup(&backup_file, password).unwrap();

    // backup not required after doing one
    let backup_required = wallet.backup_info().unwrap();
    assert!(!backup_required);

    // drop wallets
    drop(online);
    drop(wallet);

    // restore
    println!("\nrestoring...");
    restore_backup(&backup_file, password, RESTORE_DIR).unwrap();

    // check original and restored data are the same
    println!("\ncomparing data...");
    let restore_wallet_dir = PathBuf::from_str(RESTORE_DIR)
        .unwrap()
        .join(wallet_dir.file_name().unwrap());
    compare_test_directories(&wallet_dir, &restore_wallet_dir, &["log"]);

    // post-restore wallet data
    wallet_data.data_dir = RESTORE_DIR.to_string();
    let mut wallet = Wallet::new(wallet_data).unwrap();
    let online = test_go_online(&mut wallet, true, None);
    check_test_wallet_data(&mut wallet, &asset, None, 1, amount);

    // backup not required after restoring one
    let backup_required = wallet.backup_info().unwrap();
    assert!(!backup_required);

    // spend asset once more and check wallet data again
    let receive_data = test_blind_receive(&mut rcv_wallet);
    let recipient_map = HashMap::from([(
        asset.asset_id.clone(),
        vec![Recipient {
            amount,
            recipient_data: RecipientData::BlindedUTXO(
                SecretSeal::from_str(&receive_data.recipient_id).unwrap(),
            ),
            transport_endpoints: TRANSPORT_ENDPOINTS.clone(),
        }],
    )]);
    let txid = test_send(&mut wallet, &online, &recipient_map);
    assert!(!txid.is_empty());
    // take transfers from WaitingCounterparty to Settled
    stop_mining();
    test_refresh_all(&mut rcv_wallet, &rcv_online);
    test_refresh_asset(&mut wallet, &online, &asset.asset_id);
    mine(true);
    test_refresh_asset(&mut rcv_wallet, &rcv_online, &asset.asset_id);
    test_refresh_asset(&mut wallet, &online, &asset.asset_id);
    check_test_wallet_data(&mut wallet, &asset, None, 2, amount * 2);

    // issue a second asset with the restored wallet
    let _asset = test_issue_asset_nia(&mut wallet, &online, None);

    // cleanup
    std::fs::remove_file(&backup_file).unwrap_or_default();
}

#[test]
#[parallel]
fn fail() {
    initialize();

    let backup_file = format!("{TEST_DATA_DIR}/test_backup_fail.rgb-lib_backup");

    let (wallet, _online) = get_empty_wallet!();

    // backup
    wallet.backup(&backup_file, "password").unwrap();

    // backup on same file twice
    let result = wallet.backup(&backup_file, "password");
    assert!(matches!(result, Err(Error::FileAlreadyExists { path: _ })));

    // restore with wrong password
    let result = restore_backup(&backup_file, "wrong password", RESTORE_DIR);
    assert!(matches!(result, Err(Error::WrongPassword)));

    std::fs::remove_file(&backup_file).unwrap_or_default();
}

#[test]
#[parallel]
fn double_restore() {
    initialize();

    let amount: u64 = 66;
    let backup_file_1 = format!("{TEST_DATA_DIR}/test_double_restore_1.rgb-lib_backup");
    let backup_file_2 = format!("{TEST_DATA_DIR}/test_double_restore_2.rgb-lib_backup");
    let password_1 = "password1";
    let password_2 = "password2";

    // wallets
    let (mut wallet_1, online_1) = get_funded_wallet!();
    let (mut wallet_2, online_2) = get_funded_wallet!();
    let (mut rcv_wallet, rcv_online) = get_funded_wallet!();
    let mut wallet_1_data = wallet_1.wallet_data.clone();
    let mut wallet_2_data = wallet_2.wallet_data.clone();
    let wallet_1_dir = wallet_1.wallet_dir.clone();
    let wallet_2_dir = wallet_2.wallet_dir.clone();
    let asset_2_supply = AMOUNT * 2;

    // issue
    let asset_1 = test_issue_asset_nia(&mut wallet_1, &online_1, None);
    let asset_2 = test_issue_asset_nia(&mut wallet_2, &online_2, Some(&[asset_2_supply]));

    // send
    let receive_data_1 = test_blind_receive(&mut rcv_wallet);
    let receive_data_2 = test_blind_receive(&mut rcv_wallet);
    let recipient_map_1 = HashMap::from([(
        asset_1.asset_id.clone(),
        vec![Recipient {
            amount,
            recipient_data: RecipientData::BlindedUTXO(
                SecretSeal::from_str(&receive_data_1.recipient_id).unwrap(),
            ),
            transport_endpoints: TRANSPORT_ENDPOINTS.clone(),
        }],
    )]);
    let recipient_map_2 = HashMap::from([(
        asset_2.asset_id.clone(),
        vec![Recipient {
            amount: amount * 2,
            recipient_data: RecipientData::BlindedUTXO(
                SecretSeal::from_str(&receive_data_2.recipient_id).unwrap(),
            ),
            transport_endpoints: TRANSPORT_ENDPOINTS.clone(),
        }],
    )]);
    let txid_1 = test_send(&mut wallet_1, &online_1, &recipient_map_1);
    let txid_2 = test_send(&mut wallet_2, &online_2, &recipient_map_2);
    assert!(!txid_1.is_empty());
    assert!(!txid_2.is_empty());
    // take transfers from WaitingCounterparty to Settled
    stop_mining();
    test_refresh_all(&mut rcv_wallet, &rcv_online);
    test_refresh_asset(&mut wallet_1, &online_1, &asset_1.asset_id);
    test_refresh_asset(&mut wallet_2, &online_2, &asset_2.asset_id);
    mine(true);
    test_refresh_asset(&mut rcv_wallet, &rcv_online, &asset_1.asset_id);
    test_refresh_asset(&mut wallet_1, &online_1, &asset_1.asset_id);
    test_refresh_asset(&mut wallet_2, &online_2, &asset_2.asset_id);

    // pre-backup wallet data
    check_test_wallet_data(&mut wallet_1, &asset_1, None, 1, amount);
    check_test_wallet_data(&mut wallet_2, &asset_2, Some(asset_2_supply), 1, amount * 2);

    // backup
    println!("\nbacking up...");
    wallet_1.backup(&backup_file_1, password_1).unwrap();
    let custom_params = ScryptParams::new(
        Some(Params::RECOMMENDED_LOG_N + 1),
        Some(Params::RECOMMENDED_R + 1),
        Some(Params::RECOMMENDED_P + 1),
    );
    wallet_2
        .backup_customize(&backup_file_2, password_2, Some(custom_params))
        .unwrap();

    // drop wallets
    drop(online_1);
    drop(wallet_1);
    drop(online_2);
    drop(wallet_2);

    // restore
    println!("\nrestoring...");
    restore_backup(&backup_file_1, password_1, RESTORE_DIR).unwrap();
    restore_backup(&backup_file_2, password_2, RESTORE_DIR).unwrap();

    // check original and restored data are the same
    println!("\ncomparing data for wallet 1...");
    let restore_wallet_1_dir = PathBuf::from_str(RESTORE_DIR)
        .unwrap()
        .join(wallet_1_dir.file_name().unwrap());
    compare_test_directories(&wallet_1_dir, &restore_wallet_1_dir, &["log"]);
    let restore_wallet_2_dir = PathBuf::from_str(RESTORE_DIR)
        .unwrap()
        .join(wallet_2_dir.file_name().unwrap());
    compare_test_directories(&wallet_2_dir, &restore_wallet_2_dir, &["log"]);

    // post-restore wallet data
    wallet_1_data.data_dir = RESTORE_DIR.to_string();
    wallet_2_data.data_dir = RESTORE_DIR.to_string();
    let mut wallet_1 = Wallet::new(wallet_1_data).unwrap();
    let mut wallet_2 = Wallet::new(wallet_2_data).unwrap();
    let online_1 = test_go_online(&mut wallet_1, true, None);
    let online_2 = test_go_online(&mut wallet_2, true, None);
    check_test_wallet_data(&mut wallet_1, &asset_1, None, 1, amount);
    check_test_wallet_data(&mut wallet_2, &asset_2, Some(asset_2_supply), 1, amount * 2);

    // issue a second asset with the restored wallets
    test_issue_asset_nia(&mut wallet_1, &online_1, None);
    test_issue_asset_nia(&mut wallet_2, &online_2, None);

    // cleanup
    std::fs::remove_file(&backup_file_1).unwrap_or_default();
    std::fs::remove_file(&backup_file_2).unwrap_or_default();
}

#[test]
#[parallel]
fn backup_info() {
    initialize();

    // wallets
    let (wallet, _online) = get_empty_wallet!();

    // backup not required for new wallets
    let backup_required = wallet.backup_info().unwrap();
    assert!(!backup_required);
}
